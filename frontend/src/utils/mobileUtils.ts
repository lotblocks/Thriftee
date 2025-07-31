// Mobile utility functions for responsive design and mobile-specific features

export interface DeviceInfo {
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  isIOS: boolean;
  isAndroid: boolean;
  isSafari: boolean;
  isChrome: boolean;
  hasTouch: boolean;
  screenWidth: number;
  screenHeight: number;
  pixelRatio: number;
  orientation: 'portrait' | 'landscape';
}

// Detect device type and capabilities
export const getDeviceInfo = (): DeviceInfo => {
  const userAgent = navigator.userAgent.toLowerCase();
  const screenWidth = window.innerWidth;
  const screenHeight = window.innerHeight;
  
  return {
    isMobile: screenWidth < 768,
    isTablet: screenWidth >= 768 && screenWidth < 1024,
    isDesktop: screenWidth >= 1024,
    isIOS: /iphone|ipad|ipod/.test(userAgent),
    isAndroid: /android/.test(userAgent),
    isSafari: /safari/.test(userAgent) && !/chrome/.test(userAgent),
    isChrome: /chrome/.test(userAgent),
    hasTouch: 'ontouchstart' in window || navigator.maxTouchPoints > 0,
    screenWidth,
    screenHeight,
    pixelRatio: window.devicePixelRatio || 1,
    orientation: screenWidth > screenHeight ? 'landscape' : 'portrait',
  };
};

// Safe area utilities for devices with notches
export const getSafeAreaInsets = () => {
  const style = getComputedStyle(document.documentElement);
  return {
    top: parseInt(style.getPropertyValue('--safe-area-inset-top') || '0', 10),
    right: parseInt(style.getPropertyValue('--safe-area-inset-right') || '0', 10),
    bottom: parseInt(style.getPropertyValue('--safe-area-inset-bottom') || '0', 10),
    left: parseInt(style.getPropertyValue('--safe-area-inset-left') || '0', 10),
  };
};

// Viewport height utilities (handles mobile browser address bar)
export const getViewportHeight = () => {
  return window.visualViewport?.height || window.innerHeight;
};

export const setViewportHeight = () => {
  const vh = getViewportHeight() * 0.01;
  document.documentElement.style.setProperty('--vh', `${vh}px`);
};

// Touch target size utilities
export const TOUCH_TARGET_SIZE = {
  MINIMUM: 44, // iOS HIG minimum
  RECOMMENDED: 48, // Material Design recommendation
  COMFORTABLE: 56, // Comfortable for most users
};

export const ensureMinimumTouchTarget = (size: number): number => {
  return Math.max(size, TOUCH_TARGET_SIZE.MINIMUM);
};

// Grid calculation utilities for mobile
export const calculateMobileGridDimensions = (
  totalItems: number,
  containerWidth: number,
  minCellSize: number = TOUCH_TARGET_SIZE.MINIMUM,
  maxCols?: number
) => {
  const availableWidth = containerWidth - 32; // Account for padding
  const maxPossibleCols = Math.floor(availableWidth / minCellSize);
  const cols = maxCols ? Math.min(maxCols, maxPossibleCols) : maxPossibleCols;
  const rows = Math.ceil(totalItems / cols);
  const cellSize = Math.floor(availableWidth / cols);
  
  return {
    cols,
    rows,
    cellSize: Math.max(cellSize, minCellSize),
    totalWidth: cols * cellSize,
    totalHeight: rows * cellSize,
  };
};

// Scroll utilities
export const preventBodyScroll = () => {
  document.body.style.overflow = 'hidden';
  document.body.style.position = 'fixed';
  document.body.style.width = '100%';
};

export const restoreBodyScroll = () => {
  document.body.style.overflow = '';
  document.body.style.position = '';
  document.body.style.width = '';
};

