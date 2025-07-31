import React, { useState } from 'react';
import {
  WalletIcon,
  PlusIcon,
  ArrowPathIcon,
  InformationCircleIcon,
  ExclamationTriangleIcon,
  ClockIcon,
  TagIcon,
} from '@heroicons/react/24/outline';
import { useNavigate } from 'react-router-dom';

import { Button } from '../ui/Button';
import { useAppSelector } from '../../store';

interface CreditBreakdown {
  type: 'general' | 'item_specific';
  amount: number;
  count: number;
  expiringSoon: number;
}

const CreditBalance: React.FC = () => {
  const navigate = useNavigate();
  const { balance } = useAppSelector(state => state.credit);
  const [isRefreshing, setIsRefreshing] = useState(false);

  // Mock credit breakdown - in real app, this would come from API
  const creditBreakdown: CreditBreakdown[] = [
    {
      type: 'general',
      amount: balance * 0.7,
      count: 12,
      expiringSoon: 2,
    },
    {
      type: 'item_specific',
      amount: balance * 0.3,
      count: 5,
      expiringSoon: 1,
    },
  ];

  const totalExpiringSoon = creditBreakdown.reduce((sum, item) => sum + item.expiringSoon, 0);
  const expiringAmount = creditBreakdown.reduce((sum, item) => 
    sum + (item.amount * (item.expiringSoon / item.count)), 0
  );

  const handleRefresh = async () => {
    setIsRefreshing(true);
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000));
    setIsRefreshing(false);
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border p-6">
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-blue-500 text-white rounded-lg">
              <WalletIcon className="h-6 w-6" />
            </div>
            <div>
              <h3 className="text-xl font-semibold text-gray-900">
                Credit Balance
              </h3>
              <p className="text-sm text-gray-600">
                Available for raffle purchases
              </p>
            </div>
          </div>
          
          <div className="flex items-center space-x-2">
            <button
              onClick={handleRefresh}
              disabled={isRefreshing}
              className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100 transition-colors"
              title="Refresh balance"
            >
              <ArrowPathIcon className={`h-5 w-5 ${isRefreshing ? 'animate-spin' : ''}`} />
            </button>
            <Button
              onClick={() => navigate('/credits')}
              className="inline-flex items-center bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700"
            >
              <PlusIcon className="h-4 w-4 mr-2" />
              Buy Credits
            </Button>
          </div>
        </div>

        {/* Main balance display */}
        <div className="text-center p-8 bg-gradient-to-br from-indigo-50 to-purple-50 border-2 border-indigo-200 rounded-lg">
          <div className="text-4xl font-bold text-indigo-600 mb-2">
            ${balance.toFixed(2)}
          </div>
          <div className="text-lg text-gray-600">
            Total Available Credits
          </div>
        </div>

        {/* Expiring credits warning */}
        {totalExpiringSoon > 0 && (
          <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
            <div className="flex items-start space-x-3">
              <ClockIcon className="h-5 w-5 text-yellow-600 mt-0.5 flex-shrink-0" />
              <div className="flex-1">
                <p className="text-sm text-yellow-800">
                  <span className="font-semibold">${expiringAmount.toFixed(2)}</span> in credits expire within 30 days.
                  Use them soon or redeem for free items!
                </p>
              </div>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => navigate('/credits')}
                className="text-yellow-800 border-yellow-300 hover:bg-yellow-100"
              >
                View Details
              </Button>
            </div>
          </div>
        )}

        {/* Credit breakdown */}
        <div>
          <h4 className="text-lg font-semibold text-gray-900 mb-4">
            Credit Breakdown
          </h4>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {creditBreakdown.map((item, index) => (
              <div key={index} className="border border-gray-200 rounded-lg p-4">
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <h5 className="font-medium text-gray-900">
                      {item.type === 'general' ? 'General Credits' : 'Item-Specific Credits'}
                    </h5>
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      item.type === 'general' 
                        ? 'bg-blue-100 text-blue-800' 
                        : 'bg-purple-100 text-purple-800'
                    }`}>
                      {item.count} credits
                    </span>
                  </div>
                  
                  <div className="text-2xl font-semibold text-indigo-600">
                    ${item.amount.toFixed(2)}
                  </div>
                  
                  <div>
                    <p className="text-sm text-gray-600 mb-2">
                      {item.type === 'general' 
                        ? 'Can be used for any raffle purchase'
                        : 'Tied to specific items or categories'
                      }
                    </p>
                    
                    {item.expiringSoon > 0 && (
                      <div className="flex items-center space-x-1">
                        <ExclamationTriangleIcon className="h-4 w-4 text-yellow-500" />
                        <span className="text-xs text-yellow-600">
                          {item.expiringSoon} expiring soon
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Usage tips */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex items-start space-x-3">
            <InformationCircleIcon className="h-5 w-5 text-blue-600 mt-0.5 flex-shrink-0" />
            <div>
              <h5 className="font-medium text-blue-900 mb-2">
                Credit Usage Tips
              </h5>
              <ul className="text-sm text-blue-800 space-y-1">
                <li>• General credits can be used for any raffle purchase</li>
                <li>• Item-specific credits offer better value but have restrictions</li>
                <li>• Expiring credits can be redeemed for free items in our marketplace</li>
                <li>• Credits never lose value - you're guaranteed to get your money's worth!</li>
              </ul>
            </div>
          </div>
        </div>

        {/* Quick actions */}
        <div className="flex justify-center space-x-4">
          <Button
            variant="secondary"
            onClick={() => navigate('/credits/redeem')}
            className="inline-flex items-center"
          >
            <TagIcon className="h-4 w-4 mr-2" />
            Redeem Credits
          </Button>
          <Button
            variant="secondary"
            onClick={() => navigate('/credits/history')}
          >
            View History
          </Button>
        </div>
      </div>
    </div>
  );
};

export default CreditBalance;