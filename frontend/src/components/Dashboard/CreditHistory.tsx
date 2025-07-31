import React, { useState } from 'react';
import {
  ClockIcon,
  PlusIcon,
  MinusIcon,
  ArrowPathIcon,
  FunnelIcon,
  TrendingUpIcon,
  TrendingDownIcon,
  WalletIcon,
  CubeIcon,
  GiftIcon,
} from '@heroicons/react/24/outline';
import { useNavigate } from 'react-router-dom';

import { Button } from '../ui/Button';

interface CreditTransaction {
  id: string;
  type: 'earned' | 'spent' | 'expired' | 'refunded' | 'purchased';
  amount: number;
  description: string;
  date: string;
  raffleId?: string;
  paymentId?: string;
  status: 'completed' | 'pending' | 'failed';
}

const CreditHistory: React.FC = () => {
  const navigate = useNavigate();
  const [showFilterMenu, setShowFilterMenu] = useState(false);
  const [selectedFilter, setSelectedFilter] = useState<'all' | 'earned' | 'spent' | 'purchased'>('all');

  // Mock data - in real app, this would come from API
  const transactions: CreditTransaction[] = [
    {
      id: '1',
      type: 'purchased',
      amount: 100,
      description: 'Credit purchase via Stripe',
      date: '2024-01-18T10:30:00Z',
      paymentId: 'pay_123',
      status: 'completed',
    },
    {
      id: '2',
      type: 'spent',
      amount: -60,
      description: 'iPhone 15 Pro Max raffle - 5 boxes',
      date: '2024-01-17T15:45:00Z',
      raffleId: '1',
      status: 'completed',
    },
    {
      id: '3',
      type: 'earned',
      amount: 45,
      description: 'MacBook Pro raffle - non-winner credits',
      date: '2024-01-16T09:20:00Z',
      raffleId: '2',
      status: 'completed',
    },
    {
      id: '4',
      type: 'spent',
      amount: -16,
      description: 'Sony WH-1000XM5 raffle - 2 boxes',
      date: '2024-01-15T14:10:00Z',
      raffleId: '3',
      status: 'completed',
    },
    {
      id: '5',
      type: 'purchased',
      amount: 50,
      description: 'Credit purchase via PayPal',
      date: '2024-01-14T11:00:00Z',
      paymentId: 'pay_456',
      status: 'completed',
    },
    {
      id: '6',
      type: 'expired',
      amount: -25,
      description: 'Item-specific credits expired',
      date: '2024-01-13T00:00:00Z',
      status: 'completed',
    },
  ];

  const filteredTransactions = transactions.filter(t => {
    if (selectedFilter === 'all') return true;
    return t.type === selectedFilter;
  });

  const getTransactionIcon = (type: string) => {
    switch (type) {
      case 'earned': return <TrendingUpIcon className="h-5 w-5 text-green-600" />;
      case 'spent': return <TrendingDownIcon className="h-5 w-5 text-red-600" />;
      case 'purchased': return <PlusIcon className="h-5 w-5 text-blue-600" />;
      case 'expired': return <ClockIcon className="h-5 w-5 text-yellow-600" />;
      case 'refunded': return <ArrowPathIcon className="h-5 w-5 text-indigo-600" />;
      default: return <WalletIcon className="h-5 w-5 text-gray-600" />;
    }
  };

  const getAmountColor = (type: string) => {
    switch (type) {
      case 'earned': return 'text-green-600';
      case 'spent': return 'text-red-600';
      case 'purchased': return 'text-blue-600';
      case 'expired': return 'text-yellow-600';
      case 'refunded': return 'text-indigo-600';
      default: return 'text-gray-900';
    }
  };

  const formatAmount = (amount: number) => {
    const sign = amount >= 0 ? '+' : '';
    return `${sign}$${Math.abs(amount).toFixed(2)}`;
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getTypeBadge = (type: string) => {
    const badges = {
      earned: 'bg-green-100 text-green-800',
      spent: 'bg-red-100 text-red-800',
      purchased: 'bg-blue-100 text-blue-800',
      expired: 'bg-yellow-100 text-yellow-800',
      refunded: 'bg-indigo-100 text-indigo-800',
    };
    
    return (
      <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${badges[type as keyof typeof badges] || 'bg-gray-100 text-gray-800'}`}>
        {type}
      </span>
    );
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border p-6">
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-indigo-500 text-white rounded-lg">
              <ClockIcon className="h-6 w-6" />
            </div>
            <div>
              <h3 className="text-xl font-semibold text-gray-900">
                Credit History
              </h3>
              <p className="text-sm text-gray-600">
                Recent credit transactions
              </p>
            </div>
          </div>
          
          <div className="relative">
            <button
              onClick={() => setShowFilterMenu(!showFilterMenu)}
              className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100 transition-colors"
              title="Filter transactions"
            >
              <FunnelIcon className="h-5 w-5" />
            </button>
            
            {showFilterMenu && (
              <div className="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg border z-10">
                <div className="py-1">
                  {[
                    { key: 'all', label: 'All Transactions', icon: WalletIcon },
                    { key: 'earned', label: 'Credits Earned', icon: TrendingUpIcon },
                    { key: 'spent', label: 'Credits Spent', icon: TrendingDownIcon },
                    { key: 'purchased', label: 'Credits Purchased', icon: PlusIcon },
                  ].map((filter) => (
                    <button
                      key={filter.key}
                      onClick={() => {
                        setSelectedFilter(filter.key as any);
                        setShowFilterMenu(false);
                      }}
                      className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                    >
                      <filter.icon className="h-4 w-4 mr-3" />
                      {filter.label}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Filter indicator */}
        {selectedFilter !== 'all' && (
          <div>
            <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800">
              Showing: {selectedFilter}
              <button
                onClick={() => setSelectedFilter('all')}
                className="ml-2 text-blue-600 hover:text-blue-800"
              >
                ×
              </button>
            </span>
          </div>
        )}

        {/* Transaction list */}
        <div className="max-h-96 overflow-y-auto space-y-3">
          {filteredTransactions.map((transaction) => (
            <div
              key={transaction.id}
              className={`flex items-center space-x-4 p-3 rounded-lg border ${
                transaction.raffleId 
                  ? 'cursor-pointer hover:bg-gray-50 transition-colors' 
                  : ''
              }`}
              onClick={() => {
                if (transaction.raffleId) {
                  navigate(`/raffles/${transaction.raffleId}`);
                }
              }}
            >
              <div className="flex-shrink-0">
                {getTransactionIcon(transaction.type)}
              </div>
              
              <div className="flex-1 min-w-0">
                <div className="flex justify-between items-start">
                  <p className="text-sm font-medium text-gray-900 truncate">
                    {transaction.description}
                  </p>
                  <p className={`text-sm font-semibold ${getAmountColor(transaction.type)}`}>
                    {formatAmount(transaction.amount)}
                  </p>
                </div>
                
                <div className="flex justify-between items-center mt-1">
                  <p className="text-xs text-gray-500">
                    {formatDate(transaction.date)}
                  </p>
                  <div className="flex items-center space-x-2">
                    {getTypeBadge(transaction.type)}
                    {transaction.status === 'pending' && (
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-yellow-100 text-yellow-800">
                        Pending
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>

        {filteredTransactions.length === 0 && (
          <div className="text-center py-8">
            <ClockIcon className="h-16 w-16 text-gray-300 mx-auto mb-4" />
            <h4 className="text-lg font-medium text-gray-900 mb-2">
              No transactions found
            </h4>
            <p className="text-gray-600">
              {selectedFilter === 'all' 
                ? "You haven't made any credit transactions yet."
                : `No ${selectedFilter} transactions found.`
              }
            </p>
          </div>
        )}

        {/* View all button */}
        <Button
          variant="secondary"
          onClick={() => navigate('/credits/history')}
          className="w-full"
        >
          View Full History
        </Button>
      </div>
    </div>
  );
};

export default CreditHistory;