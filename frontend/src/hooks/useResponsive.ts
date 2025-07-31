import { useState, useEffect } from 'react';

interface ResponsiveState {
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  screenWidth: number;
  screenHeight: number;
  orientation: 'portrait' | 'landscape';
  isTouchDevice: boolean;
  devicePixelRatio: number;
}

// Tailwind CSS breakpoints
const BREAKPOINTS = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  '2xl': 1536,
};

export const useResponsive = (): ResponsiveState => {
  const [responsiveState, setResponsiveState] = useState<ResponsiveState>(() => {
    if (typeof window === 'undefined') {
      return {
        isMobile: false,
        isTablet: false,
        isDesktop: true,
        screenWidth: 1024,
        screenHeight: 768,
        orientation: 'landscape',
        isTouchDevice: false,
        devicePixelRatio: 1,
      };
    }

    const width = window.innerWidth;
    return {
      isMobile: width < BREAKPOINTS.md,
      isTablet: width >= BREAKPOINTS.md && width < BREAKPOINTS.lg,
      isDesktop: width >= BREAKPOINTS.lg,
      screenWidth: width,
      screenHeight: window.innerHeight,
      orientation: width > window.innerHeight ? 'landscape' : 'portrait',
      isTouchDevice: 'ontouchstart' in window,
      devicePixelRatio: window.devicePixelRatio,
    };
  });

  useEffect(() => {
    const updateResponsiveState = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;

      setResponsiveState({
        isMobile: width < BREAKPOINTS.md,
        isTablet: width >= BREAKPOINTS.md && width < BREAKPOINTS.lg,
        isDesktop: width >= BREAKPOINTS.lg,
        screenWidth: width,
        screenHeight: height,
        orientation: width > height ? 'landscape' : 'portrait',
        isTouchDevice: 'ontouchstart' in window,
        devicePixelRatio: window.devicePixelRatio,
      });
    };

    // Update on resize
    window.addEventListener('resize', updateResponsiveState);
    
    // Update on orientation change
    window.addEventListener('orientationchange', () => {
      // Delay to allow for orientation change to complete
      setTimeout(updateResponsiveState, 100);
    });

    return () => {
      window.removeEventListener('resize', updateResponsiveState);
      window.removeEventListener('orientationchange', updateResponsiveState);
    };
  }, []);

  return responsiveState;
};

// Hook for detecting specific device types
export const useDeviceType = () => {
  const [deviceType, setDeviceType] = useState<'mobile' | 'tablet' | 'desktop'>('desktop');

  useEffect(() => {
    const detectDevice = () => {
      const userAgent = navigator.userAgent.toLowerCase();
      const width = window.innerWidth;

      // Check for mobile devices
      if (/android|webos|iphone|ipad|ipod|blackberry|iemobile|opera mini/i.test(userAgent)) {
        if (width < BREAKPOINTS.md) {
          setDeviceType('mobile');
        } else {
          setDeviceType('tablet');
        }
      } else {
        // Desktop
        if (width < BREAKPOINTS.md) {
          setDeviceType('mobile');
        } else if (width < BREAKPOINTS.lg) {
          setDeviceType('tablet');
        } else {
          setDeviceType('desktop');
        }
      }
    };

    detectDevice();
    window.addEventListener('resize', detectDevice);

    return () => {
      window.removeEventListener('resize', detectDevice);
    };
  }, []);

  return deviceType;
};

// Hook for detecting touch capabilities
export const useTouchDevice = () => {
  const [isTouchDevice, setIsTouchDevice] = useState(false);

  useEffect(() => {
    const checkTouchDevice = () => {
      setIsTouchDevice(
        'ontouchstart' in window ||
        navigator.maxTouchPoints > 0 ||
        // @ts-ignore
        navigator.msMaxTouchPoints > 0
      );
    };

    checkTouchDevice();
  }, []);

  return isTouchDevice;
};

