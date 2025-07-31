// Image optimization utilities for better performance

export interface ImageOptimizationOptions {
  quality?: number;
  format?: 'webp' | 'avif' | 'jpeg' | 'png';
  width?: number;
  height?: number;
  fit?: 'cover' | 'contain' | 'fill' | 'inside' | 'outside';
  lazy?: boolean;
  placeholder?: 'blur' | 'empty' | string;
  sizes?: string;
  priority?: boolean;
}

export interface ResponsiveImageConfig {
  src: string;
  alt: string;
  sizes: Array<{
    width: number;
    height?: number;
    media?: string;
  }>;
  options?: ImageOptimizationOptions;
}

// CDN URL builders
export const buildCloudinaryUrl = (
  publicId: string,
  options: ImageOptimizationOptions = {}
): string => {
  const baseUrl = process.env.REACT_APP_CLOUDINARY_BASE_URL || '';
  const cloudName = process.env.REACT_APP_CLOUDINARY_CLOUD_NAME || '';
  
  if (!baseUrl || !cloudName) {
    console.warn('Cloudinary configuration missing');
    return publicId;
  }
  
  const transformations: string[] = [];
  
  if (options.width) transformations.push(`w_${options.width}`);
  if (options.height) transformations.push(`h_${options.height}`);
  if (options.quality) transformations.push(`q_${options.quality}`);
  if (options.format) transformations.push(`f_${options.format}`);
  if (options.fit) transformations.push(`c_${options.fit}`);
  
  // Add automatic format and quality optimization
  transformations.push('f_auto', 'q_auto');
  
  const transformString = transformations.join(',');
  return `${baseUrl}/${cloudName}/image/upload/${transformString}/${publicId}`;
};

export const buildImageKitUrl = (
  path: string,
  options: ImageOptimizationOptions = {}
): string => {
  const baseUrl = process.env.REACT_APP_IMAGEKIT_BASE_URL || '';
  
  if (!baseUrl) {
    console.warn('ImageKit configuration missing');
    return path;
  }
  
  const params = new URLSearchParams();
  
  if (options.width) params.set('tr', `w-${options.width}`);
  if (options.height) params.append('tr', `h-${options.height}`);
  if (options.quality) params.append('tr', `q-${options.quality}`);
  if (options.format) params.append('tr', `f-${options.format}`);
  if (options.fit) params.append('tr', `c-${options.fit}`);
  
  // Add automatic optimization
  params.append('tr', 'f-auto,q-auto');
  
  const queryString = params.toString();
  return `${baseUrl}${path}${queryString ? `?${queryString}` : ''}`;
};

// Generate srcset for responsive images
export const generateSrcSet = (
  baseSrc: string,
  widths: number[],
  urlBuilder: (src: string, options: ImageOptimizationOptions) => string = (src) => src
): string => {
  return widths
    .map(width => `${urlBuilder(baseSrc, { width })} ${width}w`)
    .join(', ');
};

// Generate sizes attribute
export const generateSizes = (breakpoints: Array<{ media: string; size: string }>): string => {
  return breakpoints
    .map(({ media, size }) => `${media} ${size}`)
    .join(', ');
};

// Lazy loading with Intersection Observer
export class LazyImageLoader {
  private observer: IntersectionObserver;
  private images: Set<HTMLImageElement> = new Set();
  
  constructor(options: IntersectionObserverInit = {}) {
    this.observer = new IntersectionObserver(
      this.handleIntersection.bind(this),
      {
        rootMargin: '50px',
        threshold: 0.1,
        ...options,
      }
    );
  }
  
  private handleIntersection(entries: IntersectionObserverEntry[]) {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const img = entry.target as HTMLImageElement;
        this.loadImage(img);
        this.observer.unobserve(img);
        this.images.delete(img);
      }
    });
  }
  
  private loadImage(img: HTMLImageElement) {
    const src = img.dataset.src;
    const srcset = img.dataset.srcset;
    
    if (src) {
      img.src = src;
      img.removeAttribute('data-src');
    }
    
    if (srcset) {
      img.srcset = srcset;
      img.removeAttribute('data-srcset');
    }
    
    img.classList.remove('lazy-loading');
    img.classList.add('lazy-loaded');
  }
  
  public observe(img: HTMLImageElement) {
    this.images.add(img);
    this.observer.observe(img);
  }
  
  public unobserve(img: HTMLImageElement) {
    this.images.delete(img);
    this.observer.unobserve(img);
  }
  
  public disconnect() {
    this.observer.disconnect();
    this.images.clear();
  }
}

