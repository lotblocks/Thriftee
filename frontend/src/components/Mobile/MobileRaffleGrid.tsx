import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import {
  ZoomInIcon,
  ZoomOutIcon,
  InformationCircleIcon,
  ClockIcon,
  UsersIcon,
  TagIcon,
  WifiIcon,
  SignalSlashIcon,
  XMarkIcon,
  Bars3Icon,
  Squares2X2Icon,
} from '@heroicons/react/24/outline';
import { toast } from 'react-toastify';
import MobileGridCell from './MobileGridCell';
import MobileBoxSelector from './MobileBoxSelector';
import MobileNavigation from './MobileNavigation';
import { Button } from '../ui/Button';
import { Raffle, Participant } from '../../types/raffle';
import { useAppSelector } from '../../store';
import { useRaffleRealTime } from '../../hooks/useRaffleRealTime';
import { useResponsive, useTouchDevice } from '../../hooks/useResponsive';
import { getResponsiveGridDimensions } from '../../utils/imageProcessing';
import { useTouchGestures, useMobileInteractions, usePullToRefresh, useHapticFeedback } from '../../hooks/useTouchGestures';

interface MobileRaffleGridProps {
  raffle: Raffle;
  onBoxPurchase: (boxNumbers: number[]) => void;
  onRaffleUpdate?: (updatedRaffle: Raffle) => void;
  isLoading?: boolean;
  disabled?: boolean;
}

interface GridDimensions {
  rows: number;
  cols: number;
}