// Hook for viewport dimensions
export const useViewport = () => {
  const [viewport, setViewport] = useState({
    width: typeof window !== 'undefined' ? window.innerWidth : 0,
    height: typeof window !== 'undefined' ? window.innerHeight : 0,
  });

  useEffect(() => {
    const updateViewport = () => {
      setViewport({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    window.addEventListener('resize', updateViewport);
    window.addEventListener('orientationchange', updateViewport);

    return () => {
      window.removeEventListener('resize', updateViewport);
      window.removeEventListener('orientationchange', updateViewport);
    };
  }, []);

  return viewport;
};

// Hook for breakpoint matching
export const useBreakpoint = (breakpoint: keyof typeof BREAKPOINTS) => {
  const [matches, setMatches] = useState(false);

  useEffect(() => {
    const checkBreakpoint = () => {
      setMatches(window.innerWidth >= BREAKPOINTS[breakpoint]);
    };

    checkBreakpoint();
    window.addEventListener('resize', checkBreakpoint);

    return () => {
      window.removeEventListener('resize', checkBreakpoint);
    };
  }, [breakpoint]);

  return matches;
};

// Hook for safe area insets (for devices with notches)
export const useSafeArea = () => {
  const [safeArea, setSafeArea] = useState({
    top: 0,
    right: 0,
    bottom: 0,
    left: 0,
  });

  useEffect(() => {
    const updateSafeArea = () => {
      const computedStyle = getComputedStyle(document.documentElement);
      
      setSafeArea({
        top: parseInt(computedStyle.getPropertyValue('--safe-area-inset-top') || '0'),
        right: parseInt(computedStyle.getPropertyValue('--safe-area-inset-right') || '0'),
        bottom: parseInt(computedStyle.getPropertyValue('--safe-area-inset-bottom') || '0'),
        left: parseInt(computedStyle.getPropertyValue('--safe-area-inset-left') || '0'),
      });
    };

    updateSafeArea();
    window.addEventListener('resize', updateSafeArea);
    window.addEventListener('orientationchange', updateSafeArea);

    return () => {
      window.removeEventListener('resize', updateSafeArea);
      window.removeEventListener('orientationchange', updateSafeArea);
    };
  }, []);

  return safeArea;
};

// Hook for detecting network connection
export const useNetworkStatus = () => {
  const [isOnline, setIsOnline] = useState(navigator.onLine);
  const [connectionType, setConnectionType] = useState<string>('unknown');

  useEffect(() => {
    const updateOnlineStatus = () => {
      setIsOnline(navigator.onLine);
    };

    const updateConnectionType = () => {
      // @ts-ignore
      const connection = navigator.connection || navigator.mozConnection || navigator.webkitConnection;
      if (connection) {
        setConnectionType(connection.effectiveType || 'unknown');
      }
    };

    window.addEventListener('online', updateOnlineStatus);
    window.addEventListener('offline', updateOnlineStatus);
    
    // @ts-ignore
    if (navigator.connection) {
      // @ts-ignore
      navigator.connection.addEventListener('change', updateConnectionType);
    }

    updateConnectionType();

    return () => {
      window.removeEventListener('online', updateOnlineStatus);
      window.removeEventListener('offline', updateOnlineStatus);
      
      // @ts-ignore
      if (navigator.connection) {
        // @ts-ignore
        navigator.connection.removeEventListener('change', updateConnectionType);
      }
    };
  }, []);

  return { isOnline, connectionType };
};

// Hook for detecting reduced motion preference
export const useReducedMotion = () => {
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    setPrefersReducedMotion(mediaQuery.matches);

    const handleChange = (event: MediaQueryListEvent) => {
      setPrefersReducedMotion(event.matches);
    };

    mediaQuery.addEventListener('change', handleChange);

    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, []);

  return prefersReducedMotion;
};

// Hook for container queries (useful for responsive components)
export const useContainerQuery = (containerRef: React.RefObject<HTMLElement>) => {
  const [containerWidth, setContainerWidth] = useState(0);
  const [containerHeight, setContainerHeight] = useState(0);

  useEffect(() => {
    if (!containerRef.current) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setContainerWidth(entry.contentRect.width);
        setContainerHeight(entry.contentRect.height);
      }
    });

    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, [containerRef]);

  return { containerWidth, containerHeight };
};