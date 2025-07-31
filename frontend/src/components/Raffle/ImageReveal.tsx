import React, { useState, useEffect, useMemo } from 'react';
import {
  EyeIcon,
  EyeSlashIcon,
  Squares2X2Icon,
  Square3Stack3DIcon,
} from '@heroicons/react/24/outline';

interface ImageRevealProps {
  imageUrl: string;
  revealProgress: number; // 0-100
  gridDimensions: {
    rows: number;
    cols: number;
  };
  soldBoxes: number[];
  showGrid?: boolean;
}

interface ImageSegment {
  id: number;
  x: number;
  y: number;
  width: number;
  height: number;
  isRevealed: boolean;
}

const ImageReveal: React.FC<ImageRevealProps> = ({
  imageUrl,
  revealProgress,
  gridDimensions,
  soldBoxes,
  showGrid: initialShowGrid = true,
}) => {
  const [imageLoaded, setImageLoaded] = useState(false);
  const [showGrid, setShowGrid] = useState(initialShowGrid);
  const [showRevealedOnly, setShowRevealedOnly] = useState(false);

  // Calculate image segments based on grid dimensions
  const imageSegments = useMemo((): ImageSegment[] => {
    const segments: ImageSegment[] = [];
    const { rows, cols } = gridDimensions;
    const segmentWidth = 100 / cols;
    const segmentHeight = 100 / rows;

    for (let row = 0; row < rows; row++) {
      for (let col = 0; col < cols; col++) {
        const id = row * cols + col + 1;
        const isRevealed = soldBoxes.includes(id);
        
        segments.push({
          id,
          x: col * segmentWidth,
          y: row * segmentHeight,
          width: segmentWidth,
          height: segmentHeight,
          isRevealed,
        });
      }
    }

    return segments;
  }, [gridDimensions, soldBoxes]);

  // Calculate revealed segments count
  const revealedCount = imageSegments.filter(segment => segment.isRevealed).length;
  const totalSegments = imageSegments.length;

  // Handle image load
  const handleImageLoad = () => {
    setImageLoaded(true);
  };

  // Create reveal mask based on sold boxes
  const createRevealMask = () => {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) return '';

    canvas.width = 400;
    canvas.height = 400;

    // Fill with black (hidden)
    ctx.fillStyle = 'black';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Reveal sold segments (white)
    ctx.fillStyle = 'white';
    imageSegments.forEach(segment => {
      if (segment.isRevealed) {
        const x = (segment.x / 100) * canvas.width;
        const y = (segment.y / 100) * canvas.height;
        const width = (segment.width / 100) * canvas.width;
        const height = (segment.height / 100) * canvas.height;
        
        ctx.fillRect(x, y, width, height);
      }
    });

    return canvas.toDataURL();
  };

  const maskDataUrl = useMemo(() => createRevealMask(), [imageSegments]);

  return (
    <div className="w-full max-w-2xl mx-auto">
      {/* Controls */}
      <div className="flex justify-between items-center mb-4">
        <div className="flex space-x-2">
          <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800">
            {revealedCount} / {totalSegments} revealed
          </span>
          <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-purple-100 text-purple-800">
            {revealProgress.toFixed(1)}% complete
          </span>
        </div>
        
        <div className="flex space-x-2">
          <button
            onClick={() => setShowRevealedOnly(!showRevealedOnly)}
            className={`p-2 rounded-full transition-colors ${
              showRevealedOnly 
                ? 'text-blue-600 bg-blue-100 hover:bg-blue-200' 
                : 'text-gray-600 bg-gray-100 hover:bg-gray-200'
            }`}
            title={showRevealedOnly ? 'Show full image' : 'Show revealed only'}
          >
            {showRevealedOnly ? <EyeIcon className="h-5 w-5" /> : <EyeSlashIcon className="h-5 w-5" />}
          </button>
          
          <button
            onClick={() => setShowGrid(!showGrid)}
            className={`p-2 rounded-full transition-colors ${
              showGrid 
                ? 'text-blue-600 bg-blue-100 hover:bg-blue-200' 
                : 'text-gray-600 bg-gray-100 hover:bg-gray-200'
            }`}
            title={showGrid ? 'Hide grid' : 'Show grid'}
          >
            {showGrid ? <Square3Stack3DIcon className="h-5 w-5" /> : <Squares2X2Icon className="h-5 w-5" />}
          </button>
        </div>
      </div>

      {/* Progress bar */}
      <div className="mb-4">
        <div className="w-full bg-gray-200 rounded-full h-2">
          <div
            className="bg-gradient-to-r from-indigo-500 to-purple-600 h-2 rounded-full transition-all duration-300"
            style={{ width: `${revealProgress}%` }}
          />
        </div>
      </div>

      {/* Image container */}
      <div className="relative w-full aspect-square rounded-lg overflow-hidden bg-gray-100 border-2 border-gray-200">
        {/* Base image */}
        <img
          src={imageUrl}
          alt="Raffle item"
          onLoad={handleImageLoad}
          className={`w-full h-full object-cover transition-opacity duration-300 ${
            imageLoaded ? 'opacity-100' : 'opacity-0'
          }`}
          style={showRevealedOnly ? {
            WebkitMask: `url(${maskDataUrl})`,
            mask: `url(${maskDataUrl})`,
            WebkitMaskSize: 'cover',
            maskSize: 'cover',
          } : {}}
        />

        {/* Loading placeholder */}
        {!imageLoaded && (
          <div className="absolute inset-0 flex items-center justify-center bg-gray-200">
            <div className="text-gray-600">Loading image...</div>
          </div>
        )}

        {/* Grid overlay */}
        {showGrid && imageLoaded && (
          <div
            className="absolute inset-0 grid gap-0.5 p-1"
            style={{
              gridTemplateColumns: `repeat(${gridDimensions.cols}, 1fr)`,
              gridTemplateRows: `repeat(${gridDimensions.rows}, 1fr)`,
            }}
          >
            {imageSegments.map(segment => (
              <div
                key={segment.id}
                className={`border rounded-sm transition-all duration-300 flex items-center justify-center ${
                  segment.isRevealed 
                    ? 'border-green-400 bg-transparent' 
                    : 'border-gray-400 bg-black bg-opacity-70'
                }`}
              >
                {!segment.isRevealed && (
                  <span className="text-white font-semibold text-xs">
                    {segment.id}
                  </span>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Reveal overlay for non-revealed areas (when not showing revealed only) */}
        {!showRevealedOnly && imageLoaded && (
          <div
            className="absolute inset-0 grid gap-0"
            style={{
              gridTemplateColumns: `repeat(${gridDimensions.cols}, 1fr)`,
              gridTemplateRows: `repeat(${gridDimensions.rows}, 1fr)`,
            }}
          >
            {imageSegments.map(segment => (
              !segment.isRevealed && (
                <div
                  key={segment.id}
                  className="bg-black bg-opacity-80 flex items-center justify-center transition-opacity duration-300"
                >
                  {showGrid && (
                    <span className="text-white font-semibold text-xs">
                      {segment.id}
                    </span>
                  )}
                </div>
              )
            ))}
          </div>
        )}
      </div>

      {/* Legend */}
      <div className="flex justify-center space-x-4 mt-4">
        <div className="flex items-center space-x-2">
          <div className="w-4 h-4 bg-green-500 rounded-sm" />
          <span className="text-sm text-gray-600">Revealed</span>
        </div>
        <div className="flex items-center space-x-2">
          <div className="w-4 h-4 bg-black bg-opacity-70 rounded-sm" />
          <span className="text-sm text-gray-600">Hidden</span>
        </div>
      </div>
    </div>
  );
};

export default ImageReveal;