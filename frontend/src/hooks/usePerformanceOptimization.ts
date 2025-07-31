import { useEffect, useCallback, useRef } from 'react';
import { usePerformanceMonitoring } from '../services/performanceService';
import { getGlobalLazyLoader } from '../utils/imageOptimization';

export interface PerformanceOptimizationOptions {
  enableImageLazyLoading?: boolean;
  enableIntersectionObserver?: boolean;
  enableMemoryMonitoring?: boolean;
  enableNetworkOptimization?: boolean;
  throttleScrollEvents?: boolean;
  throttleResizeEvents?: boolean;
  prefetchOnHover?: boolean;
  optimizeAnimations?: boolean;
}

export const usePerformanceOptimization = (
  options: PerformanceOptimizationOptions = {}
) => {
  const {
    enableImageLazyLoading = true,
    enableIntersectionObserver = true,
    enableMemoryMonitoring = true,
    enableNetworkOptimization = true,
    throttleScrollEvents = true,
    throttleResizeEvents = true,
    prefetchOnHover = true,
    optimizeAnimations = true,
  } = options;

  const { measureApiCall, measureUserInteraction, startMeasure, endMeasure } = usePerformanceMonitoring();
  const intersectionObserverRef = useRef<IntersectionObserver | null>(null);
  const scrollThrottleRef = useRef<NodeJS.Timeout | null>(null);
  const resizeThrottleRef = useRef<NodeJS.Timeout | null>(null);

  // Initialize image lazy loading
  useEffect(() => {
    if (enableImageLazyLoading) {
      const lazyLoader = getGlobalLazyLoader();
      
      // Observe all images with data-src attribute
      const images = document.querySelectorAll('img[data-src]');
      images.forEach(img => lazyLoader.observe(img as HTMLImageElement));
      
      return () => {
        images.forEach(img => lazyLoader.unobserve(img as HTMLImageElement));
      };
    }
  }, [enableImageLazyLoading]);

  // Initialize intersection observer for component visibility
  useEffect(() => {
    if (enableIntersectionObserver && 'IntersectionObserver' in window) {
      intersectionObserverRef.current = new IntersectionObserver(
        (entries) => {
          entries.forEach(entry => {
            const element = entry.target as HTMLElement;
            
            if (entry.isIntersecting) {
              element.classList.add('in-viewport');
              element.dispatchEvent(new CustomEvent('enterViewport'));
            } else {
              element.classList.remove('in-viewport');
              element.dispatchEvent(new CustomEvent('exitViewport'));
            }
          });
        },
        {
          rootMargin: '50px',
          threshold: 0.1,
        }
      );

      // Observe elements with data-observe attribute
      const observableElements = document.querySelectorAll('[data-observe]');
      observableElements.forEach(el => {
        intersectionObserverRef.current?.observe(el);
      });

      return () => {
        intersectionObserverRef.current?.disconnect();
      };
    }
  }, [enableIntersectionObserver]);

  // Memory monitoring
  useEffect(() => {
    if (enableMemoryMonitoring && 'memory' in performance) {
      const monitorMemory = () => {
        const memory = (performance as any).memory;
        const memoryUsage = memory.usedJSHeapSize / memory.jsHeapSizeLimit;
        
        if (memoryUsage > 0.9) {
          console.warn('High memory usage detected:', memoryUsage);
          
          // Trigger garbage collection if available
          if ('gc' in window) {
            (window as any).gc();
          }
          
          // Clear caches if memory is critically high
          if (memoryUsage > 0.95) {
            clearCaches();
          }
        }
      };

      const interval = setInterval(monitorMemory, 10000); // Every 10 seconds
      return () => clearInterval(interval);
    }
  }, [enableMemoryMonitoring]);

  // Network optimization
  useEffect(() => {
    if (enableNetworkOptimization && 'connection' in navigator) {
      const connection = (navigator as any).connection;
      
      const optimizeForConnection = () => {
        const effectiveType = connection.effectiveType;
        
        // Adjust quality based on connection
        document.documentElement.setAttribute('data-connection', effectiveType);
        
        if (effectiveType === 'slow-2g' || effectiveType === '2g') {
          // Disable non-essential features for slow connections
          document.documentElement.classList.add('low-bandwidth');
        } else {
          document.documentElement.classList.remove('low-bandwidth');
        }
      };

      optimizeForConnection();
      connection.addEventListener('change', optimizeForConnection);
      
      return () => {
        connection.removeEventListener('change', optimizeForConnection);
      };
    }
  }, [enableNetworkOptimization]);

  // Animation optimization
  useEffect(() => {
    if (optimizeAnimations) {
      // Reduce animations for users who prefer reduced motion
      const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
      
      const handleMotionPreference = (e: MediaQueryListEvent) => {
        if (e.matches) {
          document.documentElement.classList.add('reduce-motion');
        } else {
          document.documentElement.classList.remove('reduce-motion');
        }
      };

      handleMotionPreference(mediaQuery as any);
      mediaQuery.addEventListener('change', handleMotionPreference);
      
      return () => {
        mediaQuery.removeEventListener('change', handleMotionPreference);
      };
    }
  }, [optimizeAnimations]);

  // Throttled scroll handler
  const createThrottledScrollHandler = useCallback((handler: () => void, delay = 16) => {
    if (!throttleScrollEvents) return handler;
    
    return () => {
      if (scrollThrottleRef.current) {
        clearTimeout(scrollThrottleRef.current);
      }
      
      scrollThrottleRef.current = setTimeout(handler, delay);
    };
  }, [throttleScrollEvents]);

  // Throttled resize handler
  const createThrottledResizeHandler = useCallback((handler: () => void, delay = 100) => {
    if (!throttleResizeEvents) return handler;
    
    return () => {
      if (resizeThrottleRef.current) {
        clearTimeout(resizeThrottleRef.current);
      }
      
      resizeThrottleRef.current = setTimeout(handler, delay);
    };
  }, [throttleResizeEvents]);

  // Prefetch on hover
  const createPrefetchHandler = useCallback((importFn: () => Promise<any>) => {
    if (!prefetchOnHover) return () => {};
    
    return () => {
      // Use requestIdleCallback if available
      if ('requestIdleCallback' in window) {
        requestIdleCallback(() => {
          importFn().catch(console.error);
        });
      } else {
        setTimeout(() => {
          importFn().catch(console.error);
        }, 100);
      }
    };
  }, [prefetchOnHover]);

  // Optimized API call wrapper
  const optimizedApiCall = useCallback(<T>(
    apiCall: () => Promise<T>,
    endpoint: string,
    options: {
      cache?: boolean;
      timeout?: number;
      retries?: number;
    } = {}
  ): Promise<T> => {
    const { cache = true, timeout = 10000, retries = 3 } = options;
    
    return measureApiCall(async () => {
      // Check cache first if enabled
      if (cache && 'caches' in window) {
        try {
          const cachedResponse = await caches.match(endpoint);
          if (cachedResponse) {
            return cachedResponse.json();
          }
        } catch (error) {
          console.warn('Cache check failed:', error);
        }
      }
      
      // Implement timeout and retries
      let lastError: Error | null = null;
      
      for (let attempt = 0; attempt < retries; attempt++) {
        try {
          const timeoutPromise = new Promise<never>((_, reject) => {
            setTimeout(() => reject(new Error('Request timeout')), timeout);
          });
          
          const result = await Promise.race([apiCall(), timeoutPromise]);
          
          // Cache successful response
          if (cache && 'caches' in window) {
            try {
              const cache = await caches.open('api-cache');
              await cache.put(endpoint, new Response(JSON.stringify(result)));
            } catch (error) {
              console.warn('Failed to cache response:', error);
            }
          }
          
          return result;
        } catch (error) {
          lastError = error as Error;
          
          if (attempt < retries - 1) {
            // Exponential backoff
            await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000));
          }
        }
      }
      
      throw lastError;
    }, endpoint);
  }, [measureApiCall]);

  // Optimized user interaction wrapper
  const optimizedUserInteraction = useCallback((
    interaction: () => void,
    interactionType: string,
    options: {
      debounce?: number;
      throttle?: number;
    } = {}
  ) => {
    const { debounce, throttle } = options;
    
    let timeoutId: NodeJS.Timeout | null = null;
    let lastExecution = 0;
    
    return () => {
      const now = Date.now();
      
      // Throttling
      if (throttle && now - lastExecution < throttle) {
        return;
      }
      
      // Debouncing
      if (debounce) {
        if (timeoutId) {
          clearTimeout(timeoutId);
        }
        
        timeoutId = setTimeout(() => {
          measureUserInteraction(interaction, interactionType);
          lastExecution = Date.now();
        }, debounce);
      } else {
        measureUserInteraction(interaction, interactionType);
        lastExecution = now;
      }
    };
  }, [measureUserInteraction]);

  // Component performance measurement
  const measureComponentRender = useCallback((componentName: string) => {
    startMeasure(`${componentName}-render`);
    
    return () => {
      endMeasure(`${componentName}-render`);
    };
  }, [startMeasure, endMeasure]);

  // Cleanup function
  const cleanup = useCallback(() => {
    if (scrollThrottleRef.current) {
      clearTimeout(scrollThrottleRef.current);
    }
    
    if (resizeThrottleRef.current) {
      clearTimeout(resizeThrottleRef.current);
    }
    
    if (intersectionObserverRef.current) {
      intersectionObserverRef.current.disconnect();
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return cleanup;
  }, [cleanup]);

  return {
    optimizedApiCall,
    optimizedUserInteraction,
    measureComponentRender,
    createThrottledScrollHandler,
    createThrottledResizeHandler,
    createPrefetchHandler,
    cleanup,
  };
};

// Helper function to clear caches when memory is low
const clearCaches = async () => {
  try {
    if ('caches' in window) {
      const cacheNames = await caches.keys();
      await Promise.all(
        cacheNames.map(cacheName => {
          if (cacheName.includes('dynamic') || cacheName.includes('api')) {
            return caches.delete(cacheName);
          }
        })
      );
    }
    
    // Clear other caches
    if ('localStorage' in window) {
      // Clear non-essential localStorage items
      const keysToRemove = [];
      for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key && (key.includes('cache') || key.includes('temp'))) {
          keysToRemove.push(key);
        }
      }
      keysToRemove.forEach(key => localStorage.removeItem(key));
    }
    
    console.log('Caches cleared due to high memory usage');
  } catch (error) {
    console.error('Failed to clear caches:', error);
  }
};

export default usePerformanceOptimization;