// Global lazy image loader instance
let globalLazyLoader: LazyImageLoader | null = null;

export const getGlobalLazyLoader = (): LazyImageLoader => {
  if (!globalLazyLoader) {
    globalLazyLoader = new LazyImageLoader();
  }
  return globalLazyLoader;
};

// Image preloading utilities
export const preloadImage = (src: string): Promise<void> => {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve();
    img.onerror = reject;
    img.src = src;
  });
};

export const preloadImages = (sources: string[]): Promise<void[]> => {
  return Promise.all(sources.map(preloadImage));
};

// Progressive image loading
export const createProgressiveImage = (
  lowQualitySrc: string,
  highQualitySrc: string,
  onLoad?: () => void
): HTMLImageElement => {
  const img = new Image();
  
  // Load low quality first
  img.src = lowQualitySrc;
  img.classList.add('progressive-image', 'low-quality');
  
  // Load high quality in background
  const highQualityImg = new Image();
  highQualityImg.onload = () => {
    img.src = highQualitySrc;
    img.classList.remove('low-quality');
    img.classList.add('high-quality');
    onLoad?.();
  };
  highQualityImg.src = highQualitySrc;
  
  return img;
};

// WebP support detection
export const supportsWebP = (): Promise<boolean> => {
  return new Promise(resolve => {
    const webP = new Image();
    webP.onload = webP.onerror = () => {
      resolve(webP.height === 2);
    };
    webP.src = 'data:image/webp;base64,UklGRjoAAABXRUJQVlA4IC4AAACyAgCdASoCAAIALmk0mk0iIiIiIgBoSygABc6WWgAA/veff/0PP8bA//LwYAAA';
  });
};

// AVIF support detection
export const supportsAVIF = (): Promise<boolean> => {
  return new Promise(resolve => {
    const avif = new Image();
    avif.onload = avif.onerror = () => {
      resolve(avif.height === 2);
    };
    avif.src = 'data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgABogQEAwgMg8f8D///8WfhwB8+ErK42A=';
  });
};

// Format selection based on browser support
export const selectOptimalFormat = async (
  formats: Array<'webp' | 'avif' | 'jpeg' | 'png'> = ['avif', 'webp', 'jpeg']
): Promise<'webp' | 'avif' | 'jpeg' | 'png'> => {
  for (const format of formats) {
    if (format === 'avif' && await supportsAVIF()) return 'avif';
    if (format === 'webp' && await supportsWebP()) return 'webp';
    if (format === 'jpeg' || format === 'png') return format;
  }
  return 'jpeg'; // fallback
};

// Image compression utilities
export const compressImage = (
  file: File,
  options: {
    maxWidth?: number;
    maxHeight?: number;
    quality?: number;
    format?: string;
  } = {}
): Promise<Blob> => {
  return new Promise((resolve, reject) => {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    const img = new Image();
    
    img.onload = () => {
      const { maxWidth = 1920, maxHeight = 1080, quality = 0.8, format = 'image/jpeg' } = options;
      
      // Calculate new dimensions
      let { width, height } = img;
      
      if (width > maxWidth) {
        height = (height * maxWidth) / width;
        width = maxWidth;
      }
      
      if (height > maxHeight) {
        width = (width * maxHeight) / height;
        height = maxHeight;
      }
      
      canvas.width = width;
      canvas.height = height;
      
      // Draw and compress
      ctx?.drawImage(img, 0, 0, width, height);
      
      canvas.toBlob(
        (blob) => {
          if (blob) {
            resolve(blob);
          } else {
            reject(new Error('Failed to compress image'));
          }
        },
        format,
        quality
      );
    };
    
    img.onerror = reject;
    img.src = URL.createObjectURL(file);
  });
};