// Performance utilities for mobile
export const throttle = <T extends (...args: any[]) => any>(
  func: T,
  delay: number
): ((...args: Parameters<T>) => void) => {
  let timeoutId: NodeJS.Timeout | null = null;
  let lastExecTime = 0;
  
  return (...args: Parameters<T>) => {
    const currentTime = Date.now();
    
    if (currentTime - lastExecTime > delay) {
      func(...args);
      lastExecTime = currentTime;
    } else {
      if (timeoutId) clearTimeout(timeoutId);
      timeoutId = setTimeout(() => {
        func(...args);
        lastExecTime = Date.now();
      }, delay - (currentTime - lastExecTime));
    }
  };
};

export const debounce = <T extends (...args: any[]) => any>(
  func: T,
  delay: number
): ((...args: Parameters<T>) => void) => {
  let timeoutId: NodeJS.Timeout | null = null;
  
  return (...args: Parameters<T>) => {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => func(...args), delay);
  };
};

// Image optimization for mobile
export const getOptimizedImageUrl = (
  originalUrl: string,
  width: number,
  height?: number,
  quality: number = 80
): string => {
  // This would integrate with your image optimization service
  // For now, return the original URL
  return originalUrl;
};

// Network detection
export const getNetworkInfo = () => {
  const connection = (navigator as any).connection || 
                    (navigator as any).mozConnection || 
                    (navigator as any).webkitConnection;
  
  if (!connection) {
    return {
      effectiveType: 'unknown',
      downlink: undefined,
      rtt: undefined,
      saveData: false,
    };
  }
  
  return {
    effectiveType: connection.effectiveType || 'unknown',
    downlink: connection.downlink,
    rtt: connection.rtt,
    saveData: connection.saveData || false,
  };
};

// Battery API utilities
export const getBatteryInfo = async () => {
  if ('getBattery' in navigator) {
    try {
      const battery = await (navigator as any).getBattery();
      return {
        level: battery.level,
        charging: battery.charging,
        chargingTime: battery.chargingTime,
        dischargingTime: battery.dischargingTime,
      };
    } catch (error) {
      return null;
    }
  }
  return null;
};

// Memory utilities
export const getMemoryInfo = () => {
  const memory = (performance as any).memory;
  if (!memory) return null;
  
  return {
    usedJSHeapSize: memory.usedJSHeapSize,
    totalJSHeapSize: memory.totalJSHeapSize,
    jsHeapSizeLimit: memory.jsHeapSizeLimit,
  };
};

// Orientation utilities
export const lockOrientation = async (orientation: OrientationLockType) => {
  if ('orientation' in screen && 'lock' in screen.orientation) {
    try {
      await screen.orientation.lock(orientation);
      return true;
    } catch (error) {
      console.warn('Failed to lock orientation:', error);
      return false;
    }
  }
  return false;
};

export const unlockOrientation = () => {
  if ('orientation' in screen && 'unlock' in screen.orientation) {
    screen.orientation.unlock();
  }
};

// Fullscreen utilities
export const enterFullscreen = async (element?: Element) => {
  const target = element || document.documentElement;
  
  if (target.requestFullscreen) {
    await target.requestFullscreen();
  } else if ((target as any).webkitRequestFullscreen) {
    await (target as any).webkitRequestFullscreen();
  } else if ((target as any).msRequestFullscreen) {
    await (target as any).msRequestFullscreen();
  }
};

export const exitFullscreen = async () => {
  if (document.exitFullscreen) {
    await document.exitFullscreen();
  } else if ((document as any).webkitExitFullscreen) {
    await (document as any).webkitExitFullscreen();
  } else if ((document as any).msExitFullscreen) {
    await (document as any).msExitFullscreen();
  }
};

// Clipboard utilities
export const copyToClipboard = async (text: string): Promise<boolean> => {
  if (navigator.clipboard && navigator.clipboard.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch (error) {
      console.warn('Failed to copy to clipboard:', error);
    }
  }
  
  // Fallback for older browsers
  try {
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.style.position = 'fixed';
    textArea.style.opacity = '0';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    const successful = document.execCommand('copy');
    document.body.removeChild(textArea);
    return successful;
  } catch (error) {
    console.warn('Fallback copy failed:', error);
    return false;
  }
};

