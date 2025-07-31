import React, { Suspense, ComponentType, LazyExoticComponent } from 'react';
import { ErrorBoundary } from 'react-error-boundary';

// Loading component for lazy-loaded components
const LoadingSpinner: React.FC<{ message?: string }> = ({ message = 'Loading...' }) => (
  <div className="flex items-center justify-center min-h-[200px]">
    <div className="flex flex-col items-center space-y-4">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      <p className="text-sm text-gray-600">{message}</p>
    </div>
  </div>
);

// Error fallback component
const ErrorFallback: React.FC<{ error: Error; resetErrorBoundary: () => void }> = ({
  error,
  resetErrorBoundary,
}) => (
  <div className="flex items-center justify-center min-h-[200px]">
    <div className="text-center space-y-4">
      <div className="text-red-600">
        <svg className="w-12 h-12 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
      </div>
      <h3 className="text-lg font-medium text-gray-900">Something went wrong</h3>
      <p className="text-sm text-gray-600">{error.message}</p>
      <button
        onClick={resetErrorBoundary}
        className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
      >
        Try again
      </button>
    </div>
  </div>
);

// Higher-order component for lazy loading with error boundary
export const withLazyLoading = <P extends object>(
  LazyComponent: LazyExoticComponent<ComponentType<P>>,
  loadingMessage?: string,
  fallbackComponent?: ComponentType<{ error: Error; resetErrorBoundary: () => void }>
) => {
  const WrappedComponent: React.FC<P> = (props) => (
    <ErrorBoundary
      FallbackComponent={fallbackComponent || ErrorFallback}
      onReset={() => window.location.reload()}
    >
      <Suspense fallback={<LoadingSpinner message={loadingMessage} />}>
        <LazyComponent {...props} />
      </Suspense>
    </ErrorBoundary>
  );

  WrappedComponent.displayName = `withLazyLoading(${LazyComponent.displayName || 'Component'})`;
  return WrappedComponent;
};

// Preload utility for critical routes
export const preloadComponent = (componentImport: () => Promise<any>) => {
  // Start loading the component
  const componentPromise = componentImport();
  
  // Return a function that can be called to get the preloaded component
  return () => componentPromise;
};

// Route-based code splitting utilities
export const createLazyRoute = (
  importFn: () => Promise<{ default: ComponentType<any> }>,
  loadingMessage?: string
) => {
  const LazyComponent = React.lazy(importFn);
  return withLazyLoading(LazyComponent, loadingMessage);
};

// Component-based code splitting for heavy components
export const createLazyComponent = <P extends object>(
  importFn: () => Promise<{ default: ComponentType<P> }>,
  options: {
    loadingMessage?: string;
    fallback?: ComponentType<{ error: Error; resetErrorBoundary: () => void }>;
    preload?: boolean;
  } = {}
) => {
  const LazyComponent = React.lazy(importFn);
  
  // Preload if requested
  if (options.preload) {
    // Preload after a short delay to not block initial render
    setTimeout(() => {
      importFn().catch(console.error);
    }, 100);
  }
  
  return withLazyLoading(LazyComponent, options.loadingMessage, options.fallback);
};

// Intersection Observer based lazy loading for components
export const createIntersectionLazyComponent = <P extends object>(
  importFn: () => Promise<{ default: ComponentType<P> }>,
  options: {
    rootMargin?: string;
    threshold?: number;
    loadingMessage?: string;
  } = {}
) => {
  const LazyComponent = React.lazy(importFn);
  
  const IntersectionLazyComponent: React.FC<P> = (props) => {
    const [isVisible, setIsVisible] = React.useState(false);
    const ref = React.useRef<HTMLDivElement>(null);
    
    React.useEffect(() => {
      const observer = new IntersectionObserver(
        ([entry]) => {
          if (entry.isIntersecting) {
            setIsVisible(true);
            observer.disconnect();
          }
        },
        {
          rootMargin: options.rootMargin || '50px',
          threshold: options.threshold || 0.1,
        }
      );
      
      if (ref.current) {
        observer.observe(ref.current);
      }
      
      return () => observer.disconnect();
    }, []);
    
    return (
      <div ref={ref}>
        {isVisible ? (
          <ErrorBoundary FallbackComponent={ErrorFallback}>
            <Suspense fallback={<LoadingSpinner message={options.loadingMessage} />}>
              <LazyComponent {...props} />
            </Suspense>
          </ErrorBoundary>
        ) : (
          <LoadingSpinner message="Loading component..." />
        )}
      </div>
    );
  };
  
  return IntersectionLazyComponent;
};

// Bundle splitting utilities
export const createChunkName = (componentName: string, feature?: string) => {
  return feature ? `${feature}-${componentName}` : componentName;
};

