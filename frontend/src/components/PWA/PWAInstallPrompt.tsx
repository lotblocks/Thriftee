import React, { useState } from 'react';
import {
  XMarkIcon,
  ArrowDownTrayIcon,
  DevicePhoneMobileIcon,
  ComputerDesktopIcon,
  CheckCircleIcon,
} from '@heroicons/react/24/outline';
import { usePWA } from '../../services/pwaService';
import { Button } from '../ui/Button';

interface PWAInstallPromptProps {
  onClose?: () => void;
  onInstall?: () => void;
}

const PWAInstallPrompt: React.FC<PWAInstallPromptProps> = ({
  onClose,
  onInstall,
}) => {
  const { canInstall, isInstalled, install } = usePWA();
  const [isInstalling, setIsInstalling] = useState(false);
  const [installSuccess, setInstallSuccess] = useState(false);

  if (isInstalled || !canInstall) {
    return null;
  }

  const handleInstall = async () => {
    setIsInstalling(true);
    
    try {
      const success = await install();
      if (success) {
        setInstallSuccess(true);
        setTimeout(() => {
          onInstall?.();
          onClose?.();
        }, 2000);
      } else {
        setIsInstalling(false);
      }
    } catch (error) {
      console.error('Installation failed:', error);
      setIsInstalling(false);
    }
  };

  const handleClose = () => {
    onClose?.();
  };

  if (installSuccess) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
        <div className="bg-white rounded-lg max-w-sm w-full p-6 text-center">
          <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <CheckCircleIcon className="h-8 w-8 text-green-600" />
          </div>
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            App Installed Successfully!
          </h3>
          <p className="text-gray-600 text-sm">
            You can now access Raffle Platform from your home screen.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg max-w-md w-full overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">
            Install Raffle Platform
          </h3>
          <button
            onClick={handleClose}
            className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100"
          >
            <XMarkIcon className="h-5 w-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          {/* App Icon */}
          <div className="w-16 h-16 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
            <span className="text-white text-2xl font-bold">R</span>
          </div>

          {/* Description */}
          <div className="text-center mb-6">
            <h4 className="text-xl font-semibold text-gray-900 mb-2">
              Get the full experience
            </h4>
            <p className="text-gray-600 text-sm leading-relaxed">
              Install our app for faster access, offline browsing, and push notifications 
              about your raffles and winnings.
            </p>
          </div>

          {/* Features */}
          <div className="space-y-3 mb-6">
            <div className="flex items-center space-x-3">
              <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
                <DevicePhoneMobileIcon className="h-4 w-4 text-blue-600" />
              </div>
              <span className="text-sm text-gray-700">Works offline</span>
            </div>
            <div className="flex items-center space-x-3">
              <div className="w-8 h-8 bg-green-100 rounded-full flex items-center justify-center">
                <ComputerDesktopIcon className="h-4 w-4 text-green-600" />
              </div>
              <span className="text-sm text-gray-700">Fast loading</span>
            </div>
            <div className="flex items-center space-x-3">
              <div className="w-8 h-8 bg-purple-100 rounded-full flex items-center justify-center">
                <ArrowDownTrayIcon className="h-4 w-4 text-purple-600" />
              </div>
              <span className="text-sm text-gray-700">Push notifications</span>
            </div>
          </div>

          {/* Actions */}
          <div className="space-y-3">
            <Button
              onClick={handleInstall}
              disabled={isInstalling}
              className="w-full bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700"
              isLoading={isInstalling}
            >
              {isInstalling ? (
                'Installing...'
              ) : (
                <>
                  <ArrowDownTrayIcon className="h-4 w-4 mr-2" />
                  Install App
                </>
              )}
            </Button>
            
            <button
              onClick={handleClose}
              className="w-full py-2 px-4 text-gray-600 hover:text-gray-800 text-sm font-medium"
            >
              Maybe later
            </button>
          </div>

          {/* Privacy note */}
          <p className="text-xs text-gray-500 text-center mt-4">
            Installing this app will not share any additional data with us.
          </p>
        </div>
      </div>
    </div>
  );
};

export default PWAInstallPrompt;