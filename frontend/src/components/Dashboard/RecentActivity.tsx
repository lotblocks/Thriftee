import React from 'react';
import {
  ClockIcon,
  CubeIcon,
  TrophyIcon,
  WalletIcon,
  UsersIcon,
  TrendingUpIcon,
  TruckIcon,
} from '@heroicons/react/24/outline';
import { useNavigate } from 'react-router-dom';

import { Button } from '../ui/Button';

interface ActivityItem {
  id: string;
  type: 'raffle_joined' | 'raffle_won' | 'raffle_completed' | 'credit_earned' | 'item_shipped' | 'milestone_reached';
  title: string;
  description: string;
  timestamp: string;
  metadata?: {
    raffleId?: string;
    amount?: number;
    itemName?: string;
    milestone?: string;
  };
}

const RecentActivity: React.FC = () => {
  const navigate = useNavigate();

  // Mock data - in real app, this would come from API
  const activities: ActivityItem[] = [
    {
      id: '1',
      type: 'raffle_joined',
      title: 'Joined iPhone 15 Pro Max Raffle',
      description: 'Purchased 5 boxes for $60',
      timestamp: '2024-01-18T15:30:00Z',
      metadata: {
        raffleId: '1',
        amount: 60,
        itemName: 'iPhone 15 Pro Max',
      },
    },
    {
      id: '2',
      type: 'credit_earned',
      title: 'Credits Earned',
      description: 'Received $45 in credits from MacBook Pro raffle',
      timestamp: '2024-01-17T09:20:00Z',
      metadata: {
        raffleId: '2',
        amount: 45,
      },
    },
    {
      id: '3',
      type: 'item_shipped',
      title: 'Nintendo Switch Shipped',
      description: 'Your winning item is on its way!',
      timestamp: '2024-01-16T14:45:00Z',
      metadata: {
        itemName: 'Nintendo Switch OLED',
      },
    },
    {
      id: '4',
      type: 'raffle_won',
      title: 'Congratulations! You Won!',
      description: 'Won Nintendo Switch OLED raffle',
      timestamp: '2024-01-15T18:00:00Z',
      metadata: {
        raffleId: '4',
        itemName: 'Nintendo Switch OLED',
      },
    },
    {
      id: '5',
      type: 'milestone_reached',
      title: 'Milestone Achieved',
      description: 'Reached 10 raffle participations',
      timestamp: '2024-01-14T12:00:00Z',
      metadata: {
        milestone: '10 Participations',
      },
    },
    {
      id: '6',
      type: 'raffle_completed',
      title: 'Sony Headphones Raffle Ended',
      description: 'Raffle completed, winner was selected',
      timestamp: '2024-01-13T20:30:00Z',
      metadata: {
        raffleId: '3',
        itemName: 'Sony WH-1000XM5',
      },
    },
  ];

  const getActivityIcon = (type: string) => {
    const iconClasses = "h-6 w-6";
    switch (type) {
      case 'raffle_joined': return <CubeIcon className={iconClasses} />;
      case 'raffle_won': return <TrophyIcon className={iconClasses} />;
      case 'raffle_completed': return <CubeIcon className={iconClasses} />;
      case 'credit_earned': return <WalletIcon className={iconClasses} />;
      case 'item_shipped': return <TruckIcon className={iconClasses} />;
      case 'milestone_reached': return <TrendingUpIcon className={iconClasses} />;
      default: return <ClockIcon className={iconClasses} />;
    }
  };

  const getActivityColor = (type: string) => {
    switch (type) {
      case 'raffle_joined': return 'bg-blue-500';
      case 'raffle_won': return 'bg-green-500';
      case 'raffle_completed': return 'bg-gray-500';
      case 'credit_earned': return 'bg-indigo-500';
      case 'item_shipped': return 'bg-yellow-500';
      case 'milestone_reached': return 'bg-purple-500';
      default: return 'bg-gray-500';
    }
  };

  const formatTimeAgo = (timestamp: string) => {
    const now = new Date();
    const date = new Date(timestamp);
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours}h ago`;
    
    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 7) return `${diffInDays}d ago`;
    
    return date.toLocaleDateString();
  };

  const handleActivityClick = (activity: ActivityItem) => {
    if (activity.metadata?.raffleId) {
      navigate(`/raffles/${activity.metadata.raffleId}`);
    }
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border p-6">
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-gray-500 text-white rounded-lg">
              <ClockIcon className="h-6 w-6" />
            </div>
            <div>
              <h3 className="text-xl font-semibold text-gray-900">
                Recent Activity
              </h3>
              <p className="text-sm text-gray-600">
                Your latest platform interactions
              </p>
            </div>
          </div>
        </div>

        {/* Activity list */}
        <div className="max-h-96 overflow-y-auto space-y-4">
          {activities.map((activity) => (
            <div
              key={activity.id}
              className={`flex items-start space-x-4 p-3 rounded-lg ${
                activity.metadata?.raffleId 
                  ? 'cursor-pointer hover:bg-gray-50 transition-colors' 
                  : ''
              }`}
              onClick={() => handleActivityClick(activity)}
            >
              <div className={`flex-shrink-0 w-10 h-10 rounded-full ${getActivityColor(activity.type)} text-white flex items-center justify-center`}>
                {getActivityIcon(activity.type)}
              </div>
              
              <div className="flex-1 min-w-0">
                <div className="flex justify-between items-start">
                  <h4 className="text-sm font-medium text-gray-900">
                    {activity.title}
                  </h4>
                  <span className="text-xs text-gray-500 flex-shrink-0 ml-2">
                    {formatTimeAgo(activity.timestamp)}
                  </span>
                </div>
                
                <p className="text-sm text-gray-600 mt-1">
                  {activity.description}
                </p>
                
                {/* Activity-specific metadata */}
                <div className="flex flex-wrap gap-2 mt-2">
                  {activity.metadata?.amount && (
                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                      ${activity.metadata.amount}
                    </span>
                  )}
                  
                  {activity.metadata?.milestone && (
                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800">
                      {activity.metadata.milestone}
                    </span>
                  )}
                  
                  {activity.type === 'raffle_won' && (
                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                      Winner!
                    </span>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>

        {activities.length === 0 && (
          <div className="text-center py-8">
            <ClockIcon className="h-16 w-16 text-gray-300 mx-auto mb-4" />
            <h4 className="text-lg font-medium text-gray-900 mb-2">
              No recent activity
            </h4>
            <p className="text-gray-600">
              Start participating in raffles to see your activity here!
            </p>
          </div>
        )}

        {/* View all button */}
        <Button
          variant="secondary"
          onClick={() => navigate('/activity')}
          className="w-full"
        >
          View All Activity
        </Button>
      </div>
    </div>
  );
};

export default RecentActivity;