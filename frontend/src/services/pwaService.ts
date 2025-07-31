// PWA Service for managing service worker and push notifications
import React from 'react';

export interface PWAInstallPrompt {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>;
}

export interface NotificationPermission {
  granted: boolean;
  denied: boolean;
  default: boolean;
}

export interface PushSubscription {
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
}

class PWAService {
  private swRegistration: ServiceWorkerRegistration | null = null;
  private installPrompt: PWAInstallPrompt | null = null;
  private isOnline = navigator.onLine;

  constructor() {
    this.initializeEventListeners();
  }

  // Initialize PWA event listeners
  private initializeEventListeners() {
    // Listen for install prompt
    window.addEventListener('beforeinstallprompt', (e) => {
      e.preventDefault();
      this.installPrompt = e as any;
      this.dispatchCustomEvent('pwa-installable', { canInstall: true });
    });

    // Listen for app installed
    window.addEventListener('appinstalled', () => {
      this.installPrompt = null;
      this.dispatchCustomEvent('pwa-installed', { installed: true });
    });

    // Listen for online/offline status
    window.addEventListener('online', () => {
      this.isOnline = true;
      this.dispatchCustomEvent('pwa-online', { online: true });
    });

    window.addEventListener('offline', () => {
      this.isOnline = false;
      this.dispatchCustomEvent('pwa-offline', { online: false });
    });

    // Listen for visibility change (app focus/blur)
    document.addEventListener('visibilitychange', () => {
      this.dispatchCustomEvent('pwa-visibility-change', {
        visible: !document.hidden,
      });
    });
  }

  // Register service worker
  async registerServiceWorker(): Promise<ServiceWorkerRegistration | null> {
    if (!('serviceWorker' in navigator)) {
      console.warn('Service Worker not supported');
      return null;
    }

    try {
      this.swRegistration = await navigator.serviceWorker.register('/sw.js', {
        scope: '/',
      });

      console.log('Service Worker registered successfully');

      // Listen for updates
      this.swRegistration.addEventListener('updatefound', () => {
        const newWorker = this.swRegistration?.installing;
        if (newWorker) {
          newWorker.addEventListener('statechange', () => {
            if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
              this.dispatchCustomEvent('pwa-update-available', {
                registration: this.swRegistration,
              });
            }
          });
        }
      });

      return this.swRegistration;
    } catch (error) {
      console.error('Service Worker registration failed:', error);
      return null;
    }
  }

  // Update service worker
  async updateServiceWorker(): Promise<void> {
    if (this.swRegistration) {
      await this.swRegistration.update();
      
      // Tell the new service worker to skip waiting
      if (this.swRegistration.waiting) {
        this.swRegistration.waiting.postMessage({ type: 'SKIP_WAITING' });
        window.location.reload();
      }
    }
  }

  // Check if app can be installed
  canInstall(): boolean {
    return this.installPrompt !== null;
  }

  // Trigger install prompt
  async install(): Promise<boolean> {
    if (!this.installPrompt) {
      return false;
    }

    try {
      await this.installPrompt.prompt();
      const choiceResult = await this.installPrompt.userChoice;
      
      if (choiceResult.outcome === 'accepted') {
        this.installPrompt = null;
        return true;
      }
      
      return false;
    } catch (error) {
      console.error('Install prompt failed:', error);
      return false;
    }
  }

  // Check if app is installed
  isInstalled(): boolean {
    return window.matchMedia('(display-mode: standalone)').matches ||
           (window.navigator as any).standalone === true;
  }

  // Get notification permission status
  getNotificationPermission(): NotificationPermission {
    if (!('Notification' in window)) {
      return { granted: false, denied: true, default: false };
    }

    const permission = Notification.permission;
    return {
      granted: permission === 'granted',
      denied: permission === 'denied',
      default: permission === 'default',
    };
  }

  // Request notification permission
  async requestNotificationPermission(): Promise<boolean> {
    if (!('Notification' in window)) {
      console.warn('Notifications not supported');
      return false;
    }

    if (Notification.permission === 'granted') {
      return true;
    }

    if (Notification.permission === 'denied') {
      return false;
    }

    try {
      const permission = await Notification.requestPermission();
      return permission === 'granted';
    } catch (error) {
      console.error('Notification permission request failed:', error);
      return false;
    }
  }

  // Subscribe to push notifications
  async subscribeToPush(vapidPublicKey: string): Promise<PushSubscription | null> {
    if (!this.swRegistration) {
      console.error('Service Worker not registered');
      return null;
    }

    if (!('PushManager' in window)) {
      console.warn('Push notifications not supported');
      return null;
    }

    try {
      const subscription = await this.swRegistration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: this.urlBase64ToUint8Array(vapidPublicKey),
      });

      const p256dh = subscription.getKey('p256dh');
      const auth = subscription.getKey('auth');

      if (!p256dh || !auth) {
        throw new Error('Failed to get subscription keys');
      }

      return {
        endpoint: subscription.endpoint,
        keys: {
          p256dh: this.arrayBufferToBase64(p256dh),
          auth: this.arrayBufferToBase64(auth),
        },
      };
    } catch (error) {
      console.error('Push subscription failed:', error);
      return null;
    }
  }

  // Unsubscribe from push notifications
  async unsubscribeFromPush(): Promise<boolean> {
    if (!this.swRegistration) {
      return false;
    }

    try {
      const subscription = await this.swRegistration.pushManager.getSubscription();
      if (subscription) {
        await subscription.unsubscribe();
        return true;
      }
      return false;
    } catch (error) {
      console.error('Push unsubscription failed:', error);
      return false;
    }
  }

  // Get current push subscription
  async getPushSubscription(): Promise<PushSubscription | null> {
    if (!this.swRegistration) {
      return null;
    }

    try {
      const subscription = await this.swRegistration.pushManager.getSubscription();
      if (!subscription) {
        return null;
      }

      const p256dh = subscription.getKey('p256dh');
      const auth = subscription.getKey('auth');

      if (!p256dh || !auth) {
        return null;
      }

      return {
        endpoint: subscription.endpoint,
        keys: {
          p256dh: this.arrayBufferToBase64(p256dh),
          auth: this.arrayBufferToBase64(auth),
        },
      };
    } catch (error) {
      console.error('Failed to get push subscription:', error);
      return null;
    }
  }

  // Show local notification
  async showNotification(
    title: string,
    options: NotificationOptions = {}
  ): Promise<void> {
    if (!this.swRegistration) {
      // Fallback to browser notification
      if ('Notification' in window && Notification.permission === 'granted') {
        new Notification(title, options);
      }
      return;
    }

    try {
      await this.swRegistration.showNotification(title, {
        icon: '/icon-192x192.png',
        badge: '/badge-72x72.png',
        ...options,
      });
    } catch (error) {
      console.error('Failed to show notification:', error);
    }
  }

  // Cache data for offline use
  async cacheData(key: string, data: any): Promise<void> {
    if (!this.swRegistration) {
      // Fallback to localStorage
      try {
        localStorage.setItem(`pwa_cache_${key}`, JSON.stringify(data));
      } catch (error) {
        console.error('Failed to cache data in localStorage:', error);
      }
      return;
    }

    try {
      this.swRegistration.active?.postMessage({
        type: 'CACHE_DATA',
        key,
        data,
      });
    } catch (error) {
      console.error('Failed to cache data:', error);
    }
  }

  // Get cached data
  async getCachedData(key: string): Promise<any> {
    // Try localStorage first
    try {
      const cached = localStorage.getItem(`pwa_cache_${key}`);
      if (cached) {
        return JSON.parse(cached);
      }
    } catch (error) {
      console.error('Failed to get cached data from localStorage:', error);
    }

    return null;
  }

  // Check online status
  isOnlineStatus(): boolean {
    return this.isOnline;
  }

  // Get app info
  getAppInfo() {
    return {
      isInstalled: this.isInstalled(),
      canInstall: this.canInstall(),
      isOnline: this.isOnline,
      notificationPermission: this.getNotificationPermission(),
      serviceWorkerSupported: 'serviceWorker' in navigator,
      pushSupported: 'PushManager' in window,
    };
  }

  // Utility methods
  private urlBase64ToUint8Array(base64String: string): Uint8Array {
    const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
    const base64 = (base64String + padding)
      .replace(/-/g, '+')
      .replace(/_/g, '/');

    const rawData = window.atob(base64);
    const outputArray = new Uint8Array(rawData.length);

    for (let i = 0; i < rawData.length; ++i) {
      outputArray[i] = rawData.charCodeAt(i);
    }
    return outputArray;
  }

  private arrayBufferToBase64(buffer: ArrayBuffer): string {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary);
  }

  private dispatchCustomEvent(eventName: string, detail: any) {
    window.dispatchEvent(new CustomEvent(eventName, { detail }));
  }
}

