import React, { useState } from 'react';
import {
  CubeIcon,
  TrophyIcon,
  ClockIcon,
  UsersIcon,
  EyeIcon,
  EllipsisVerticalIcon,
} from '@heroicons/react/24/outline';
import { useNavigate } from 'react-router-dom';

import { Button } from '../ui/Button';

interface RaffleParticipation {
  id: string;
  title: string;
  imageUrl: string;
  status: 'active' | 'completed' | 'won';
  boxesPurchased: number;
  totalBoxes: number;
  boxPrice: number;
  amountSpent: number;
  progress: number;
  endDate?: string;
  winnerSelected?: boolean;
  isWinner?: boolean;
  participants: number;
}

const ParticipationHistory: React.FC = () => {
  const navigate = useNavigate();
  const [viewMode, setViewMode] = useState<'active' | 'completed' | 'all'>('active');

  // Mock data - in real app, this would come from API
  const participations: RaffleParticipation[] = [
    {
      id: '1',
      title: 'iPhone 15 Pro Max - 256GB Space Black',
      imageUrl: 'https://images.unsplash.com/photo-1592750475338-74b7b21085ab?w=300',
      status: 'active',
      boxesPurchased: 5,
      totalBoxes: 100,
      boxPrice: 12,
      amountSpent: 60,
      progress: 78,
      participants: 45,
    },
    {
      id: '2',
      title: 'MacBook Pro 16" M3 Max',
      imageUrl: 'https://images.unsplash.com/photo-1517336714731-489689fd1ca8?w=300',
      status: 'active',
      boxesPurchased: 3,
      totalBoxes: 200,
      boxPrice: 15,
      amountSpent: 45,
      progress: 92,
      participants: 67,
    },
    {
      id: '3',
      title: 'Sony WH-1000XM5 Headphones',
      imageUrl: 'https://images.unsplash.com/photo-1583394838336-acd977736f90?w=300',
      status: 'completed',
      boxesPurchased: 2,
      totalBoxes: 50,
      boxPrice: 8,
      amountSpent: 16,
      progress: 100,
      winnerSelected: true,
      isWinner: false,
      participants: 28,
    },
    {
      id: '4',
      title: 'Nintendo Switch OLED',
      imageUrl: 'https://images.unsplash.com/photo-1606144042614-b2417e99c4e3?w=300',
      status: 'won',
      boxesPurchased: 4,
      totalBoxes: 75,
      boxPrice: 5,
      amountSpent: 20,
      progress: 100,
      winnerSelected: true,
      isWinner: true,
      participants: 42,
    },
  ];

  const filteredParticipations = participations.filter(p => {
    if (viewMode === 'all') return true;
    if (viewMode === 'active') return p.status === 'active';
    if (viewMode === 'completed') return p.status === 'completed' || p.status === 'won';
    return true;
  });

  const getStatusBadge = (participation: RaffleParticipation) => {
    if (participation.status === 'won') {
      return (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
          Won!
        </span>
      );
    }
    if (participation.status === 'completed') {
      return (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
          Completed
        </span>
      );
    }
    return (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
        Active
      </span>
    );
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border p-6">
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-purple-500 text-white rounded-lg">
              <CubeIcon className="h-6 w-6" />
            </div>
            <div>
              <h3 className="text-xl font-semibold text-gray-900">
                My Raffles
              </h3>
              <p className="text-sm text-gray-600">
                Your raffle participation history
              </p>
            </div>
          </div>
          
          <Button
            variant="secondary"
            onClick={() => navigate('/raffles')}
          >
            Browse More
          </Button>
        </div>

        {/* Filter tabs */}
        <div className="flex space-x-2">
          {[
            { key: 'active', label: 'Active', count: participations.filter(p => p.status === 'active').length },
            { key: 'completed', label: 'Completed', count: participations.filter(p => p.status === 'completed' || p.status === 'won').length },
            { key: 'all', label: 'All', count: participations.length },
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => setViewMode(tab.key as any)}
              className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
                viewMode === tab.key
                  ? 'bg-indigo-100 text-indigo-700 border border-indigo-200'
                  : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
              }`}
            >
              {tab.label} ({tab.count})
            </button>
          ))}
        </div>

        {/* Participation cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {filteredParticipations.map((participation) => (
            <div
              key={participation.id}
              className="border border-gray-200 rounded-lg overflow-hidden hover:shadow-md transition-shadow cursor-pointer"
              onClick={() => navigate(`/raffles/${participation.id}`)}
            >
              <div className="relative">
                <img
                  src={participation.imageUrl}
                  alt={participation.title}
                  className="w-full h-32 object-cover"
                />
                <div className="absolute top-2 right-2">
                  {getStatusBadge(participation)}
                </div>
                {participation.status === 'won' && (
                  <div className="absolute top-2 left-2 bg-green-500 text-white rounded-full p-2">
                    <TrophyIcon className="h-4 w-4" />
                  </div>
                )}
              </div>
              
              <div className="p-4">
                <div className="space-y-3">
                  <div>
                    <h4 className="font-medium text-gray-900 truncate">
                      {participation.title}
                    </h4>
                    <p className="text-sm text-gray-600">
                      {participation.boxesPurchased} box{participation.boxesPurchased !== 1 ? 'es' : ''} • ${participation.amountSpent} spent
                    </p>
                  </div>

                  {participation.status === 'active' && (
                    <div>
                      <div className="flex justify-between items-center mb-2">
                        <span className="text-sm text-gray-600">Progress</span>
                        <span className="text-sm font-medium">{participation.progress}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-indigo-600 h-2 rounded-full transition-all duration-300"
                          style={{ width: `${participation.progress}%` }}
                        />
                      </div>
                      <p className="text-xs text-gray-500 mt-1">
                        {Math.floor((participation.totalBoxes * participation.progress) / 100)} / {participation.totalBoxes} boxes sold
                      </p>
                    </div>
                  )}

                  <hr className="border-gray-200" />

                  <div className="flex justify-between items-center">
                    <div className="flex items-center space-x-1">
                      <UsersIcon className="h-4 w-4 text-gray-400" />
                      <span className="text-xs text-gray-500">
                        {participation.participants} participants
                      </span>
                    </div>
                    
                    <div className="flex space-x-1">
                      <button
                        className="p-1 text-gray-400 hover:text-gray-600 rounded"
                        title="View Details"
                        onClick={(e) => {
                          e.stopPropagation();
                          navigate(`/raffles/${participation.id}`);
                        }}
                      >
                        <EyeIcon className="h-4 w-4" />
                      </button>
                      <button
                        className="p-1 text-gray-400 hover:text-gray-600 rounded"
                        title="More Options"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <EllipsisVerticalIcon className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>

        {filteredParticipations.length === 0 && (
          <div className="text-center py-12">
            <CubeIcon className="h-16 w-16 text-gray-300 mx-auto mb-4" />
            <h4 className="text-lg font-medium text-gray-900 mb-2">
              No raffles found
            </h4>
            <p className="text-gray-600 mb-6">
              {viewMode === 'active' 
                ? "You're not currently participating in any active raffles."
                : "You haven't participated in any completed raffles yet."
              }
            </p>
            <Button
              onClick={() => navigate('/raffles')}
              className="bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700"
            >
              Browse Raffles
            </Button>
          </div>
        )}
      </div>
    </div>
  );
};

export default ParticipationHistory;