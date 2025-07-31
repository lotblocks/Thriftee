import React, { useState, useEffect } from 'react';
import {
  BellIcon,
  BellSlashIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  InformationCircleIcon,
} from '@heroicons/react/24/outline';
import { usePWA } from '../../services/pwaService';
import { Button } from '../ui/Button';

interface NotificationSetting {
  id: string;
  title: string;
  description: string;
  enabled: boolean;
  category: 'raffle' | 'account' | 'system';
}

const PushNotificationSettings: React.FC = () => {
  const { 
    notificationPermission, 
    requestNotifications, 
    subscribeToPush, 
    unsubscribeFromPush 
  } = usePWA();
  
  const [isLoading, setIsLoading] = useState(false);
  const [subscriptionStatus, setSubscriptionStatus] = useState<'unknown' | 'subscribed' | 'unsubscribed'>('unknown');
  const [settings, setSettings] = useState<NotificationSetting[]>([
    {
      id: 'raffle_winner',
      title: 'Raffle Winners',
      description: 'Get notified when you win a raffle',
      enabled: true,
      category: 'raffle',
    },
    {
      id: 'raffle_full',
      title: 'Raffle Full',
      description: 'Get notified when raffles you\'re watching are full',
      enabled: true,
      category: 'raffle',
    },
    {
      id: 'new_raffles',
      title: 'New Raffles',
      description: 'Get notified about new raffles in your favorite categories',
      enabled: false,
      category: 'raffle',
    },
    {
      id: 'credit_expiring',
      title: 'Credits Expiring',
      description: 'Get reminded when your credits are about to expire',
      enabled: true,
      category: 'account',
    },
    {
      id: 'purchase_confirmation',
      title: 'Purchase Confirmations',
      description: 'Get notified when your box purchases are confirmed',
      enabled: true,
      category: 'account',
    },
    {
      id: 'system_updates',
      title: 'System Updates',
      description: 'Get notified about important platform updates',
      enabled: false,
      category: 'system',
    },
  ]);

  useEffect(() => {
    // Check current subscription status
    checkSubscriptionStatus();
  }, []);

  const checkSubscriptionStatus = async () => {
    // This would typically check with your backend
    // For now, we'll simulate based on permission
    if (notificationPermission.granted) {
      setSubscriptionStatus('subscribed');
    } else {
      setSubscriptionStatus('unsubscribed');
    }
  };

  const handleEnableNotifications = async () => {
    setIsLoading(true);
    
    try {
      const granted = await requestNotifications();
      if (granted) {
        // Subscribe to push notifications
        const subscription = await subscribeToPush(process.env.REACT_APP_VAPID_PUBLIC_KEY || '');
        if (subscription) {
          // Send subscription to backend
          await sendSubscriptionToBackend(subscription);
          setSubscriptionStatus('subscribed');
        }
      }
    } catch (error) {
      console.error('Failed to enable notifications:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDisableNotifications = async () => {
    setIsLoading(true);
    
    try {
      await unsubscribeFromPush();
      // Remove subscription from backend
      await removeSubscriptionFromBackend();
      setSubscriptionStatus('unsubscribed');
    } catch (error) {
      console.error('Failed to disable notifications:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSettingChange = (settingId: string, enabled: boolean) => {
    setSettings(prev => 
      prev.map(setting => 
        setting.id === settingId 
          ? { ...setting, enabled }
          : setting
      )
    );
    
    // Save to backend
    saveNotificationSettings(settingId, enabled);
  };

  const sendSubscriptionToBackend = async (subscription: any) => {
    try {
      await fetch('/api/notifications/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify(subscription),
      });
    } catch (error) {
      console.error('Failed to send subscription to backend:', error);
    }
  };

  const removeSubscriptionFromBackend = async () => {
    try {
      await fetch('/api/notifications/unsubscribe', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });
    } catch (error) {
      console.error('Failed to remove subscription from backend:', error);
    }
  };

  const saveNotificationSettings = async (settingId: string, enabled: boolean) => {
    try {
      await fetch('/api/notifications/settings', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify({ [settingId]: enabled }),
      });
    } catch (error) {
      console.error('Failed to save notification settings:', error);
    }
  };

  const getPermissionStatus = () => {
    if (notificationPermission.granted) {
      return {
        icon: CheckCircleIcon,
        text: 'Notifications Enabled',
        color: 'text-green-600',
        bgColor: 'bg-green-100',
      };
    } else if (notificationPermission.denied) {
      return {
        icon: ExclamationTriangleIcon,
        text: 'Notifications Blocked',
        color: 'text-red-600',
        bgColor: 'bg-red-100',
      };
    } else {
      return {
        icon: InformationCircleIcon,
        text: 'Notifications Not Set',
        color: 'text-yellow-600',
        bgColor: 'bg-yellow-100',
      };
    }
  };

  const permissionStatus = getPermissionStatus();
  const StatusIcon = permissionStatus.icon;

  const groupedSettings = settings.reduce((acc, setting) => {
    if (!acc[setting.category]) {
      acc[setting.category] = [];
    }
    acc[setting.category].push(setting);
    return acc;
  }, {} as Record<string, NotificationSetting[]>);

  const categoryTitles = {
    raffle: 'Raffle Notifications',
    account: 'Account Notifications',
    system: 'System Notifications',
  };

  return (
    <div className="space-y-6">
      {/* Permission Status */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">
            Push Notifications
          </h3>
          <div className={`flex items-center space-x-2 px-3 py-1 rounded-full ${permissionStatus.bgColor}`}>
            <StatusIcon className={`h-4 w-4 ${permissionStatus.color}`} />
            <span className={`text-sm font-medium ${permissionStatus.color}`}>
              {permissionStatus.text}
            </span>
          </div>
        </div>

        <p className="text-gray-600 text-sm mb-4">
          Get real-time notifications about your raffles, winnings, and account updates.
        </p>

        {/* Enable/Disable Notifications */}
        {!notificationPermission.granted ? (
          <div className="space-y-3">
            {notificationPermission.denied ? (
              <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
                <div className="flex items-start space-x-3">
                  <ExclamationTriangleIcon className="h-5 w-5 text-red-600 mt-0.5" />
                  <div>
                    <h4 className="text-sm font-medium text-red-900">
                      Notifications Blocked
                    </h4>
                    <p className="text-sm text-red-700 mt-1">
                      You've blocked notifications for this site. To enable them, 
                      click the lock icon in your browser's address bar and allow notifications.
                    </p>
                  </div>
                </div>
              </div>
            ) : (
              <Button
                onClick={handleEnableNotifications}
                disabled={isLoading}
                className="w-full sm:w-auto bg-indigo-600 hover:bg-indigo-700"
                isLoading={isLoading}
              >
                <BellIcon className="h-4 w-4 mr-2" />
                Enable Notifications
              </Button>
            )}
          </div>
        ) : (
          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-700">
              Notifications are enabled for this device
            </span>
            <Button
              onClick={handleDisableNotifications}
              disabled={isLoading}
              variant="outline"
              size="sm"
              isLoading={isLoading}
            >
              <BellSlashIcon className="h-4 w-4 mr-2" />
              Disable
            </Button>
          </div>
        )}
      </div>

      {/* Notification Settings */}
      {notificationPermission.granted && (
        <div className="space-y-4">
          {Object.entries(groupedSettings).map(([category, categorySettings]) => (
            <div key={category} className="bg-white rounded-lg border border-gray-200 p-6">
              <h4 className="text-md font-semibold text-gray-900 mb-4">
                {categoryTitles[category as keyof typeof categoryTitles]}
              </h4>
              
              <div className="space-y-4">
                {categorySettings.map((setting) => (
                  <div key={setting.id} className="flex items-center justify-between">
                    <div className="flex-1">
                      <h5 className="text-sm font-medium text-gray-900">
                        {setting.title}
                      </h5>
                      <p className="text-sm text-gray-600">
                        {setting.description}
                      </p>
                    </div>
                    
                    <label className="relative inline-flex items-center cursor-pointer ml-4">
                      <input
                        type="checkbox"
                        checked={setting.enabled}
                        onChange={(e) => handleSettingChange(setting.id, e.target.checked)}
                        className="sr-only peer"
                      />
                      <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
                    </label>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Test Notification */}
      {notificationPermission.granted && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h4 className="text-md font-semibold text-gray-900 mb-2">
            Test Notifications
          </h4>
          <p className="text-sm text-gray-600 mb-4">
            Send a test notification to make sure everything is working correctly.
          </p>
          
          <Button
            onClick={() => {
              // This would send a test notification
              console.log('Sending test notification...');
            }}
            variant="outline"
            size="sm"
          >
            Send Test Notification
          </Button>
        </div>
      )}
    </div>
  );
};

export default PushNotificationSettings;