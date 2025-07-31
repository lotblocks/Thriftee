import React from 'react';
import {
  WalletIcon,
  CubeIcon,
  TrophyIcon,
  TrendingUpIcon,
  TagIcon,
  ChartBarIcon,
} from '@heroicons/react/24/outline';

import { useAppSelector } from '../../store';

interface StatCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: React.ReactElement;
  color: 'blue' | 'purple' | 'green' | 'yellow' | 'red' | 'indigo';
  trend?: {
    value: number;
    isPositive: boolean;
  };
  progress?: number;
}

const StatCard: React.FC<StatCardProps> = ({
  title,
  value,
  subtitle,
  icon,
  color,
  trend,
  progress,
}) => {
  const colorClasses = {
    blue: {
      bg: 'bg-blue-500',
      text: 'text-blue-600',
      lightBg: 'bg-blue-50',
      border: 'border-blue-200',
    },
    purple: {
      bg: 'bg-purple-500',
      text: 'text-purple-600',
      lightBg: 'bg-purple-50',
      border: 'border-purple-200',
    },
    green: {
      bg: 'bg-green-500',
      text: 'text-green-600',
      lightBg: 'bg-green-50',
      border: 'border-green-200',
    },
    yellow: {
      bg: 'bg-yellow-500',
      text: 'text-yellow-600',
      lightBg: 'bg-yellow-50',
      border: 'border-yellow-200',
    },
    red: {
      bg: 'bg-red-500',
      text: 'text-red-600',
      lightBg: 'bg-red-50',
      border: 'border-red-200',
    },
    indigo: {
      bg: 'bg-indigo-500',
      text: 'text-indigo-600',
      lightBg: 'bg-indigo-50',
      border: 'border-indigo-200',
    },
  };

  const colors = colorClasses[color];

  return (
    <div className="bg-white rounded-lg shadow-sm border p-6 h-full">
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div className={`p-2 rounded-lg ${colors.bg} text-white`}>
            {React.cloneElement(icon, { className: 'h-6 w-6' })}
          </div>
          {trend && (
            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
              trend.isPositive 
                ? 'bg-green-100 text-green-800' 
                : 'bg-red-100 text-red-800'
            }`}>
              {trend.isPositive ? '+' : ''}{trend.value}%
            </span>
          )}
        </div>

        <div>
          <div className={`text-2xl font-bold ${colors.text}`}>
            {value}
          </div>
          <div className="text-sm text-gray-600 mb-1">
            {title}
          </div>
          {subtitle && (
            <div className="text-xs text-gray-500">
              {subtitle}
            </div>
          )}
        </div>

        {progress !== undefined && (
          <div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className={`h-2 rounded-full ${colors.bg}`}
                style={{ width: `${progress}%` }}
              />
            </div>
            <div className="text-xs text-gray-500 mt-1">
              {progress.toFixed(0)}% of monthly goal
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

const DashboardStats: React.FC = () => {
  const { balance } = useAppSelector(state => state.credit);
  
  // Mock data - in real app, this would come from API
  const stats = [
    {
      title: 'Credit Balance',
      value: `$${balance.toFixed(2)}`,
      subtitle: 'Available for purchases',
      icon: <WalletIcon />,
      color: 'blue' as const,
      trend: { value: 12.5, isPositive: true },
    },
    {
      title: 'Active Raffles',
      value: 3,
      subtitle: 'Currently participating',
      icon: <CubeIcon />,
      color: 'purple' as const,
    },
    {
      title: 'Items Won',
      value: 7,
      subtitle: 'Total wins this year',
      icon: <TrophyIcon />,
      color: 'green' as const,
      trend: { value: 8.3, isPositive: true },
    },
    {
      title: 'Win Rate',
      value: '23%',
      subtitle: 'Success rate',
      icon: <TrendingUpIcon />,
      color: 'indigo' as const,
      progress: 23,
    },
    {
      title: 'Total Spent',
      value: '$1,247',
      subtitle: 'This month',
      icon: <TagIcon />,
      color: 'yellow' as const,
      trend: { value: -5.2, isPositive: false },
    },
    {
      title: 'Savings',
      value: '$892',
      subtitle: 'Compared to retail',
      icon: <ChartBarIcon />,
      color: 'green' as const,
      trend: { value: 15.7, isPositive: true },
    },
  ];

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-4">
      {stats.map((stat, index) => (
        <StatCard key={index} {...stat} />
      ))}
    </div>
  );
};

export default DashboardStats;