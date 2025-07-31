import { useState, useEffect, useCallback, useMemo } from 'react';
import { toast } from 'react-toastify';

import { Raffle } from '../types/raffle';
import { calculateGridDimensions, divideImageIntoSegments, ImageSegment } from '../utils/imageProcessing';
import { useAppSelector, useAppDispatch } from '../store';
import { apiRequest } from '../services/api';

interface UseRaffleGridProps {
  raffle: Raffle;
  onBoxPurchase?: (boxNumbers: number[]) => Promise<void>;
}

interface UseRaffleGridReturn {
  // Grid state
  gridDimensions: { rows: number; cols: number };
  imageSegments: ImageSegment[];
  soldBoxes: Set<number>;
  selectedBoxes: number[];
  
  // Loading states
  isLoadingImage: boolean;
  isProcessingPurchase: boolean;
  
  // Actions
  selectBox: (boxNumber: number) => void;
  deselectBox: (boxNumber: number) => void;
  clearSelection: () => void;
  purchaseSelectedBoxes: () => Promise<void>;
  
  // Computed values
  totalCost: number;
  canAffordWithCredits: boolean;
  progressPercentage: number;
}

export const useRaffleGrid = ({ 
  raffle, 
  onBoxPurchase 
}: UseRaffleGridProps): UseRaffleGridReturn => {
  const dispatch = useAppDispatch();
  const { balance } = useAppSelector(state => state.credit);
  const { user } = useAppSelector(state => state.auth);
  
  // State
  const [selectedBoxes, setSelectedBoxes] = useState<number[]>([]);
  const [imageSegments, setImageSegments] = useState<ImageSegment[]>([]);
  const [isLoadingImage, setIsLoadingImage] = useState(false);
  const [isProcessingPurchase, setIsProcessingPurchase] = useState(false);

  // Calculate grid dimensions
  const gridDimensions = useMemo(() => {
    return calculateGridDimensions(raffle.totalBoxes);
  }, [raffle.totalBoxes]);

  // Get sold boxes from participants
  const soldBoxes = useMemo(() => {
    const sold = new Set<number>();
    raffle.participants?.forEach(participant => {
      sold.add(participant.boxNumber);
    });
    return sold;
  }, [raffle.participants]);

  // Calculate progress percentage
  const progressPercentage = useMemo(() => {
    return (raffle.boxesSold / raffle.totalBoxes) * 100;
  }, [raffle.boxesSold, raffle.totalBoxes]);

  // Calculate total cost
  const totalCost = useMemo(() => {
    return selectedBoxes.length * raffle.boxPrice;
  }, [selectedBoxes.length, raffle.boxPrice]);

  // Check if user can afford with credits
  const canAffordWithCredits = useMemo(() => {
    return balance >= totalCost;
  }, [balance, totalCost]);

  // Load and process image segments
  useEffect(() => {
    const loadImageSegments = async () => {
      if (!raffle.item.imageUrls[0]) return;

      setIsLoadingImage(true);
      try {
        const segments = await divideImageIntoSegments(
          raffle.item.imageUrls[0],
          gridDimensions,
          {
            quality: 0.85,
            format: 'jpeg',
            maxSegmentSize: 150,
          }
        );
        setImageSegments(segments);
      } catch (error) {
        console.error('Failed to process image segments:', error);
        toast.error('Failed to load raffle image');
      } finally {
        setIsLoadingImage(false);
      }
    };

    loadImageSegments();
  }, [raffle.item.imageUrls, gridDimensions]);

  // Box selection actions
  const selectBox = useCallback((boxNumber: number) => {
    if (soldBoxes.has(boxNumber)) {
      toast.warning('This box has already been sold');
      return;
    }

    setSelectedBoxes(prev => {
      if (prev.includes(boxNumber)) {
        return prev; // Already selected
      }
      return [...prev, boxNumber];
    });
  }, [soldBoxes]);

  const deselectBox = useCallback((boxNumber: number) => {
    setSelectedBoxes(prev => prev.filter(box => box !== boxNumber));
  }, []);

  const clearSelection = useCallback(() => {
    setSelectedBoxes([]);
  }, []);

  // Purchase selected boxes
  const purchaseSelectedBoxes = useCallback(async () => {
    if (!user) {
      toast.error('Please log in to purchase boxes');
      return;
    }

    if (selectedBoxes.length === 0) {
      toast.warning('Please select at least one box');
      return;
    }

    // Check if any selected boxes are now sold
    const nowSoldBoxes = selectedBoxes.filter(box => soldBoxes.has(box));
    if (nowSoldBoxes.length > 0) {
      toast.error(`Box${nowSoldBoxes.length > 1 ? 'es' : ''} ${nowSoldBoxes.join(', ')} ${nowSoldBoxes.length > 1 ? 'have' : 'has'} been sold by another user`);
      setSelectedBoxes(prev => prev.filter(box => !soldBoxes.has(box)));
      return;
    }

    setIsProcessingPurchase(true);
    try {
      if (onBoxPurchase) {
        await onBoxPurchase(selectedBoxes);
      } else {
        // Default purchase logic
        await apiRequest.post(`/raffles/${raffle.id}/buy-boxes`, {
          boxNumbers: selectedBoxes,
          paymentMethod: canAffordWithCredits ? 'credits' : 'card',
        });
      }

      toast.success(`Successfully purchased ${selectedBoxes.length} box${selectedBoxes.length > 1 ? 'es' : ''}!`);
      setSelectedBoxes([]);
    } catch (error: any) {
      console.error('Purchase failed:', error);
      toast.error(error.response?.data?.message || 'Purchase failed. Please try again.');
    } finally {
      setIsProcessingPurchase(false);
    }
  }, [user, selectedBoxes, soldBoxes, raffle.id, canAffordWithCredits, onBoxPurchase]);

  // Handle box click (toggle selection)
  const handleBoxClick = useCallback((boxNumber: number) => {
    if (selectedBoxes.includes(boxNumber)) {
      deselectBox(boxNumber);
    } else {
      selectBox(boxNumber);
    }
  }, [selectedBoxes, selectBox, deselectBox]);

  return {
    // Grid state
    gridDimensions,
    imageSegments,
    soldBoxes,
    selectedBoxes,
    
    // Loading states
    isLoadingImage,
    isProcessingPurchase,
    
    // Actions
    selectBox,
    deselectBox,
    clearSelection,
    purchaseSelectedBoxes,
    
    // Computed values
    totalCost,
    canAffordWithCredits,
    progressPercentage,
  };
};