// Create singleton instance
export const pwaService = new PWAService();

// React hook for PWA functionality
export const usePWA = () => {
  const [appInfo, setAppInfo] = React.useState(pwaService.getAppInfo());
  const [updateAvailable, setUpdateAvailable] = React.useState(false);

  React.useEffect(() => {
    const handleInstallable = () => {
      setAppInfo(pwaService.getAppInfo());
    };

    const handleInstalled = () => {
      setAppInfo(pwaService.getAppInfo());
    };

    const handleUpdateAvailable = () => {
      setUpdateAvailable(true);
    };

    const handleOnlineStatus = () => {
      setAppInfo(pwaService.getAppInfo());
    };

    window.addEventListener('pwa-installable', handleInstallable);
    window.addEventListener('pwa-installed', handleInstalled);
    window.addEventListener('pwa-update-available', handleUpdateAvailable);
    window.addEventListener('pwa-online', handleOnlineStatus);
    window.addEventListener('pwa-offline', handleOnlineStatus);

    return () => {
      window.removeEventListener('pwa-installable', handleInstallable);
      window.removeEventListener('pwa-installed', handleInstalled);
      window.removeEventListener('pwa-update-available', handleUpdateAvailable);
      window.removeEventListener('pwa-online', handleOnlineStatus);
      window.removeEventListener('pwa-offline', handleOnlineStatus);
    };
  }, []);

  return {
    ...appInfo,
    updateAvailable,
    install: pwaService.install.bind(pwaService),
    updateApp: pwaService.updateServiceWorker.bind(pwaService),
    requestNotifications: pwaService.requestNotificationPermission.bind(pwaService),
    subscribeToPush: pwaService.subscribeToPush.bind(pwaService),
    unsubscribeFromPush: pwaService.unsubscribeFromPush.bind(pwaService),
    showNotification: pwaService.showNotification.bind(pwaService),
  };
};

export default pwaService;