import { pwaService } from '../pwaService';

// Mock service worker
const mockServiceWorker = {
  register: jest.fn(),
  addEventListener: jest.fn(),
  postMessage: jest.fn(),
};

const mockRegistration = {
  installing: null,
  waiting: null,
  active: mockServiceWorker,
  addEventListener: jest.fn(),
  update: jest.fn(),
  showNotification: jest.fn(),
  pushManager: {
    subscribe: jest.fn(),
    getSubscription: jest.fn(),
  },
};

// Mock navigator
Object.defineProperty(navigator, 'serviceWorker', {
  value: {
    register: jest.fn().mockResolvedValue(mockRegistration),
    addEventListener: jest.fn(),
  },
  writable: true,
});

Object.defineProperty(navigator, 'onLine', {
  value: true,
  writable: true,
});

// Mock Notification
Object.defineProperty(window, 'Notification', {
  value: {
    permission: 'default',
    requestPermission: jest.fn().mockResolvedValue('granted'),
  },
  writable: true,
});

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  value: jest.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(),
    removeListener: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
  writable: true,
});

describe('PWAService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Service Worker Registration', () => {
    it('should register service worker successfully', async () => {
      const registration = await pwaService.registerServiceWorker();
      
      expect(navigator.serviceWorker.register).toHaveBeenCalledWith('/sw.js', {
        scope: '/',
      });
      expect(registration).toBe(mockRegistration);
    });

    it('should handle service worker registration failure', async () => {
      (navigator.serviceWorker.register as jest.Mock).mockRejectedValueOnce(
        new Error('Registration failed')
      );

      const registration = await pwaService.registerServiceWorker();
      expect(registration).toBeNull();
    });

    it('should return null if service worker is not supported', async () => {
      const originalServiceWorker = navigator.serviceWorker;
      delete (navigator as any).serviceWorker;

      const registration = await pwaService.registerServiceWorker();
      expect(registration).toBeNull();

      (navigator as any).serviceWorker = originalServiceWorker;
    });
  });

  describe('Installation', () => {
    it('should detect if app can be installed', () => {
      expect(pwaService.canInstall()).toBe(false);
    });

    it('should detect if app is installed', () => {
      expect(pwaService.isInstalled()).toBe(false);
    });

    it('should handle install prompt', async () => {
      // Simulate install prompt event
      const mockPrompt = {
        prompt: jest.fn().mockResolvedValue(undefined),
        userChoice: Promise.resolve({ outcome: 'accepted' }),
      };

      // Trigger beforeinstallprompt event
      const event = new Event('beforeinstallprompt') as any;
      event.prompt = mockPrompt.prompt;
      event.userChoice = mockPrompt.userChoice;
      event.preventDefault = jest.fn();

      window.dispatchEvent(event);

      const result = await pwaService.install();
      expect(result).toBe(true);
    });
  });

  describe('Notifications', () => {
    it('should get notification permission status', () => {
      const permission = pwaService.getNotificationPermission();
      
      expect(permission).toEqual({
        granted: false,
        denied: false,
        default: true,
      });
    });

    it('should request notification permission', async () => {
      const granted = await pwaService.requestNotificationPermission();
      
      expect(Notification.requestPermission).toHaveBeenCalled();
      expect(granted).toBe(true);
    });

    it('should handle notification permission denial', async () => {
      (Notification.requestPermission as jest.Mock).mockResolvedValueOnce('denied');
      
      const granted = await pwaService.requestNotificationPermission();
      expect(granted).toBe(false);
    });

    it('should return false if notifications are not supported', async () => {
      const originalNotification = window.Notification;
      delete (window as any).Notification;

      const granted = await pwaService.requestNotificationPermission();
      expect(granted).toBe(false);

      (window as any).Notification = originalNotification;
    });
  });

  describe('Push Notifications', () => {
    const mockSubscription = {
      endpoint: 'https://example.com/push',
      getKey: jest.fn(),
    };

    beforeEach(() => {
      mockSubscription.getKey.mockImplementation((name: string) => {
        if (name === 'p256dh') return new ArrayBuffer(8);
        if (name === 'auth') return new ArrayBuffer(8);
        return null;
      });
    });

    it('should subscribe to push notifications', async () => {
      mockRegistration.pushManager.subscribe.mockResolvedValueOnce(mockSubscription);
      
      // Mock service worker registration
      await pwaService.registerServiceWorker();
      
      const subscription = await pwaService.subscribeToPush('test-vapid-key');
      
      expect(subscription).toEqual({
        endpoint: 'https://example.com/push',
        keys: {
          p256dh: expect.any(String),
          auth: expect.any(String),
        },
      });
    });

    it('should handle push subscription failure', async () => {
      mockRegistration.pushManager.subscribe.mockRejectedValueOnce(
        new Error('Subscription failed')
      );
      
      await pwaService.registerServiceWorker();
      
      const subscription = await pwaService.subscribeToPush('test-vapid-key');
      expect(subscription).toBeNull();
    });

    it('should unsubscribe from push notifications', async () => {
      const mockUnsubscribe = jest.fn().mockResolvedValue(true);
      mockRegistration.pushManager.getSubscription.mockResolvedValueOnce({
        unsubscribe: mockUnsubscribe,
      });
      
      await pwaService.registerServiceWorker();
      
      const result = await pwaService.unsubscribeFromPush();
      expect(result).toBe(true);
      expect(mockUnsubscribe).toHaveBeenCalled();
    });
  });

  describe('App Info', () => {
    it('should return app info', () => {
      const appInfo = pwaService.getAppInfo();
      
      expect(appInfo).toEqual({
        isInstalled: false,
        canInstall: false,
        isOnline: true,
        notificationPermission: {
          granted: false,
          denied: false,
          default: true,
        },
        serviceWorkerSupported: true,
        pushSupported: true,
      });
    });
  });

  describe('Online Status', () => {
    it('should track online status', () => {
      expect(pwaService.isOnlineStatus()).toBe(true);
      
      // Simulate going offline
      Object.defineProperty(navigator, 'onLine', { value: false });
      window.dispatchEvent(new Event('offline'));
      
      expect(pwaService.isOnlineStatus()).toBe(false);
    });
  });

  describe('Caching', () => {
    it('should cache data in localStorage as fallback', async () => {
      const testData = { test: 'data' };
      
      await pwaService.cacheData('test-key', testData);
      
      const cached = await pwaService.getCachedData('test-key');
      expect(cached).toEqual(testData);
    });

    it('should handle cache errors gracefully', async () => {
      // Mock localStorage to throw error
      const originalSetItem = localStorage.setItem;
      localStorage.setItem = jest.fn().mockImplementation(() => {
        throw new Error('Storage full');
      });

      await pwaService.cacheData('test-key', { test: 'data' });
      
      // Should not throw error
      expect(true).toBe(true);
      
      localStorage.setItem = originalSetItem;
    });
  });
});