const MobileRaffleGrid: React.FC<MobileRaffleGridProps> = ({
  raffle,
  onBoxPurchase,
  onRaffleUpdate,
  isLoading = false,
  disabled = false,
}) => {
  const { user } = useAppSelector(state => state.auth);
  const { screenWidth, screenHeight, orientation } = useResponsive();
  const isTouchDevice = useTouchDevice();
  const { lightTap, mediumTap, success, error } = useHapticFeedback();
  
  const [selectedBoxes, setSelectedBoxes] = useState<number[]>([]);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [showImagePreview, setShowImagePreview] = useState(false);
  const [hoveredBox, setHoveredBox] = useState<number | null>(null);
  const [recentPurchases, setRecentPurchases] = useState<Array<{ boxNumbers: number[]; timestamp: number }>>([]);
  const [animatingBoxes, setAnimatingBoxes] = useState<Set<number>>(new Set());
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [showMobileMenu, setShowMobileMenu] = useState(false);
  
  const gridContainerRef = useRef<HTMLDivElement>(null);
  const { isScrolling, scrollDirection } = useMobileInteractions();

  // Calculate mobile-optimized grid dimensions
  const gridDimensions = useMemo((): GridDimensions => {
    const mobileOptimized = getResponsiveGridDimensions(raffle.totalBoxes, screenWidth);
    
    // Further optimize for mobile touch targets
    if (screenWidth < 640) {
      const minCellSize = 44; // iOS recommended minimum touch target
      const availableWidth = screenWidth - 32; // Account for padding
      const maxCols = Math.floor(availableWidth / minCellSize);
      
      if (mobileOptimized.cols > maxCols) {
        const newCols = maxCols;
        const newRows = Math.ceil(raffle.totalBoxes / newCols);
        return { rows: newRows, cols: newCols };
      }
    }
    
    return mobileOptimized;
  }, [raffle.totalBoxes, screenWidth]);

  // Calculate progress percentage
  const progressPercentage = (raffle.boxesSold / raffle.totalBoxes) * 100;

  // Get sold boxes from participants
  const soldBoxes = useMemo(() => {
    const sold = new Set<number>();
    raffle.participants?.forEach(participant => {
      sold.add(participant.boxNumber);
    });
    return sold;
  }, [raffle.participants]);

  // Touch gestures for grid interaction
  const { isGesturing } = useTouchGestures(gridContainerRef, {
    onPinch: ({ scale, center }) => {
      setZoomLevel(prev => Math.max(0.5, Math.min(3, prev * scale)));
    },
    onDoubleTap: ({ x, y }) => {
      // Double tap to zoom in/out
      setZoomLevel(prev => prev > 1.5 ? 1 : 2);
      mediumTap();
    },
    onLongPress: ({ x, y }) => {
      // Long press for context menu or info
      mediumTap();
      // Could show box info or context menu
    },
    preventDefault: false,
  });

  // Pull to refresh functionality
  const { containerRef: pullRefreshRef, isPulling, pullDistance, isRefreshing } = usePullToRefresh(
    async () => {
      // Refresh raffle data
      if (onRaffleUpdate) {
        // Simulate refresh - in real app, this would fetch fresh data
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
  );

  // Handle box selection with haptic feedback
  const handleBoxClick = useCallback((boxNumber: number) => {
    if (disabled || soldBoxes.has(boxNumber)) {
      error();
      return;
    }

    lightTap();
    
    setSelectedBoxes(prev => {
      if (prev.includes(boxNumber)) {
        return prev.filter(box => box !== boxNumber);
      } else {
        return [...prev, boxNumber];
      }
    });
  }, [disabled, soldBoxes, lightTap, error]);

  // Handle purchase
  const handlePurchase = useCallback(() => {
    if (selectedBoxes.length > 0) {
      success();
      onBoxPurchase(selectedBoxes);
      setSelectedBoxes([]);
    }
  }, [selectedBoxes, onBoxPurchase, success]);

  // Clear selection
  const handleClearSelection = useCallback(() => {
    setSelectedBoxes([]);
    lightTap();
  }, [lightTap]);

  // Zoom controls (limited on mobile)
  const handleZoomIn = useCallback(() => {
    setZoomLevel(prev => Math.min(prev + 0.2, 3));
    lightTap();
  }, [lightTap]);

  const handleZoomOut = useCallback(() => {
    setZoomLevel(prev => Math.max(prev - 0.2, 0.5));
    lightTap();
  }, [lightTap]);

  // Calculate total cost
  const totalCost = selectedBoxes.length * raffle.boxPrice;

  // Real-time updates with mobile-specific handling
  const {
    isConnected: isRealTimeConnected,
    isConnecting: isRealTimeConnecting,
    activeUsers,
  } = useRaffleRealTime({
    raffleId: raffle.id,
    onBoxPurchased: useCallback((data: { boxNumbers: number[]; participant: Participant }) => {
      // Mobile-specific animations
      setAnimatingBoxes(prev => new Set([...prev, ...data.boxNumbers]));
      setRecentPurchases(prev => [
        ...prev,
        { boxNumbers: data.boxNumbers, timestamp: Date.now() }
      ]);

      // Shorter toast duration on mobile
      const purchaserName = data.participant.user?.email || 'Someone';
      toast.success(
        `${purchaserName} bought ${data.boxNumbers.length} box${data.boxNumbers.length > 1 ? 'es' : ''}!`,
        {
          position: 'bottom-center',
          autoClose: 2000,
        }
      );

      // Haptic feedback for purchases
      success();

      setTimeout(() => {
        setAnimatingBoxes(prev => {
          const newSet = new Set(prev);
          data.boxNumbers.forEach(box => newSet.delete(box));
          return newSet;
        });
        setRecentPurchases(prev => 
          prev.filter(purchase => Date.now() - purchase.timestamp < 2000)
        );
      }, 2000);

      if (onRaffleUpdate && data.updatedRaffle) {
        onRaffleUpdate(data.updatedRaffle);
      }
      setConnectionError(null);
    }, [onRaffleUpdate, success]),
    onRaffleFull: useCallback(() => {
      toast.info('🎉 Raffle is full!', {
        position: 'bottom-center',
        autoClose: 3000,
      });
    }, []),
    onWinnerSelected: useCallback((data: { winners: any[] }) => {
      toast.success('🏆 Winners selected!', {
        position: 'bottom-center',
        autoClose: 5000,
      });
    }, []),
    onRaffleCompleted: useCallback(() => {
      toast.success('🎊 Raffle completed!', {
        position: 'bottom-center',
        autoClose: 5000,
      });
    }, []),
  });

  // Handle connection errors
  useEffect(() => {
    if (!isRealTimeConnected && !isRealTimeConnecting) {
      const timer = setTimeout(() => {
        setConnectionError('Live updates unavailable');
      }, 3000);
      return () => clearTimeout(timer);
    } else {
      setConnectionError(null);
    }
  }, [isRealTimeConnected, isRealTimeConnecting]);

  // Time remaining (simplified for mobile)
  const timeRemaining = useMemo(() => {
    return null; // Implement based on raffle end time
  }, []);

  return (
    <div 
      ref={pullRefreshRef}
      className="w-full h-full min-h-screen bg-gray-50 relative"
    >
      {/* Pull to refresh indicator */}
      {isPulling && (
        <div 
          className="absolute top-0 left-0 right-0 z-30 bg-blue-500 text-white text-center py-2 transition-all duration-300"
          style={{ transform: `translateY(${Math.min(pullDistance - 80, 0)}px)` }}
        >
          {pullDistance >= 80 ? 'Release to refresh' : 'Pull to refresh'}
        </div>
      )}

      {/* Loading overlay for refresh */}
      {isRefreshing && (
        <div className="absolute top-0 left-0 right-0 z-30 bg-blue-500 text-white text-center py-2">
          <div className="flex items-center justify-center space-x-2">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
            <span>Refreshing...</span>
          </div>
        </div>
      )}

      {/* Mobile Navigation */}
      <MobileNavigation
        isOpen={showMobileMenu}
        onClose={() => setShowMobileMenu(false)}
        raffle={raffle}
        selectedBoxes={selectedBoxes}
        onClearSelection={handleClearSelection}
      />

      {/* Mobile Header */}
      <div className={`sticky top-0 z-40 bg-white border-b border-gray-200 px-4 py-3 transition-transform duration-300 ${
        isScrolling && scrollDirection === 'down' ? '-translate-y-full' : 'translate-y-0'
      }`}>
        <div className="flex items-center justify-between">
          <button
            onClick={() => setShowMobileMenu(true)}
            className="p-2 text-gray-600 hover:text-gray-900 rounded-lg hover:bg-gray-100"
          >
            <Bars3Icon className="h-6 w-6" />
          </button>
          
          <h1 className="text-lg font-semibold text-gray-900 truncate mx-4 flex-1">
            {raffle.item.title}
          </h1>
          
          <div className="flex items-center space-x-1">
            {/* Connection indicator */}
            <div className="relative">
              <button
                className={`p-2 rounded-lg transition-colors ${
                  isRealTimeConnected 
                    ? 'text-green-600 bg-green-50' 
                    : 'text-gray-400 bg-gray-50'
                }`}
              >
                {isRealTimeConnected ? (
                  <WifiIcon className="h-5 w-5" />
                ) : (
                  <SignalSlashIcon className="h-5 w-5" />
                )}
              </button>
              {activeUsers > 0 && (
                <span className="absolute -top-1 -right-1 bg-blue-500 text-white text-xs rounded-full h-4 w-4 flex items-center justify-center">
                  {activeUsers > 9 ? '9+' : activeUsers}
                </span>
              )}
            </div>
            
            <button
              onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')}
              className="p-2 text-gray-600 hover:text-gray-900 rounded-lg hover:bg-gray-100"
            >
              <Squares2X2Icon className="h-5 w-5" />
            </button>
          </div>
        </div>

        {/* Progress bar */}
        <div className="mt-3">
          <div className="flex justify-between items-center mb-1">
            <span className="text-xs text-gray-600">
              {raffle.boxesSold} / {raffle.totalBoxes} sold
            </span>
            <span className="text-xs text-gray-600">
              {progressPercentage.toFixed(0)}%
            </span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-gradient-to-r from-indigo-500 to-purple-600 h-2 rounded-full transition-all duration-300"
              style={{ width: `${progressPercentage}%` }}
            />
          </div>
        </div>

        {/* Stats chips */}
        <div className="flex space-x-2 mt-3 overflow-x-auto pb-1">
          <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800 whitespace-nowrap">
            <TagIcon className="h-3 w-3 mr-1" />
            ${raffle.boxPrice}
          </span>
          <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800 whitespace-nowrap">
            <UsersIcon className="h-3 w-3 mr-1" />
            {raffle.participants?.length || 0}
          </span>
          {timeRemaining && (
            <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 whitespace-nowrap">
              <ClockIcon className="h-3 w-3 mr-1" />
              {timeRemaining}
            </span>
          )}
        </div>
      </div>

      {/* Connection status alert */}
      {connectionError && (
        <div className="mx-4 mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <SignalSlashIcon className="h-4 w-4 text-yellow-600 mr-2" />
              <p className="text-yellow-800 text-sm">{connectionError}</p>
            </div>
            <button
              onClick={() => setConnectionError(null)}
              className="text-yellow-600 hover:text-yellow-800"
            >
              <XMarkIcon className="h-4 w-4" />
            </button>
          </div>
        </div>
      )}

      {/* Connecting indicator */}
      {isRealTimeConnecting && (
        <div className="mx-4 mt-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
          <div className="flex items-center">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600 mr-2"></div>
            <p className="text-blue-800 text-sm">Connecting...</p>
          </div>
        </div>
      )}

      {/* Main Grid Container */}
      <div className="flex-1 p-4">
        <div className="bg-white rounded-lg shadow-sm border overflow-hidden">
          {/* Grid Controls */}
          <div className="flex items-center justify-between p-3 border-b border-gray-200">
            <span className="text-sm font-medium text-gray-900">
              Select boxes to purchase
            </span>
            <div className="flex items-center space-x-2">
              <button
                onClick={handleZoomOut}
                disabled={zoomLevel <= 0.5}
                className="p-1 text-gray-600 hover:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ZoomOutIcon className="h-4 w-4" />
              </button>
              <span className="text-xs text-gray-500 min-w-[3rem] text-center">
                {Math.round(zoomLevel * 100)}%
              </span>
              <button
                onClick={handleZoomIn}
                disabled={zoomLevel >= 3}
                className="p-1 text-gray-600 hover:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ZoomInIcon className="h-4 w-4" />
              </button>
            </div>
          </div>

          {/* Grid */}
          <div 
            ref={gridContainerRef}
            className="p-3 overflow-auto" 
            style={{ maxHeight: 'calc(100vh - 300px)' }}
          >
            <div
              className="grid gap-1 mx-auto transition-transform duration-300"
              style={{
                gridTemplateColumns: `repeat(${gridDimensions.cols}, 1fr)`,
                gridTemplateRows: `repeat(${gridDimensions.rows}, 1fr)`,
                transform: `scale(${zoomLevel})`,
                transformOrigin: 'center top',
                maxWidth: `${gridDimensions.cols * 50}px`,
                touchAction: 'manipulation',
              }}
            >
              {Array.from({ length: raffle.totalBoxes }, (_, index) => {
                const boxNumber = index + 1;
                const isSold = soldBoxes.has(boxNumber);
                const isSelected = selectedBoxes.includes(boxNumber);
                const isHovered = hoveredBox === boxNumber;
                const isRecentlyPurchased = recentPurchases.some(purchase => 
                  purchase.boxNumbers.includes(boxNumber) && 
                  Date.now() - purchase.timestamp < 2000
                );
                const isAnimating = animatingBoxes.has(boxNumber);

                return (
                  <MobileGridCell
                    key={boxNumber}
                    boxNumber={boxNumber}
                    isSold={isSold}
                    isSelected={isSelected}
                    isHovered={isHovered}
                    isRecentlyPurchased={isRecentlyPurchased}
                    isAnimating={isAnimating}
                    onClick={() => handleBoxClick(boxNumber)}
                    onTouchStart={() => setHoveredBox(boxNumber)}
                    onTouchEnd={() => setHoveredBox(null)}
                    disabled={disabled || isSold}
                    imageSegment={raffle.item.imageUrls[0]}
                    revealProgress={progressPercentage}
                  />
                );
              })}
            </div>
          </div>
        </div>
      </div>

      {/* Mobile Box Selector - Fixed at bottom */}
      {user && (
        <div className="fixed bottom-0 left-0 right-0 z-50 bg-white border-t border-gray-200 safe-area-pb">
          <MobileBoxSelector
            selectedBoxes={selectedBoxes}
            totalCost={totalCost}
            currency="USD"
            onPurchase={handlePurchase}
            onClearSelection={handleClearSelection}
            isLoading={isLoading}
            disabled={disabled || selectedBoxes.length === 0}
          />
        </div>
      )}

      {/* Login prompt for non-authenticated users */}
      {!user && (
        <div className="fixed bottom-0 left-0 right-0 z-50 bg-blue-50 border-t border-blue-200 p-4 safe-area-pb">
          <p className="text-blue-800 text-sm text-center">
            <button className="font-medium text-blue-600 underline">
              Log in
            </button>
            {' '}or{' '}
            <button className="font-medium text-blue-600 underline">
              sign up
            </button>
            {' '}to participate
          </p>
        </div>
      )}

      {/* Image preview modal */}
      {showImagePreview && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-sm w-full max-h-[80vh] overflow-auto">
            <div className="flex justify-between items-center p-4 border-b">
              <h3 className="text-lg font-semibold text-gray-900">
                {raffle.item.title}
              </h3>
              <button
                onClick={() => setShowImagePreview(false)}
                className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100"
              >
                <XMarkIcon className="h-5 w-5" />
              </button>
            </div>
            <div className="p-4">
              <img
                src={raffle.item.imageUrls[0]}
                alt={raffle.item.title}
                className="w-full h-48 object-cover rounded-lg mb-4"
              />
              <p className="text-gray-700 text-sm mb-4">
                {raffle.item.description}
              </p>
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Original Price:</span>
                  <span className="font-semibold">${raffle.item.price}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Category:</span>
                  <span className="font-semibold">{raffle.item.category}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Winners:</span>
                  <span className="font-semibold">{raffle.totalWinners}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Gesture feedback overlay */}
      {isGesturing && (
        <div className="fixed inset-0 bg-black bg-opacity-10 z-20 pointer-events-none" />
      )}
    </div>
  );
};

export default MobileRaffleGrid;