// Responsive image component utilities
export const getResponsiveImageProps = (
  config: ResponsiveImageConfig,
  urlBuilder: (src: string, options: ImageOptimizationOptions) => string = (src) => src
) => {
  const { src, alt, sizes, options = {} } = config;
  
  // Generate srcset
  const widths = sizes.map(size => size.width);
  const srcset = generateSrcSet(src, widths, urlBuilder);
  
  // Generate sizes attribute
  const sizesAttr = sizes
    .map(size => {
      const media = size.media || `(max-width: ${size.width}px)`;
      return `${media} ${size.width}px`;
    })
    .join(', ');
  
  // Default src (largest size)
  const defaultSrc = urlBuilder(src, { width: Math.max(...widths), ...options });
  
  return {
    src: defaultSrc,
    srcSet: srcset,
    sizes: sizesAttr,
    alt,
    loading: options.lazy ? 'lazy' : options.priority ? 'eager' : 'lazy',
    decoding: 'async',
  };
};

// Image placeholder utilities
export const generateBlurDataURL = (width: number, height: number): string => {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  
  const ctx = canvas.getContext('2d');
  if (!ctx) return '';
  
  // Create a simple gradient placeholder
  const gradient = ctx.createLinearGradient(0, 0, width, height);
  gradient.addColorStop(0, '#f3f4f6');
  gradient.addColorStop(1, '#e5e7eb');
  
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, width, height);
  
  return canvas.toDataURL('image/jpeg', 0.1);
};

// Performance monitoring for images
export const monitorImagePerformance = () => {
  if (!window.PerformanceObserver) return;
  
  const observer = new PerformanceObserver((list) => {
    const entries = list.getEntries();
    
    entries.forEach((entry) => {
      if (entry.entryType === 'resource' && entry.name.match(/\.(jpg|jpeg|png|gif|webp|avif)$/i)) {
        const resource = entry as PerformanceResourceTiming;
        
        console.log(`Image loaded: ${resource.name}`);
        console.log(`Load time: ${resource.duration.toFixed(2)}ms`);
        console.log(`Transfer size: ${resource.transferSize} bytes`);
        
        // Report slow images
        if (resource.duration > 1000) {
          console.warn(`Slow image detected: ${resource.name} took ${resource.duration.toFixed(2)}ms`);
        }
        
        // Report large images
        if (resource.transferSize > 500000) { // 500KB
          console.warn(`Large image detected: ${resource.name} is ${(resource.transferSize / 1024).toFixed(2)}KB`);
        }
      }
    });
  });
  
  observer.observe({ entryTypes: ['resource'] });
  
  return observer;
};

// Image optimization recommendations
export const analyzeImagePerformance = () => {
  const images = Array.from(document.querySelectorAll('img'));
  const analysis = {
    totalImages: images.length,
    lazyImages: 0,
    missingAlt: 0,
    oversizedImages: 0,
    unoptimizedFormats: 0,
    recommendations: [] as string[],
  };
  
  images.forEach(img => {
    // Check for lazy loading
    if (img.loading === 'lazy' || img.dataset.src) {
      analysis.lazyImages++;
    }
    
    // Check for missing alt text
    if (!img.alt) {
      analysis.missingAlt++;
    }
    
    // Check for oversized images
    if (img.naturalWidth > img.clientWidth * 2) {
      analysis.oversizedImages++;
    }
    
    // Check for unoptimized formats
    if (img.src.match(/\.(jpg|jpeg|png)$/i) && !img.src.includes('f_auto')) {
      analysis.unoptimizedFormats++;
    }
  });
  
  // Generate recommendations
  if (analysis.lazyImages < analysis.totalImages * 0.8) {
    analysis.recommendations.push('Consider implementing lazy loading for more images');
  }
  
  if (analysis.missingAlt > 0) {
    analysis.recommendations.push(`Add alt text to ${analysis.missingAlt} images for accessibility`);
  }
  
  if (analysis.oversizedImages > 0) {
    analysis.recommendations.push(`Resize ${analysis.oversizedImages} oversized images`);
  }
  
  if (analysis.unoptimizedFormats > 0) {
    analysis.recommendations.push(`Use modern formats (WebP/AVIF) for ${analysis.unoptimizedFormats} images`);
  }
  
  return analysis;
};

export default {
  buildCloudinaryUrl,
  buildImageKitUrl,
  generateSrcSet,
  generateSizes,
  LazyImageLoader,
  getGlobalLazyLoader,
  preloadImage,
  preloadImages,
  createProgressiveImage,
  supportsWebP,
  supportsAVIF,
  selectOptimalFormat,
  compressImage,
  getResponsiveImageProps,
  generateBlurDataURL,
  monitorImagePerformance,
  analyzeImagePerformance,
};