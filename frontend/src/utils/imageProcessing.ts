/**
 * Image processing utilities for the raffle grid system
 */

export interface GridDimensions {
  rows: number;
  cols: number;
}

export interface ImageSegment {
  id: number;
  x: number;
  y: number;
  width: number;
  height: number;
  dataUrl?: string;
}

/**
 * Calculate optimal grid dimensions for a given number of boxes
 */
export const calculateGridDimensions = (totalBoxes: number): GridDimensions => {
  if (totalBoxes <= 0) return { rows: 1, cols: 1 };
  
  // Try to create a roughly square grid
  const sqrt = Math.sqrt(totalBoxes);
  let cols = Math.ceil(sqrt);
  let rows = Math.ceil(totalBoxes / cols);
  
  // Adjust for better aspect ratio (prefer wider than taller for better UX)
  const aspectRatio = rows / cols;
  if (aspectRatio > 0.8) {
    cols = Math.ceil(sqrt * 1.2);
    rows = Math.ceil(totalBoxes / cols);
  }
  
  // Ensure we don't have empty rows
  while (rows > 1 && (rows - 1) * cols >= totalBoxes) {
    rows--;
  }
  
  return { rows, cols };
};

/**
 * Divide an image into grid segments with enhanced processing
 */
export const divideImageIntoSegments = async (
  imageUrl: string,
  gridDimensions: GridDimensions,
  options: {
    quality?: number;
    format?: 'jpeg' | 'png' | 'webp';
    maxSegmentSize?: number;
  } = {}
): Promise<ImageSegment[]> => {
  const { quality = 0.8, format = 'jpeg', maxSegmentSize = 200 } = options;
  
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        
        if (!ctx) {
          reject(new Error('Could not get canvas context'));
          return;
        }
        
        const { rows, cols } = gridDimensions;
        const segmentWidth = Math.min(img.width / cols, maxSegmentSize);
        const segmentHeight = Math.min(img.height / rows, maxSegmentSize);
        
        canvas.width = segmentWidth;
        canvas.height = segmentHeight;
        
        const segments: ImageSegment[] = [];
        
        for (let row = 0; row < rows; row++) {
          for (let col = 0; col < cols; col++) {
            const id = row * cols + col + 1;
            
            // Clear canvas
            ctx.clearRect(0, 0, segmentWidth, segmentHeight);
            
            // Calculate source coordinates
            const sourceX = (col * img.width) / cols;
            const sourceY = (row * img.height) / rows;
            const sourceWidth = img.width / cols;
            const sourceHeight = img.height / rows;
            
            // Draw the segment with smooth scaling
            ctx.imageSmoothingEnabled = true;
            ctx.imageSmoothingQuality = 'high';
            
            ctx.drawImage(
              img,
              sourceX,
              sourceY,
              sourceWidth,
              sourceHeight,
              0,
              0,
              segmentWidth,
              segmentHeight
            );
            
            // Apply subtle enhancement
            const imageData = ctx.getImageData(0, 0, segmentWidth, segmentHeight);
            const data = imageData.data;
            
            // Slight contrast and brightness adjustment
            for (let i = 0; i < data.length; i += 4) {
              // Increase contrast slightly
              data[i] = Math.min(255, Math.max(0, (data[i] - 128) * 1.1 + 128));     // Red
              data[i + 1] = Math.min(255, Math.max(0, (data[i + 1] - 128) * 1.1 + 128)); // Green
              data[i + 2] = Math.min(255, Math.max(0, (data[i + 2] - 128) * 1.1 + 128)); // Blue
            }
            
            ctx.putImageData(imageData, 0, 0);
            
            segments.push({
              id,
              x: (col * 100) / cols, // percentage
              y: (row * 100) / rows, // percentage
              width: 100 / cols, // percentage
              height: 100 / rows, // percentage
              dataUrl: canvas.toDataURL(`image/${format}`, quality),
            });
          }
        }
        
        resolve(segments);
      } catch (error) {
        reject(error);
      }
    };
    
    img.onerror = () => {
      reject(new Error('Failed to load image'));
    };
    
    img.src = imageUrl;
  });
};

/**
 * Create a reveal mask for the image based on revealed segments
 */