// Dynamic import with retry logic
export const dynamicImportWithRetry = (
  importFn: () => Promise<any>,
  retries = 3,
  delay = 1000
): Promise<any> => {
  return new Promise((resolve, reject) => {
    const attemptImport = (attempt: number) => {
      importFn()
        .then(resolve)
        .catch((error) => {
          if (attempt < retries) {
            console.warn(`Import failed, retrying... (${attempt + 1}/${retries})`);
            setTimeout(() => attemptImport(attempt + 1), delay);
          } else {
            reject(error);
          }
        });
    };
    
    attemptImport(0);
  });
};

// Prefetch utilities for better UX
export const prefetchRoute = (routeImport: () => Promise<any>) => {
  // Use requestIdleCallback if available, otherwise use setTimeout
  if ('requestIdleCallback' in window) {
    requestIdleCallback(() => {
      routeImport().catch(console.error);
    });
  } else {
    setTimeout(() => {
      routeImport().catch(console.error);
    }, 2000);
  }
};

// Link component with prefetching
export const PrefetchLink: React.FC<{
  to: string;
  prefetch?: () => Promise<any>;
  children: React.ReactNode;
  className?: string;
  onMouseEnter?: () => void;
}> = ({ to, prefetch, children, className, onMouseEnter, ...props }) => {
  const handleMouseEnter = React.useCallback(() => {
    if (prefetch) {
      prefetch().catch(console.error);
    }
    onMouseEnter?.();
  }, [prefetch, onMouseEnter]);
  
  return (
    <a
      href={to}
      className={className}
      onMouseEnter={handleMouseEnter}
      {...props}
    >
      {children}
    </a>
  );
};

// Resource hints utilities
export const addResourceHints = (resources: Array<{ href: string; as?: string; type?: string }>) => {
  resources.forEach(({ href, as, type }) => {
    const link = document.createElement('link');
    link.rel = 'prefetch';
    link.href = href;
    if (as) link.setAttribute('as', as);
    if (type) link.type = type;
    document.head.appendChild(link);
  });
};

// Critical resource preloading
export const preloadCriticalResources = () => {
  // Preload critical fonts
  const criticalFonts = [
    '/fonts/inter-var.woff2',
    '/fonts/inter-var-italic.woff2',
  ];
  
  criticalFonts.forEach(font => {
    const link = document.createElement('link');
    link.rel = 'preload';
    link.href = font;
    link.as = 'font';
    link.type = 'font/woff2';
    link.crossOrigin = 'anonymous';
    document.head.appendChild(link);
  });
  
  // Preload critical images
  const criticalImages = [
    '/images/logo.webp',
    '/images/hero-bg.webp',
  ];
  
  criticalImages.forEach(image => {
    const link = document.createElement('link');
    link.rel = 'preload';
    link.href = image;
    link.as = 'image';
    document.head.appendChild(link);
  });
};

// Module federation utilities (for micro-frontends)
export const loadRemoteModule = async (
  remoteUrl: string,
  moduleName: string,
  retries = 3
): Promise<any> => {
  return dynamicImportWithRetry(
    () => import(/* webpackIgnore: true */ `${remoteUrl}/${moduleName}`),
    retries
  );
};

// Performance monitoring for code splitting
export const measureChunkLoad = (chunkName: string) => {
  const startTime = performance.now();
  
  return {
    end: () => {
      const loadTime = performance.now() - startTime;
      console.log(`Chunk "${chunkName}" loaded in ${loadTime.toFixed(2)}ms`);
      
      // Report to analytics if available
      if (window.gtag) {
        window.gtag('event', 'chunk_load_time', {
          chunk_name: chunkName,
          load_time: Math.round(loadTime),
        });
      }
      
      return loadTime;
    },
  };
};

// Lazy loading with performance tracking
export const createTrackedLazyComponent = <P extends object>(
  importFn: () => Promise<{ default: ComponentType<P> }>,
  componentName: string,
  options: {
    loadingMessage?: string;
    preload?: boolean;
  } = {}
) => {
  const trackedImportFn = () => {
    const measurement = measureChunkLoad(componentName);
    return importFn().then(module => {
      measurement.end();
      return module;
    });
  };
  
  return createLazyComponent(trackedImportFn, options);
};

export default {
  withLazyLoading,
  createLazyRoute,
  createLazyComponent,
  createIntersectionLazyComponent,
  preloadComponent,
  prefetchRoute,
  PrefetchLink,
  addResourceHints,
  preloadCriticalResources,
  loadRemoteModule,
  measureChunkLoad,
  createTrackedLazyComponent,
};