// Share API utilities
export const canShare = (data?: ShareData): boolean => {
  return 'share' in navigator && (!data || navigator.canShare?.(data) !== false);
};

export const shareContent = async (data: ShareData): Promise<boolean> => {
  if (canShare(data)) {
    try {
      await navigator.share(data);
      return true;
    } catch (error) {
      if ((error as Error).name !== 'AbortError') {
        console.warn('Share failed:', error);
      }
      return false;
    }
  }
  
  // Fallback to copying URL
  if (data.url) {
    return await copyToClipboard(data.url);
  }
  
  return false;
};

// PWA utilities
export const isPWA = (): boolean => {
  return window.matchMedia('(display-mode: standalone)').matches ||
         (window.navigator as any).standalone === true;
};

export const canInstallPWA = (): boolean => {
  return 'serviceWorker' in navigator && 'PushManager' in window;
};

// Accessibility utilities
export const announceToScreenReader = (message: string) => {
  const announcement = document.createElement('div');
  announcement.setAttribute('aria-live', 'polite');
  announcement.setAttribute('aria-atomic', 'true');
  announcement.style.position = 'absolute';
  announcement.style.left = '-10000px';
  announcement.style.width = '1px';
  announcement.style.height = '1px';
  announcement.style.overflow = 'hidden';
  
  document.body.appendChild(announcement);
  announcement.textContent = message;
  
  setTimeout(() => {
    document.body.removeChild(announcement);
  }, 1000);
};

// Focus management
export const trapFocus = (element: HTMLElement) => {
  const focusableElements = element.querySelectorAll(
    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
  );
  
  const firstElement = focusableElements[0] as HTMLElement;
  const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;
  
  const handleTabKey = (e: KeyboardEvent) => {
    if (e.key === 'Tab') {
      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          lastElement.focus();
          e.preventDefault();
        }
      } else {
        if (document.activeElement === lastElement) {
          firstElement.focus();
          e.preventDefault();
        }
      }
    }
  };
  
  element.addEventListener('keydown', handleTabKey);
  firstElement?.focus();
  
  return () => {
    element.removeEventListener('keydown', handleTabKey);
  };
};

// CSS custom properties utilities
export const setCSSCustomProperty = (property: string, value: string) => {
  document.documentElement.style.setProperty(property, value);
};

export const getCSSCustomProperty = (property: string): string => {
  return getComputedStyle(document.documentElement).getPropertyValue(property);
};

// Mobile-specific event utilities
export const addMobileEventListeners = () => {
  // Update viewport height on resize (mobile browser address bar)
  window.addEventListener('resize', setViewportHeight);
  window.addEventListener('orientationchange', () => {
    setTimeout(setViewportHeight, 100);
  });
  
  // Set initial viewport height
  setViewportHeight();
  
  // Add safe area CSS custom properties
  const safeAreaInsets = getSafeAreaInsets();
  setCSSCustomProperty('--safe-area-top', `${safeAreaInsets.top}px`);
  setCSSCustomProperty('--safe-area-right', `${safeAreaInsets.right}px`);
  setCSSCustomProperty('--safe-area-bottom', `${safeAreaInsets.bottom}px`);
  setCSSCustomProperty('--safe-area-left', `${safeAreaInsets.left}px`);
};

// Initialize mobile utilities
export const initializeMobileUtils = () => {
  addMobileEventListeners();
  
  // Add device classes to body
  const deviceInfo = getDeviceInfo();
  const classes = [];
  
  if (deviceInfo.isMobile) classes.push('is-mobile');
  if (deviceInfo.isTablet) classes.push('is-tablet');
  if (deviceInfo.isDesktop) classes.push('is-desktop');
  if (deviceInfo.isIOS) classes.push('is-ios');
  if (deviceInfo.isAndroid) classes.push('is-android');
  if (deviceInfo.hasTouch) classes.push('has-touch');
  if (isPWA()) classes.push('is-pwa');
  
  document.body.classList.add(...classes);
};