export const createRevealMask = (
  revealedSegments: number[],
  gridDimensions: GridDimensions,
  imageWidth: number = 400,
  imageHeight: number = 400
): string => {
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');
  
  if (!ctx) return '';
  
  canvas.width = imageWidth;
  canvas.height = imageHeight;
  
  const { rows, cols } = gridDimensions;
  const segmentWidth = imageWidth / cols;
  const segmentHeight = imageHeight / rows;
  
  // Fill with black (hidden)
  ctx.fillStyle = 'black';
  ctx.fillRect(0, 0, imageWidth, imageHeight);
  
  // Reveal segments (white)
  ctx.fillStyle = 'white';
  
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const segmentId = row * cols + col + 1;
      
      if (revealedSegments.includes(segmentId)) {
        ctx.fillRect(
          col * segmentWidth,
          row * segmentHeight,
          segmentWidth,
          segmentHeight
        );
      }
    }
  }
  
  return canvas.toDataURL();
};

/**
 * Generate a blurred version of an image for preview
 */
export const generateBlurredImage = async (
  imageUrl: string,
  blurAmount: number = 10
): Promise<string> => {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        
        if (!ctx) {
          reject(new Error('Could not get canvas context'));
          return;
        }
        
        canvas.width = img.width;
        canvas.height = img.height;
        
        // Apply blur filter
        ctx.filter = `blur(${blurAmount}px)`;\n        ctx.drawImage(img, 0, 0);
        
        resolve(canvas.toDataURL('image/jpeg', 0.8));
      } catch (error) {
        reject(error);
      }
    };
    
    img.onerror = () => {
      reject(new Error('Failed to load image'));
    };
    
    img.src = imageUrl;
  });
};

/**
 * Optimize image for web display
 */
export const optimizeImageForWeb = async (
  file: File,
  maxWidth: number = 800,
  maxHeight: number = 800,
  quality: number = 0.8
): Promise<string> => {
  return new Promise((resolve, reject) => {
    const img = new Image();
    
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        
        if (!ctx) {
          reject(new Error('Could not get canvas context'));
          return;
        }
        
        // Calculate new dimensions
        let { width, height } = img;
        
        if (width > height) {
          if (width > maxWidth) {
            height = (height * maxWidth) / width;
            width = maxWidth;
          }
        } else {
          if (height > maxHeight) {
            width = (width * maxHeight) / height;
            height = maxHeight;
          }
        }
        
        canvas.width = width;
        canvas.height = height;
        
        // Draw and compress
        ctx.drawImage(img, 0, 0, width, height);
        
        resolve(canvas.toDataURL('image/jpeg', quality));
      } catch (error) {
        reject(error);
      }
    };
    
    img.onerror = () => {
      reject(new Error('Failed to load image'));
    };
    
    // Convert file to data URL
    const reader = new FileReader();
    reader.onload = (e) => {
      img.src = e.target?.result as string;
    };
    reader.onerror = () => {
      reject(new Error('Failed to read file'));
    };
    reader.readAsDataURL(file);
  });
};

/**
 * Check if an image URL is valid and accessible
 */
export const validateImageUrl = (url: string): Promise<boolean> => {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve(true);
    img.onerror = () => resolve(false);
    img.src = url;
  });
};

/**
 * Get image dimensions without loading the full image
 */
export const getImageDimensions = (url: string): Promise<{ width: number; height: number }> => {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      resolve({ width: img.width, height: img.height });
    };
    img.onerror = () => {
      reject(new Error('Failed to load image'));
    };
    img.src = url;
  });
};
/**
 * Cre
ate a progressive reveal effect for the image
 */
