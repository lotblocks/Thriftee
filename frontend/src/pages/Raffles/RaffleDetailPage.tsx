import React, { useEffect, useState } from 'react';
import {
  ArrowLeftIcon,
  ShareIcon,
  HeartIcon,
  ExclamationTriangleIcon,
} from '@heroicons/react/24/outline';
import { HeartIcon as HeartSolidIcon } from '@heroicons/react/24/solid';
import { useParams, useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';

import RaffleGrid from '../../components/Raffle/RaffleGrid';
import { Button } from '../../components/ui/Button';
import { LoadingSpinner } from '../../components/ui/LoadingSpinner';
import { Raffle } from '../../types/raffle';
import { raffleService } from '../../services/raffleService';
import { useAppSelector } from '../../store';

const RaffleDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { user } = useAppSelector(state => state.auth);
  
  const [raffle, setRaffle] = useState<Raffle | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isFavorited, setIsFavorited] = useState(false);

  // Load raffle data
  useEffect(() => {
    const loadRaffle = async () => {
      if (!id) {
        setError('Raffle ID not provided');
        setIsLoading(false);
        return;
      }

      try {
        setIsLoading(true);
        const raffleData = await raffleService.getRaffle(id);
        setRaffle(raffleData);
        setError(null);
      } catch (error: any) {
        console.error('Failed to load raffle:', error);
        setError(error.response?.data?.message || 'Failed to load raffle');
      } finally {
        setIsLoading(false);
      }
    };

    loadRaffle();
  }, [id]);

  // Handle box purchase
  const handleBoxPurchase = async (boxNumbers: number[]) => {
    if (!raffle || !user) return;

    try {
      await raffleService.purchaseBoxes(raffle.id, {
        boxNumbers,
        paymentMethod: 'credits', // This would be determined by user selection
      });

      // Reload raffle data to get updated state
      const updatedRaffle = await raffleService.getRaffle(raffle.id);
      setRaffle(updatedRaffle);
      
      toast.success(`Successfully purchased ${boxNumbers.length} box${boxNumbers.length > 1 ? 'es' : ''}!`);
    } catch (error: any) {
      console.error('Purchase failed:', error);
      throw error; // Re-throw to let the grid component handle it
    }
  };

  // Handle share
  const handleShare = async () => {
    if (navigator.share && raffle) {
      try {
        await navigator.share({
          title: raffle.item.title,
          text: `Check out this raffle: ${raffle.item.title}`,
          url: window.location.href,
        });
      } catch (error) {
        // Fallback to clipboard
        navigator.clipboard.writeText(window.location.href);
        toast.success('Link copied to clipboard!');
      }
    } else {
      // Fallback to clipboard
      navigator.clipboard.writeText(window.location.href);
      toast.success('Link copied to clipboard!');
    }
  };

  // Handle favorite toggle
  const handleFavoriteToggle = () => {
    setIsFavorited(!isFavorited);
    toast.success(isFavorited ? 'Removed from favorites' : 'Added to favorites');
  };

  if (isLoading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="flex items-center justify-center min-h-[400px]">
          <LoadingSpinner size="lg" />
        </div>
      </div>
    );
  }

  if (error || !raffle) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="max-w-md mx-auto text-center">
          <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
            <div className="flex items-center justify-center mb-2">
              <ExclamationTriangleIcon className="h-6 w-6 text-red-600" />
            </div>
            <p className="text-red-800">{error || 'Raffle not found'}</p>
          </div>
          <Button
            variant="secondary"
            onClick={() => navigate('/raffles')}
            className="inline-flex items-center"
          >
            <ArrowLeftIcon className="h-4 w-4 mr-2" />
            Back to Raffles
          </Button>
        </div>
      </div>
    );
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800';
      case 'full':
        return 'bg-blue-100 text-blue-800';
      case 'completed':
        return 'bg-gray-100 text-gray-800';
      case 'cancelled':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      {/* Header */}
      <div className="mb-8">
        <div className="flex justify-between items-center mb-4">
          <Button
            variant="secondary"
            onClick={() => navigate('/raffles')}
            className="inline-flex items-center"
          >
            <ArrowLeftIcon className="h-4 w-4 mr-2" />
            Back to Raffles
          </Button>
          
          <div className="flex space-x-2">
            <button
              onClick={handleFavoriteToggle}
              className={`inline-flex items-center px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                isFavorited
                  ? 'bg-red-100 text-red-800 hover:bg-red-200'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              {isFavorited ? (
                <HeartSolidIcon className="h-4 w-4 mr-2" />
              ) : (
                <HeartIcon className="h-4 w-4 mr-2" />
              )}
              {isFavorited ? 'Favorited' : 'Favorite'}
            </button>
            <button
              onClick={handleShare}
              className="inline-flex items-center px-3 py-2 rounded-lg text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition-colors"
            >
              <ShareIcon className="h-4 w-4 mr-2" />
              Share
            </button>
            <button className="inline-flex items-center px-3 py-2 rounded-lg text-sm font-medium bg-yellow-100 text-yellow-800 hover:bg-yellow-200 transition-colors">
              <ExclamationTriangleIcon className="h-4 w-4 mr-2" />
              Report
            </button>
          </div>
        </div>

        <h1 className="text-3xl font-bold text-gray-900 mb-4">
          {raffle.item.title}
        </h1>
        
        <div className="flex flex-wrap gap-2">
          <span className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${getStatusColor(raffle.status)}`}>
            {raffle.status.replace('_', ' ').toUpperCase()}
          </span>
          <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-100 text-gray-800">
            {raffle.item.category}
          </span>
          <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-100 text-gray-800">
            {raffle.totalWinners} Winner{raffle.totalWinners > 1 ? 's' : ''}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Main raffle grid */}
        <div className="lg:col-span-2">
          <RaffleGrid
            raffle={raffle}
            onBoxPurchase={handleBoxPurchase}
            onRaffleUpdate={setRaffle}
            disabled={raffle.status !== 'active' || !user}
          />
        </div>

        {/* Sidebar with details */}
        <div className="space-y-6">
          {/* Item details */}
          <div className="bg-white rounded-lg shadow-sm border p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Item Details
            </h3>
            <div className="space-y-4">
              <div>
                <p className="text-sm text-gray-600 mb-1">Original Price</p>
                <p className="text-xl font-semibold text-indigo-600">
                  ${raffle.item.price}
                </p>
              </div>
              
              <div>
                <p className="text-sm text-gray-600 mb-1">Description</p>
                <p className="text-gray-900">
                  {raffle.item.description}
                </p>
              </div>

              <div>
                <p className="text-sm text-gray-600 mb-1">Seller</p>
                <p className="text-gray-900">
                  Seller #{raffle.sellerId}
                </p>
              </div>
            </div>
          </div>

          {/* Raffle stats */}
          <div className="bg-white rounded-lg shadow-sm border p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Raffle Statistics
            </h3>
            <div className="space-y-4">
              <div>
                <p className="text-sm text-gray-600 mb-1">Total Boxes</p>
                <p className="text-xl font-semibold text-gray-900">
                  {raffle.totalBoxes}
                </p>
              </div>
              
              <div>
                <p className="text-sm text-gray-600 mb-1">Boxes Sold</p>
                <p className="text-xl font-semibold text-gray-900">
                  {raffle.boxesSold} / {raffle.totalBoxes}
                </p>
              </div>

              <div>
                <p className="text-sm text-gray-600 mb-1">Participants</p>
                <p className="text-xl font-semibold text-gray-900">
                  {raffle.participants?.length || 0}
                </p>
              </div>

              <div>
                <p className="text-sm text-gray-600 mb-1">Box Price</p>
                <p className="text-xl font-semibold text-indigo-600">
                  ${raffle.boxPrice}
                </p>
              </div>
            </div>
          </div>

          {/* Login prompt for non-authenticated users */}
          {!user && (
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
              <p className="text-blue-800 text-sm">
                <button
                  onClick={() => navigate('/login')}
                  className="font-medium text-blue-600 hover:text-blue-800 underline"
                >
                  Log in
                </button>
                {' '}or{' '}
                <button
                  onClick={() => navigate('/register')}
                  className="font-medium text-blue-600 hover:text-blue-800 underline"
                >
                  sign up
                </button>
                {' '}to participate in this raffle.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default RaffleDetailPage;