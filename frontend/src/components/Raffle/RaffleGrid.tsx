import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { 
  ZoomInIcon, 
  ZoomOutIcon, 
  ArrowsPointingOutIcon,
  InformationCircleIcon,
  ClockIcon,
  UsersIcon,
  TagIcon,
  WifiIcon,
  SignalSlashIcon,
  XMarkIcon
} from '@heroicons/react/24/outline';
import { toast } from 'react-toastify';

import GridCell from './GridCell';
import BoxSelector from './BoxSelector';
import ImageReveal from './ImageReveal';
import MobileRaffleGrid from '../Mobile/MobileRaffleGrid';
import { Button } from '../ui/Button';
import { Raffle, Participant } from '../../types/raffle';
import { useAppSelector } from '../../store';
import { useRaffleRealTime } from '../../hooks/useRaffleRealTime';
import { useResponsive } from '../../hooks/useResponsive';
import { getResponsiveGridDimensions } from '../../utils/imageProcessing';

interface RaffleGridProps {
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

const RaffleGrid: React.FC<RaffleGridProps> = ({
  raffle,
  onBoxPurchase,
  onRaffleUpdate,
  isLoading = false,
  disabled = false,
}) => {
  const { screenWidth, isMobile, isTablet } = useResponsive();

  // Use mobile component for mobile devices
  if (isMobile) {
    return (
      <MobileRaffleGrid
        raffle={raffle}
        onBoxPurchase={onBoxPurchase}
        onRaffleUpdate={onRaffleUpdate}
        isLoading={isLoading}
        disabled={disabled}
      />
    );
  }

  const { user } = useAppSelector(state => state.auth);
  const [selectedBoxes, setSelectedBoxes] = useState<number[]>([]);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [showImagePreview, setShowImagePreview] = useState(false);
  const [hoveredBox, setHoveredBox] = useState<number | null>(null);
  const [recentPurchases, setRecentPurchases] = useState<Array<{ boxNumbers: number[]; timestamp: number }>>([]);
  const [animatingBoxes, setAnimatingBoxes] = useState<Set<number>>(new Set());
  const [connectionError, setConnectionError] = useState<string | null>(null);

  // Calculate optimal grid dimensions based on total boxes and screen size
  const gridDimensions = useMemo((): GridDimensions => {
    return getResponsiveGridDimensions(raffle.totalBoxes, screenWidth);
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

  // Handle box selection
  const handleBoxClick = useCallback((boxNumber: number) => {
    if (disabled || soldBoxes.has(boxNumber)) return;

    setSelectedBoxes(prev => {
      if (prev.includes(boxNumber)) {
        return prev.filter(box => box !== boxNumber);
      } else {
        return [...prev, boxNumber];
      }
    });
  }, [disabled, soldBoxes]);

  // Handle purchase
  const handlePurchase = useCallback(() => {
    if (selectedBoxes.length > 0) {
      onBoxPurchase(selectedBoxes);
      setSelectedBoxes([]);
    }
  }, [selectedBoxes, onBoxPurchase]);

  // Clear selection
  const handleClearSelection = useCallback(() => {
    setSelectedBoxes([]);
  }, []);

  // Zoom controls
  const handleZoomIn = useCallback(() => {
    setZoomLevel(prev => Math.min(prev + 0.2, 2));
  }, []);

  const handleZoomOut = useCallback(() => {
    setZoomLevel(prev => Math.max(prev - 0.2, 0.5));
  }, []);

  // Calculate total cost
  const totalCost = selectedBoxes.length * raffle.boxPrice;

  // Real-time updates
  const {
    isConnected: isRealTimeConnected,
    isConnecting: isRealTimeConnecting,
    activeUsers,
  } = useRaffleRealTime({
    raffleId: raffle.id,
    onBoxPurchased: useCallback((data: { boxNumbers: number[]; participant: Participant }) => {
      // Add animation for purchased boxes
      setAnimatingBoxes(prev => new Set([...prev, ...data.boxNumbers]));
      
      // Add visual feedback for recent purchases
      setRecentPurchases(prev => [
        ...prev,
        { boxNumbers: data.boxNumbers, timestamp: Date.now() }
      ]);

      // Show success toast
      const purchaserName = data.participant.user?.email || 'Someone';
      toast.success(
        `${purchaserName} purchased ${data.boxNumbers.length} box${data.boxNumbers.length > 1 ? 'es' : ''}!`,
        {
          position: 'top-right',
          autoClose: 3000,
        }
      );

      // Remove animations and recent purchase indicators
      setTimeout(() => {
        setAnimatingBoxes(prev => {
          const newSet = new Set(prev);
          data.boxNumbers.forEach(box => newSet.delete(box));
          return newSet;
        });
        
        setRecentPurchases(prev => 
          prev.filter(purchase => Date.now() - purchase.timestamp < 3000)
        );
      }, 3000);

      // Update raffle data if callback provided
      if (onRaffleUpdate && data.updatedRaffle) {
        onRaffleUpdate(data.updatedRaffle);
      }
      
      // Clear any connection errors
      setConnectionError(null);
    }, [onRaffleUpdate]),
    onRaffleFull: useCallback(() => {
      toast.info('🎉 Raffle is now full! Winner selection will begin shortly.', {
        position: 'top-center',
        autoClose: 5000,
      });
    }, []),
    onWinnerSelected: useCallback((data: { winners: any[] }) => {
      const winnerNames = data.winners.map((w: any) => w.user?.email || 'Anonymous').join(', ');
      toast.success(`🏆 Winners selected: ${winnerNames}`, {
        position: 'top-center',
        autoClose: 10000,
      });
    }, []),
    onRaffleCompleted: useCallback(() => {
      toast.success('🎊 Raffle has been completed! Check your dashboard for results.', {
        position: 'top-center',
        autoClose: 10000,
      });
    }, []),
  });

  // Handle connection errors
  useEffect(() => {
    if (!isRealTimeConnected && !isRealTimeConnecting) {
      const timer = setTimeout(() => {
        setConnectionError('Live updates are currently unavailable');
      }, 5000);
      
      return () => clearTimeout(timer);
    } else {
      setConnectionError(null);
    }
  }, [isRealTimeConnected, isRealTimeConnecting]);

  // Time remaining (if applicable)
  const timeRemaining = useMemo(() => {
    // This would be calculated based on raffle end time
    // For now, return null as we don't have end time in the model
    return null;
  }, []);

  return (
    <div className="w-full h-full">
      {/* Connection status alert */}
      {connectionError && (
        <div className="mb-4 p-4 bg-yellow-50 border border-yellow-200 rounded-lg animate-fade-in">
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <SignalSlashIcon className="h-5 w-5 text-yellow-600 mr-2" />
              <p className="text-yellow-800 text-sm">
                {connectionError}. The grid may not show real-time changes.
              </p>
            </div>
            <button
              onClick={() => setConnectionError(null)}
              className="text-yellow-600 hover:text-yellow-800 transition-colors"
            >
              <XMarkIcon className="h-4 w-4" />
            </button>
          </div>
        </div>
      )}

      {/* Connecting indicator */}
      {isRealTimeConnecting && (
        <div className="mb-4 p-4 bg-blue-50 border border-blue-200 rounded-lg animate-pulse">
          <div className="flex items-center">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600 mr-2"></div>
            <p className="text-blue-800 text-sm">
              Connecting to live updates...
            </p>
          </div>
        </div>
      )}

      {/* Header with raffle info */}
      <div className="bg-white rounded-lg shadow-sm border p-6 mb-4">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-2xl font-semibold text-gray-900">
            {raffle.item.title}
          </h2>
          <div className="flex items-center space-x-2">
            {/* Real-time connection indicator */}
            <div className="relative">
              <button
                className={`p-2 rounded-full transition-colors ${
                  isRealTimeConnected 
                    ? 'text-green-600 hover:bg-green-50' 
                    : 'text-gray-400 hover:bg-gray-50'
                }`}
                title={
                  isRealTimeConnected 
                    ? `Live updates active • ${activeUsers} users viewing`
                    : isRealTimeConnecting 
                      ? 'Connecting to live updates...'
                      : 'Live updates disconnected'
                }
              >
                {isRealTimeConnected ? (
                  <WifiIcon className="h-5 w-5" />
                ) : (
                  <SignalSlashIcon className="h-5 w-5" />
                )}
              </button>
              {activeUsers > 0 && (
                <span className="absolute -top-1 -right-1 bg-blue-500 text-white text-xs rounded-full h-5 w-5 flex items-center justify-center">
                  {activeUsers > 99 ? '99+' : activeUsers}
                </span>
              )}
            </div>
            
            <button
              onClick={() => setShowImagePreview(true)}
              className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-50 rounded-full transition-colors"
              title="Raffle Information"
            >
              <InformationCircleIcon className="h-5 w-5" />
            </button>
            <button
              onClick={handleZoomIn}
              disabled={zoomLevel >= 2}
              className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-50 rounded-full transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              title="Zoom In"
            >
              <ZoomInIcon className="h-5 w-5" />
            </button>
            <button
              onClick={handleZoomOut}
              disabled={zoomLevel <= 0.5}
              className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-50 rounded-full transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              title="Zoom Out"
            >
              <ZoomOutIcon className="h-5 w-5" />
            </button>
          </div>
        </div>

        {/* Progress and stats */}
        <div className="space-y-4">
          <div>
            <div className="flex justify-between items-center mb-2">
              <span className="text-sm text-gray-600">
                Progress: {raffle.boxesSold} / {raffle.totalBoxes} boxes sold
              </span>
              <span className="text-sm text-gray-600">
                {progressPercentage.toFixed(1)}%
              </span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="bg-gradient-to-r from-indigo-500 to-purple-600 h-2 rounded-full transition-all duration-300"
                style={{ width: `${progressPercentage}%` }}
              />
            </div>
          </div>

          <div className="flex flex-wrap gap-2">
            <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800">
              <TagIcon className="h-4 w-4 mr-1" />
              ${raffle.boxPrice} per box
            </span>
            <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-800">
              <UsersIcon className="h-4 w-4 mr-1" />
              {raffle.participants?.length || 0} participants
            </span>
            {timeRemaining && (
              <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-yellow-100 text-yellow-800">
                <ClockIcon className="h-4 w-4 mr-1" />
                {timeRemaining}
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Grid container */}
      <div className="bg-white rounded-lg shadow-sm border p-4 mb-4 overflow-hidden">
        <div
          className="grid gap-1 max-h-[60vh] overflow-auto transition-transform duration-300"
          style={{
            gridTemplateColumns: `repeat(${gridDimensions.cols}, 1fr)`,
            gridTemplateRows: `repeat(${gridDimensions.rows}, 1fr)`,
            aspectRatio: `${gridDimensions.cols} / ${gridDimensions.rows}`,
            transform: `scale(${zoomLevel})`,
            transformOrigin: 'center',
          }}
        >
          {Array.from({ length: raffle.totalBoxes }, (_, index) => {
            const boxNumber = index + 1;
            const isSold = soldBoxes.has(boxNumber);
            const isSelected = selectedBoxes.includes(boxNumber);
            const isHovered = hoveredBox === boxNumber;

            const isRecentlyPurchased = recentPurchases.some(purchase => 
              purchase.boxNumbers.includes(boxNumber) && 
              Date.now() - purchase.timestamp < 3000
            );
            const isAnimating = animatingBoxes.has(boxNumber);

            return (
              <GridCell
                key={boxNumber}
                boxNumber={boxNumber}
                isSold={isSold}
                isSelected={isSelected}
                isHovered={isHovered}
                isRecentlyPurchased={isRecentlyPurchased}
                isAnimating={isAnimating}
                onClick={() => handleBoxClick(boxNumber)}
                onMouseEnter={() => setHoveredBox(boxNumber)}
                onMouseLeave={() => setHoveredBox(null)}
                disabled={disabled || isSold}
                imageSegment={raffle.item.imageUrls[0]} // We'll implement image segmentation later
                revealProgress={progressPercentage}
              />
            );
          })}
        </div>
      </div>

      {/* Box selector and purchase controls */}
      {user && (
        <BoxSelector
          selectedBoxes={selectedBoxes}
          totalCost={totalCost}
          currency="USD"
          onPurchase={handlePurchase}
          onClearSelection={handleClearSelection}
          isLoading={isLoading}
          disabled={disabled || selectedBoxes.length === 0}
        />
      )}

      {/* Image preview modal */}
      {showImagePreview && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-4xl w-full max-h-[90vh] overflow-auto">
            <div className="flex justify-between items-center p-6 border-b">
              <h3 className="text-xl font-semibold text-gray-900">
                {raffle.item.title}
              </h3>
              <button
                onClick={() => setShowImagePreview(false)}
                className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100 transition-colors"
              >
                <XMarkIcon className="h-6 w-6" />
              </button>
            </div>
            <div className="p-6">
              <ImageReveal
                imageUrl={raffle.item.imageUrls[0]}
                revealProgress={progressPercentage}
                gridDimensions={gridDimensions}
                soldBoxes={Array.from(soldBoxes)}
              />
              <div className="mt-6">
                <p className="text-gray-700 mb-4">
                  {raffle.item.description}
                </p>
                <div className="flex flex-wrap gap-2">
                  <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-100 text-gray-800">
                    Original Price: ${raffle.item.price}
                  </span>
                  <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-100 text-gray-800">
                    Category: {raffle.item.category}
                  </span>
                  <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-100 text-gray-800">
                    Winners: {raffle.totalWinners}
                  </span>
                </div>
              </div>
            </div>
            <div className="flex justify-end p-6 border-t">
              <Button
                variant="secondary"
                onClick={() => setShowImagePreview(false)}
              >
                Close
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default RaffleGrid;