export const createProgressiveReveal = async (
  imageUrl: string,
  gridDimensions: GridDimensions,
  revealedBoxes: number[],
  options: {
    blurAmount?: number;
    overlayOpacity?: number;
    revealAnimation?: boolean;
  } = {}
): Promise<string> => {
  const { blurAmount = 20, overlayOpacity = 0.8, revealAnimation = true } = options;
  
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        
        if (!ctx) {
          reject(new Error('Could not get canvas context'));
          return;
        }
        
        canvas.width = img.width;
        canvas.height = img.height;
        
        // Draw the base image
        ctx.drawImage(img, 0, 0);
        
        // Create overlay for unrevealed areas
        const { rows, cols } = gridDimensions;
        const segmentWidth = img.width / cols;
        const segmentHeight = img.height / rows;
        
        for (let row = 0; row < rows; row++) {
          for (let col = 0; col < cols; col++) {
            const boxId = row * cols + col + 1;
            
            if (!revealedBoxes.includes(boxId)) {
              const x = col * segmentWidth;
              const y = row * segmentHeight;
              
              // Create a subtle overlay
              ctx.fillStyle = `rgba(0, 0, 0, ${overlayOpacity})`;
              ctx.fillRect(x, y, segmentWidth, segmentHeight);
              
              // Add box number
              ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
              ctx.font = `${Math.min(segmentWidth, segmentHeight) * 0.3}px Arial`;
              ctx.textAlign = 'center';
              ctx.textBaseline = 'middle';
              ctx.fillText(
                boxId.toString(),
                x + segmentWidth / 2,
                y + segmentHeight / 2
              );
            }
          }
        }
        
        resolve(canvas.toDataURL('image/jpeg', 0.9));
      } catch (error) {
        reject(error);
      }
    };
    
    img.onerror = () => {
      reject(new Error('Failed to load image'));
    };
    
    img.src = imageUrl;
  });
};

/**
 * Generate thumbnail with grid overlay
 */
export const generateThumbnailWithGrid = async (
  imageUrl: string,
  gridDimensions: GridDimensions,
  thumbnailSize: number = 300
): Promise<string> => {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        
        if (!ctx) {
          reject(new Error('Could not get canvas context'));
          return;
        }
        
        canvas.width = thumbnailSize;
        canvas.height = thumbnailSize;
        
        // Draw scaled image
        ctx.drawImage(img, 0, 0, thumbnailSize, thumbnailSize);
        
        // Draw grid overlay
        const { rows, cols } = gridDimensions;
        const cellWidth = thumbnailSize / cols;
        const cellHeight = thumbnailSize / rows;
        
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
        ctx.lineWidth = 1;
        
        // Draw vertical lines
        for (let col = 1; col < cols; col++) {
          const x = col * cellWidth;
          ctx.beginPath();
          ctx.moveTo(x, 0);
          ctx.lineTo(x, thumbnailSize);
          ctx.stroke();
        }
        
        // Draw horizontal lines
        for (let row = 1; row < rows; row++) {
          const y = row * cellHeight;
          ctx.beginPath();
          ctx.moveTo(0, y);
          ctx.lineTo(thumbnailSize, y);
          ctx.stroke();
        }
        
        resolve(canvas.toDataURL('image/jpeg', 0.8));
      } catch (error) {
        reject(error);
      }
    };
    
    img.onerror = () => {
      reject(new Error('Failed to load image'));
    };
    
    img.src = imageUrl;
  });
};

/**
 * Create animated reveal effect for newly purchased boxes
 */
export const createRevealAnimation = async (
  imageUrl: string,
  gridDimensions: GridDimensions,
  newlyRevealedBoxes: number[],
  duration: number = 1000
): Promise<string[]> => {
  const frames: string[] = [];
  const frameCount = 10;
  
  for (let frame = 0; frame <= frameCount; frame++) {
    const progress = frame / frameCount;
    const easeProgress = 1 - Math.pow(1 - progress, 3); // Ease-out cubic
    
    const frameDataUrl = await new Promise<string>((resolve, reject) => {
      const img = new Image();
      img.crossOrigin = 'anonymous';
      
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');
          
          if (!ctx) {
            reject(new Error('Could not get canvas context'));
            return;
          }
          
          canvas.width = img.width;
          canvas.height = img.height;
          
          // Draw base image
          ctx.drawImage(img, 0, 0);
          
          // Apply reveal effect to newly revealed boxes
          const { rows, cols } = gridDimensions;
          const segmentWidth = img.width / cols;
          const segmentHeight = img.height / rows;
          
          newlyRevealedBoxes.forEach(boxId => {
            const row = Math.floor((boxId - 1) / cols);
            const col = (boxId - 1) % cols;
            const x = col * segmentWidth;
            const y = row * segmentHeight;
            
            // Create reveal effect
            const overlayOpacity = 0.8 * (1 - easeProgress);
            ctx.fillStyle = `rgba(0, 0, 0, ${overlayOpacity})`;
            ctx.fillRect(x, y, segmentWidth, segmentHeight);
            
            // Add sparkle effect
            if (progress > 0.5) {
              const sparkleOpacity = (progress - 0.5) * 2;
              ctx.fillStyle = `rgba(255, 215, 0, ${sparkleOpacity})`;
              ctx.beginPath();
              ctx.arc(
                x + segmentWidth / 2,
                y + segmentHeight / 2,
                Math.min(segmentWidth, segmentHeight) * 0.1 * sparkleOpacity,
                0,
                2 * Math.PI
              );
              ctx.fill();
            }
          });
          
          resolve(canvas.toDataURL('image/jpeg', 0.9));
        } catch (error) {
          reject(error);
        }
      };
      
      img.onerror = () => {
        reject(new Error('Failed to load image'));
      };
      
      img.src = imageUrl;
    });
    
    frames.push(frameDataUrl);
  }
  
  return frames;
};

