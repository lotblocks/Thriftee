import React, { useEffect, useState, useRef } from 'react';
import { Outlet, useLocation } from 'react-router-dom';
import {
  HomeIcon,
  Squares2X2Icon,
  WalletIcon,
  UserCircleIcon,
  Bars3Icon,
  XMarkIcon,
  BellIcon,
  MagnifyingGlassIcon,
} from '@heroicons/react/24/outline';
import {
  HomeIcon as HomeSolidIcon,
  Squares2X2Icon as SquaresSolidIcon,
  WalletIcon as WalletSolidIcon,
  UserCircleIcon as UserSolidIcon,
} from '@heroicons/react/24/solid';
import { Link } from 'react-router-dom';
import { useAppSelector } from '../../store';
import { useMobileInteractions, usePullToRefresh } from '../../hooks/useTouchGestures';
import { getDeviceInfo, getSafeAreaInsets } from '../../utils/mobileUtils';
import { Button } from '../ui/Button';

interface MobileLayoutProps {
  children?: React.ReactNode;
}

const MobileLayout: React.FC<MobileLayoutProps> = ({ children }) => {
  const location = useLocation();
  const { user } = useAppSelector(state => state.auth);
  const { balance } = useAppSelector(state => state.credit);
  const [showMobileMenu, setShowMobileMenu] = useState(false);
  const [showNotifications, setShowNotifications] = useState(false);
  const [deviceInfo] = useState(() => getDeviceInfo());
  const [safeAreaInsets] = useState(() => getSafeAreaInsets());
  const { isScrolling, scrollDirection } = useMobileInteractions();
  
  // Pull to refresh functionality
  const { containerRef, isPulling, pullDistance, isRefreshing } = usePullToRefresh(
    async () => {
      // Refresh current page data
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  );

  // Navigation items
  const navigationItems = [
    {
      name: 'Home',
      href: '/',
      icon: HomeIcon,
      activeIcon: HomeSolidIcon,
      active: location.pathname === '/',
    },
    {
      name: 'Raffles',
      href: '/raffles',
      icon: Squares2X2Icon,
      activeIcon: SquaresSolidIcon,
      active: location.pathname.startsWith('/raffles'),
    },
    {
      name: 'Wallet',
      href: '/wallet',
      icon: WalletIcon,
      activeIcon: WalletSolidIcon,
      active: location.pathname.startsWith('/wallet'),
    },
    {
      name: 'Profile',
      href: '/profile',
      icon: UserCircleIcon,
      activeIcon: UserSolidIcon,
      active: location.pathname.startsWith('/profile'),
    },
  ];

  // Close mobile menu when route changes
  useEffect(() => {
    setShowMobileMenu(false);
    setShowNotifications(false);
  }, [location.pathname]);

  // Handle safe area insets
  useEffect(() => {
    document.documentElement.style.setProperty('--safe-area-top', `${safeAreaInsets.top}px`);
    document.documentElement.style.setProperty('--safe-area-bottom', `${safeAreaInsets.bottom}px`);
  }, [safeAreaInsets]);

  const handleRefresh = async () => {
    // Implement refresh logic based on current route
    window.location.reload();
  };

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col" style={{ paddingTop: safeAreaInsets.top }}>
      {/* Pull to refresh indicator */}
      {isPulling && (
        <div 
          className="fixed top-0 left-0 right-0 z-50 bg-indigo-500 text-white text-center py-2 transition-transform duration-200"
          style={{ 
            transform: `translateY(${Math.min(pullDistance - 80, 0)}px)`,
            paddingTop: safeAreaInsets.top 
          }}
        >
          {isRefreshing ? (
            <div className="flex items-center justify-center space-x-2">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
              <span className="text-sm">Refreshing...</span>
            </div>
          ) : (
            <span className="text-sm">
              {pullDistance >= 80 ? 'Release to refresh' : 'Pull to refresh'}
            </span>
          )}
        </div>
      )}

      {/* Mobile Header */}
      <header 
        className={`sticky top-0 z-40 bg-white border-b border-gray-200 transition-transform duration-300 ${
          isScrolling && scrollDirection === 'down' ? '-translate-y-full' : 'translate-y-0'
        }`}
        style={{ paddingTop: safeAreaInsets.top }}
      >
        <div className="flex items-center justify-between px-4 py-3">
          {/* Left side - Menu button */}
          <button
            onClick={() => setShowMobileMenu(true)}
            className="p-2 text-gray-600 hover:text-gray-900 rounded-lg hover:bg-gray-100"
          >
            <Bars3Icon className="h-6 w-6" />
          </button>

          {/* Center - Logo/Title */}
          <div className="flex-1 flex justify-center">
            <Link to="/" className="text-xl font-bold text-indigo-600">
              RafflePlatform
            </Link>
          </div>

          {/* Right side - Actions */}
          <div className="flex items-center space-x-2">
            <button className="p-2 text-gray-600 hover:text-gray-900 rounded-lg hover:bg-gray-100">
              <MagnifyingGlassIcon className="h-5 w-5" />
            </button>
            <button 
              onClick={() => setShowNotifications(true)}
              className="p-2 text-gray-600 hover:text-gray-900 rounded-lg hover:bg-gray-100 relative"
            >
              <BellIcon className="h-5 w-5" />
              {/* Notification badge */}
              <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full h-4 w-4 flex items-center justify-center">
                3
              </span>
            </button>
          </div>
        </div>

        {/* Credit balance bar (if user is logged in) */}
        {user && (
          <div className="px-4 pb-3">
            <div className="flex items-center justify-between p-2 bg-indigo-50 rounded-lg">
              <span className="text-sm text-indigo-700">Credit Balance</span>
              <span className="text-sm font-semibold text-indigo-900">
                ${balance.toFixed(2)}
              </span>
            </div>
          </div>
        )}
      </header>

      {/* Main Content */}
      <main ref={containerRef} className="flex-1 overflow-auto">
        {children || <Outlet />}
      </main>

      {/* Bottom Navigation */}
      <nav 
        className="sticky bottom-0 z-40 bg-white border-t border-gray-200"
        style={{ paddingBottom: safeAreaInsets.bottom }}
      >
        <div className="flex items-center justify-around py-2">
          {navigationItems.map((item) => {
            const Icon = item.active ? item.activeIcon : item.icon;
            return (
              <Link
                key={item.name}
                to={item.href}
                className={`flex flex-col items-center py-2 px-3 rounded-lg transition-colors ${
                  item.active
                    ? 'text-indigo-600 bg-indigo-50'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
                }`}
              >
                <Icon className="h-6 w-6" />
                <span className="text-xs mt-1 font-medium">{item.name}</span>
              </Link>
            );
          })}
        </div>
      </nav>

      {/* Mobile Menu Overlay */}
      {showMobileMenu && (
        <>
          <div
            className="fixed inset-0 bg-black bg-opacity-50 z-50"
            onClick={() => setShowMobileMenu(false)}
          />
          <div className="fixed top-0 left-0 h-full w-80 max-w-[85vw] bg-white z-50 transform transition-transform duration-300 ease-in-out overflow-y-auto">
            {/* Menu Header */}
            <div 
              className="flex items-center justify-between p-4 border-b border-gray-200"
              style={{ paddingTop: safeAreaInsets.top + 16 }}
            >
              <h2 className="text-lg font-semibold text-gray-900">Menu</h2>
              <button
                onClick={() => setShowMobileMenu(false)}
                className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100"
              >
                <XMarkIcon className="h-5 w-5" />
              </button>
            </div>

            {/* User Info */}
            {user ? (
              <div className="p-4 border-b border-gray-200">
                <div className="flex items-center space-x-3">
                  <div className="w-12 h-12 bg-indigo-100 rounded-full flex items-center justify-center">
                    <UserCircleIcon className="h-8 w-8 text-indigo-600" />
                  </div>
                  <div>
                    <p className="text-sm font-medium text-gray-900">{user.email}</p>
                    <p className="text-xs text-gray-600">Credits: ${balance.toFixed(2)}</p>
                  </div>
                </div>
              </div>
            ) : (
              <div className="p-4 border-b border-gray-200">
                <div className="space-y-2">
                  <Button
                    as={Link}
                    to="/auth/login"
                    className="w-full"
                    variant="primary"
                  >
                    Sign In
                  </Button>
                  <Button
                    as={Link}
                    to="/auth/register"
                    className="w-full"
                    variant="outline"
                  >
                    Sign Up
                  </Button>
                </div>
              </div>
            )}

            {/* Menu Items */}
            <div className="p-4 space-y-2">
              {navigationItems.map((item) => {
                const Icon = item.icon;
                return (
                  <Link
                    key={item.name}
                    to={item.href}
                    className={`flex items-center space-x-3 p-3 rounded-lg transition-colors ${
                      item.active
                        ? 'bg-indigo-50 text-indigo-600'
                        : 'text-gray-700 hover:bg-gray-100'
                    }`}
                  >
                    <Icon className="h-5 w-5" />
                    <span className="font-medium">{item.name}</span>
                  </Link>
                );
              })}
              
              {user && (
                <>
                  <Link
                    to="/dashboard"
                    className="flex items-center space-x-3 p-3 rounded-lg text-gray-700 hover:bg-gray-100 transition-colors"
                  >
                    <Squares2X2Icon className="h-5 w-5" />
                    <span className="font-medium">Dashboard</span>
                  </Link>
                  <Link
                    to="/settings"
                    className="flex items-center space-x-3 p-3 rounded-lg text-gray-700 hover:bg-gray-100 transition-colors"
                  >
                    <UserCircleIcon className="h-5 w-5" />
                    <span className="font-medium">Settings</span>
                  </Link>
                </>
              )}
            </div>

            {/* Footer */}
            <div className="p-4 border-t border-gray-200 mt-auto">
              <p className="text-xs text-gray-500 text-center">
                Version 1.0.0
              </p>
            </div>
          </div>
        </>
      )}

      {/* Notifications Panel */}
      {showNotifications && (
        <>
          <div
            className="fixed inset-0 bg-black bg-opacity-50 z-50"
            onClick={() => setShowNotifications(false)}
          />
          <div className="fixed top-0 right-0 h-full w-80 max-w-[85vw] bg-white z-50 transform transition-transform duration-300 ease-in-out overflow-y-auto">
            {/* Notifications Header */}
            <div 
              className="flex items-center justify-between p-4 border-b border-gray-200"
              style={{ paddingTop: safeAreaInsets.top + 16 }}
            >
              <h2 className="text-lg font-semibold text-gray-900">Notifications</h2>
              <button
                onClick={() => setShowNotifications(false)}
                className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100"
              >
                <XMarkIcon className="h-5 w-5" />
              </button>
            </div>

            {/* Notifications List */}
            <div className="p-4 space-y-3">
              <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
                <p className="text-sm font-medium text-blue-900">Raffle Update</p>
                <p className="text-xs text-blue-700 mt-1">
                  Your raffle "iPhone 15 Pro" is 80% full!
                </p>
                <p className="text-xs text-blue-600 mt-2">2 minutes ago</p>
              </div>
              
              <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
                <p className="text-sm font-medium text-green-900">Winner Selected!</p>
                <p className="text-xs text-green-700 mt-1">
                  Congratulations! You won the "AirPods Pro" raffle!
                </p>
                <p className="text-xs text-green-600 mt-2">1 hour ago</p>
              </div>
              
              <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
                <p className="text-sm font-medium text-yellow-900">Credits Expiring</p>
                <p className="text-xs text-yellow-700 mt-1">
                  $25.00 in credits expire in 3 days
                </p>
                <p className="text-xs text-yellow-600 mt-2">1 day ago</p>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export default MobileLayout;