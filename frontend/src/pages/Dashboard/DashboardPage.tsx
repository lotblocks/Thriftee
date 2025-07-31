import React, { useEffect, useState } from 'react';
import {
  ArrowPathIcon,
  CubeIcon,
  PlusIcon,
  WalletIcon,
  ClockIcon,
} from '@heroicons/react/24/outline';
import { useNavigate } from 'react-router-dom';

import CreditBalance from '../../components/Dashboard/CreditBalance';
import CreditHistory from '../../components/Dashboard/CreditHistory';
import ParticipationHistory from '../../components/Dashboard/ParticipationHistory';
import WinningsDisplay from '../../components/Dashboard/WinningsDisplay';
import DashboardStats from '../../components/Dashboard/DashboardStats';
import RecentActivity from '../../components/Dashboard/RecentActivity';
import { LoadingSpinner } from '../../components/ui/LoadingSpinner';
import { Button } from '../../components/ui/Button';
import { useAppSelector } from '../../store';

const DashboardPage: React.FC = () => {
  const navigate = useNavigate();
  const { user } = useAppSelector(state => state.auth);
  const { balance } = useAppSelector(state => state.credit);
  const [isLoading, setIsLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  // Simulate loading data
  useEffect(() => {
    const loadDashboardData = async () => {
      setIsLoading(true);
      // Simulate API calls
      await new Promise(resolve => setTimeout(resolve, 1000));
      setIsLoading(false);
    };

    loadDashboardData();
  }, []);

  const handleRefresh = async () => {
    setRefreshing(true);
    // Simulate refresh
    await new Promise(resolve => setTimeout(resolve, 500));
    setRefreshing(false);
  };

  if (isLoading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="flex items-center justify-center min-h-[400px]">
          <LoadingSpinner size="lg" />
          <span className="ml-3 text-gray-600">Loading your dashboard...</span>
        </div>
      </div>
    );
  }

  const userName = user?.username || user?.email?.split('@')[0] || 'User';

  return (
    <div className="container mx-auto px-4 py-8">
      {/* Header */}
      <div className="mb-8">
        <div className="flex justify-between items-center mb-6">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">
              Welcome back, {userName}!
            </h1>
            <p className="text-gray-600">
              Here's what's happening with your account
            </p>
          </div>
          
          <div className="flex space-x-3">
            <Button
              variant="secondary"
              onClick={handleRefresh}
              disabled={refreshing}
              className="inline-flex items-center"
            >
              <ArrowPathIcon className={`h-4 w-4 mr-2 ${refreshing ? 'animate-spin' : ''}`} />
              {refreshing ? 'Refreshing...' : 'Refresh'}
            </Button>
            <Button
              onClick={() => navigate('/raffles')}
              className="inline-flex items-center bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700"
            >
              <CubeIcon className="h-4 w-4 mr-2" />
              Browse Raffles
            </Button>
          </div>
        </div>

        {/* Quick stats cards */}
        <DashboardStats />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left column - Main content */}
        <div className="lg:col-span-2 space-y-6">
          {/* Credit balance overview */}
          <CreditBalance />

          {/* Recent participation */}
          <ParticipationHistory />

          {/* Recent activity feed */}
          <RecentActivity />
        </div>

        {/* Right column - Sidebar */}
        <div className="space-y-6">
          {/* Winnings display */}
          <WinningsDisplay />

          {/* Credit history */}
          <CreditHistory />

          {/* Quick actions */}
          <div className="bg-white rounded-lg shadow-sm border p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Quick Actions
            </h3>
            <div className="space-y-3">
              <Button
                variant="secondary"
                onClick={() => navigate('/credits')}
                className="w-full justify-start"
              >
                <PlusIcon className="h-4 w-4 mr-2" />
                Buy Credits
              </Button>
              <Button
                variant="secondary"
                onClick={() => navigate('/wallet')}
                className="w-full justify-start"
              >
                <WalletIcon className="h-4 w-4 mr-2" />
                Manage Wallet
              </Button>
              <Button
                variant="secondary"
                onClick={() => navigate('/payments')}
                className="w-full justify-start"
              >
                <ClockIcon className="h-4 w-4 mr-2" />
                Payment History
              </Button>
            </div>
          </div>

          {/* Account status */}
          <div className="bg-white rounded-lg shadow-sm border p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Account Status
            </h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600">Email Verified</span>
                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                  Verified
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600">2FA Enabled</span>
                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                  Disabled
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600">Account Level</span>
                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                  Standard
                </span>
              </div>
              <button
                onClick={() => navigate('/profile')}
                className="text-sm text-indigo-600 hover:text-indigo-800 font-medium"
              >
                Manage Account Settings
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DashboardPage;