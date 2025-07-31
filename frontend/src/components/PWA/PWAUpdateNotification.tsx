import React, { useState } from 'react';
import {
  XMarkIcon,
  ArrowPathIcon,
  ExclamationTriangleIcon,
} from '@heroicons/react/24/outline';
import { usePWA } from '../../services/pwaService';
import { Button } from '../ui/Button';

interface PWAUpdateNotificationProps {
  onClose?: () => void;
  onUpdate?: () => void;
}

const PWAUpdateNotification: React.FC<PWAUpdateNotificationProps> = ({
  onClose,
  onUpdate,
}) => {
  const { updateAvailable, updateApp } = usePWA();
  const [isUpdating, setIsUpdating] = useState(false);

  if (!updateAvailable) {
    return null;
  }

  const handleUpdate = async () => {
    setIsUpdating(true);
    
    try {
      await updateApp();
      onUpdate?.();
    } catch (error) {
      console.error('Update failed:', error);
      setIsUpdating(false);
    }
  };

  const handleClose = () => {
    onClose?.();
  };

  return (
    <div className="fixed bottom-4 left-4 right-4 md:left-auto md:right-4 md:max-w-sm z-50">
      <div className="bg-white rounded-lg shadow-lg border border-gray-200 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 bg-blue-50 border-b border-blue-100">
          <div className="flex items-center space-x-2">
            <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
              <ArrowPathIcon className="h-4 w-4 text-blue-600" />
            </div>
            <h3 className="text-sm font-semibold text-blue-900">
              Update Available
            </h3>
          </div>
          <button
            onClick={handleClose}
            className="p-1 text-blue-400 hover:text-blue-600 rounded-full hover:bg-blue-100"
          >
            <XMarkIcon className="h-4 w-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4">
          <p className="text-sm text-gray-700 mb-4">
            A new version of the app is available with improvements and bug fixes.
          </p>

          {/* Actions */}
          <div className="flex space-x-2">
            <Button
              onClick={handleUpdate}
              disabled={isUpdating}
              size="sm"
              className="flex-1 bg-blue-600 hover:bg-blue-700"
              isLoading={isUpdating}
            >
              {isUpdating ? 'Updating...' : 'Update Now'}
            </Button>
            
            <button
              onClick={handleClose}
              className="px-3 py-2 text-sm text-gray-600 hover:text-gray-800 font-medium"
            >
              Later
            </button>
          </div>

          {/* Warning */}
          <div className="flex items-start space-x-2 mt-3 p-2 bg-yellow-50 rounded-md">
            <ExclamationTriangleIcon className="h-4 w-4 text-yellow-600 mt-0.5 flex-shrink-0" />
            <p className="text-xs text-yellow-800">
              The app will reload after updating to apply changes.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PWAUpdateNotification;