/**
 * Preload and cache image segments for better performance
 */
export class ImageSegmentCache {
  private cache = new Map<string, ImageSegment[]>();
  private loadingPromises = new Map<string, Promise<ImageSegment[]>>();
  
  async getSegments(
    imageUrl: string,
    gridDimensions: GridDimensions,
    options?: Parameters<typeof divideImageIntoSegments>[2]
  ): Promise<ImageSegment[]> {
    const cacheKey = `${imageUrl}-${gridDimensions.rows}x${gridDimensions.cols}`;
    
    // Return cached segments if available
    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey)!;
    }
    
    // Return existing loading promise if in progress
    if (this.loadingPromises.has(cacheKey)) {
      return this.loadingPromises.get(cacheKey)!;
    }
    
    // Start loading segments
    const loadingPromise = divideImageIntoSegments(imageUrl, gridDimensions, options);
    this.loadingPromises.set(cacheKey, loadingPromise);
    
    try {
      const segments = await loadingPromise;
      this.cache.set(cacheKey, segments);
      this.loadingPromises.delete(cacheKey);
      return segments;
    } catch (error) {
      this.loadingPromises.delete(cacheKey);
      throw error;
    }
  }
  
  preloadSegments(
    imageUrl: string,
    gridDimensions: GridDimensions,
    options?: Parameters<typeof divideImageIntoSegments>[2]
  ): void {
    // Start loading in background without waiting
    this.getSegments(imageUrl, gridDimensions, options).catch(console.error);
  }
  
  clearCache(): void {
    this.cache.clear();
    this.loadingPromises.clear();
  }
  
  getCacheSize(): number {
    return this.cache.size;
  }
}

// Global cache instance
export const imageSegmentCache = new ImageSegmentCache();

/**
 * Responsive grid dimensions based on screen size
 */
export const getResponsiveGridDimensions = (
  totalBoxes: number,
  screenWidth: number
): GridDimensions => {
  const baseGridDimensions = calculateGridDimensions(totalBoxes);
  
  // Adjust for mobile screens
  if (screenWidth < 640) { // sm breakpoint
    // Prefer more rows on mobile for better touch interaction
    const sqrt = Math.sqrt(totalBoxes);
    let rows = Math.ceil(sqrt * 1.2);
    let cols = Math.ceil(totalBoxes / rows);
    
    // Ensure minimum cell size for touch
    const maxCols = Math.floor(screenWidth / 60); // 60px minimum cell width
    if (cols > maxCols) {
      cols = maxCols;
      rows = Math.ceil(totalBoxes / cols);
    }
    
    return { rows, cols };
  }
  
  // Adjust for tablet screens
  if (screenWidth < 1024) { // lg breakpoint
    return baseGridDimensions;
  }
  
  // Desktop - prefer wider grids
  const { rows, cols } = baseGridDimensions;
  if (rows / cols > 0.7) {
    const newCols = Math.ceil(cols * 1.3);
    const newRows = Math.ceil(totalBoxes / newCols);
    return { rows: newRows, cols: newCols };
  }
  
  return baseGridDimensions;
};

/**
 * Calculate optimal image size for grid display
 */
export const calculateOptimalImageSize = (
  gridDimensions: GridDimensions,
  containerWidth: number,
  containerHeight: number
): { width: number; height: number } => {
  const { rows, cols } = gridDimensions;
  const aspectRatio = cols / rows;
  
  let width = containerWidth;
  let height = containerWidth / aspectRatio;
  
  if (height > containerHeight) {
    height = containerHeight;
    width = containerHeight * aspectRatio;
  }
  
  // Ensure minimum size for visibility
  const minSize = 400;
  if (width < minSize) {
    width = minSize;
    height = minSize / aspectRatio;
  }
  
  return { width: Math.round(width), height: Math.round(height) };
};