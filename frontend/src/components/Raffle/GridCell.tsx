import React from 'react';
import {
  CheckCircleIcon,
  LockClosedIcon,
} from '@heroicons/react/24/solid';
import {
  RadioButtonIcon,
} from '@heroicons/react/24/outline';

interface GridCellProps {
  boxNumber: number;
  isSold: boolean;
  isSelected: boolean;
  isHovered: boolean;
  isRecentlyPurchased?: boolean;
  isAnimating?: boolean;
  onClick: () => void;
  onMouseEnter: () => void;
  onMouseLeave: () => void;
  disabled: boolean;
  imageSegment?: string;
  revealProgress: number;
}

const GridCell: React.FC<GridCellProps> = ({
  boxNumber,
  isSold,
  isSelected,
  isHovered,
  isRecentlyPurchased = false,
  isAnimating = false,
  onClick,
  onMouseEnter,
  onMouseLeave,
  disabled,
  imageSegment,
  revealProgress,
}) => {
  // Calculate if this cell should show the image based on reveal progress
  const shouldReveal = revealProgress >= (boxNumber / 100) * 100; // Simplified logic

  // Determine cell state and styling
  const getCellClasses = () => {
    let baseClasses = "relative aspect-square border-2 rounded cursor-pointer transition-all duration-200 flex items-center justify-center overflow-hidden min-h-[40px] min-w-[40px]";
    
    // Add animation classes
    if (isAnimating) {
      baseClasses += " animate-bounce";
    }
    
    if (isSold) {
      return `${baseClasses} bg-red-50 border-red-300 cursor-not-allowed`;
    }
    
    if (isSelected) {
      return `${baseClasses} bg-indigo-100 border-indigo-500 transform scale-95 shadow-lg ring-2 ring-indigo-300`;
    }
    
    if (isHovered && !disabled) {
      return `${baseClasses} bg-indigo-50 border-indigo-400 transform scale-105 shadow-md`;
    }
    
    return `${baseClasses} bg-white border-gray-300 hover:border-indigo-400 hover:scale-102`;
  };

  // Get icon based on state
  const getIcon = () => {
    if (isSold) {
      return <LockClosedIcon className="h-4 w-4 text-red-600" />;
    }
    if (isSelected) {
      return <CheckCircleIcon className="h-4 w-4 text-indigo-600" />;
    }
    return <div className="h-4 w-4 border-2 border-gray-400 rounded-full" />;
  };

  // Get tooltip content
  const getTooltipContent = () => {
    if (isSold) return `Box ${boxNumber} - Already sold`;
    if (isSelected) return `Box ${boxNumber} - Selected for purchase`;
    return `Box ${boxNumber} - Click to select`;
  };

  return (
    <div
      className={getCellClasses()}
      onClick={onClick}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      title={getTooltipContent()}
    >
      {/* Background image segment (if available and should be revealed) */}
      {imageSegment && shouldReveal && (
        <div
          className="absolute inset-0 bg-cover bg-center opacity-30"
          style={{ backgroundImage: `url(${imageSegment})` }}
        />
      )}

      {/* Overlay content */}
      <div className="relative z-10 flex flex-col items-center justify-center gap-1">
        {getIcon()}
        <span
          className={`text-xs font-semibold ${
            isSold 
              ? 'text-red-600' 
              : isSelected 
                ? 'text-indigo-600' 
                : 'text-gray-600'
          }`}
        >
          {boxNumber}
        </span>
      </div>

      {/* Selection indicator */}
      {isSelected && (
        <div className="absolute -top-1 -right-1 w-3 h-3 bg-indigo-600 rounded-full flex items-center justify-center z-20">
          <CheckCircleIcon className="h-2 w-2 text-white" />
        </div>
      )}

      {/* Sold indicator */}
      {isSold && (
        <div className="absolute inset-0 bg-red-600 bg-opacity-80 flex items-center justify-center z-20">
          <span className="text-white font-bold text-xs transform -rotate-45">
            SOLD
          </span>
        </div>
      )}

      {/* Hover effect overlay */}
      {isHovered && !disabled && !isSold && (
        <div className="absolute inset-0 bg-indigo-500 bg-opacity-10 z-10 pointer-events-none" />
      )}

      {/* Recently purchased animation */}
      {isRecentlyPurchased && (
        <div className="absolute -inset-1 border-2 border-green-500 rounded z-30 animate-pulse" />
      )}

      {/* Purchase animation overlay */}
      {isAnimating && (
        <div className="absolute inset-0 bg-green-400 bg-opacity-20 rounded z-30 animate-ping" />
      )}
    </div>
  );
};

export default GridCell;