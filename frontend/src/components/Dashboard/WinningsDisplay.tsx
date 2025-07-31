import React, { useState } from 'react';
import {
  TrophyIcon,
  TruckIcon,
  CheckCircleIcon,
  ClockIcon,
  EyeIcon,
  ShareIcon,
  GiftIcon,
  XMarkIcon,
} from '@heroicons/react/24/outline';

import { Button } from '../ui/Button';

interface WinningItem {
  id: string;
  raffleId: string;
  title: string;
  imageUrl: string;
  wonDate: string;
  value: number;
  status: 'pending_shipment' | 'shipped' | 'delivered' | 'claimed';
  trackingNumber?: string;
  estimatedDelivery?: string;
  shippingSteps: Array<{
    label: string;
    completed: boolean;
    date?: string;
  }>;
}

const WinningsDisplay: React.FC = () => {
  const [selectedWinning, setSelectedWinning] = useState<WinningItem | null>(null);
  const [showTrackingDialog, setShowTrackingDialog] = useState(false);

  // Mock data - in real app, this would come from API
  const winnings: WinningItem[] = [
    {
      id: '1',
      raffleId: '4',
      title: 'Nintendo Switch OLED',
      imageUrl: 'https://images.unsplash.com/photo-1606144042614-b2417e99c4e3?w=300',
      wonDate: '2024-01-15',
      value: 350,
      status: 'shipped',
      trackingNumber: 'UPS123456789',
      estimatedDelivery: '2024-01-20',
      shippingSteps: [
        { label: 'Order Confirmed', completed: true, date: '2024-01-15' },
        { label: 'Item Packaged', completed: true, date: '2024-01-16' },
        { label: 'Shipped', completed: true, date: '2024-01-17' },
        { label: 'Out for Delivery', completed: false },
        { label: 'Delivered', completed: false },
      ],
    },
    {
      id: '2',
      raffleId: '7',
      title: 'AirPods Pro (2nd Gen)',
      imageUrl: 'https://images.unsplash.com/photo-1606220945770-b5b6c2c55bf1?w=300',
      wonDate: '2023-12-20',
      value: 249,
      status: 'delivered',
      trackingNumber: 'FDX987654321',
      shippingSteps: [
        { label: 'Order Confirmed', completed: true, date: '2023-12-20' },
        { label: 'Item Packaged', completed: true, date: '2023-12-21' },
        { label: 'Shipped', completed: true, date: '2023-12-22' },
        { label: 'Out for Delivery', completed: true, date: '2023-12-24' },
        { label: 'Delivered', completed: true, date: '2023-12-24' },
      ],
    },
    {
      id: '3',
      raffleId: '12',
      title: 'Samsung Galaxy Watch 6',
      imageUrl: 'https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=300',
      wonDate: '2023-11-10',
      value: 329,
      status: 'pending_shipment',
      shippingSteps: [
        { label: 'Order Confirmed', completed: true, date: '2023-11-10' },
        { label: 'Item Packaged', completed: false },
        { label: 'Shipped', completed: false },
        { label: 'Out for Delivery', completed: false },
        { label: 'Delivered', completed: false },
      ],
    },
  ];

  const totalWinningsValue = winnings.reduce((sum, item) => sum + item.value, 0);

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'pending_shipment':
        return (
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
            <ClockIcon className="h-3 w-3 mr-1" />
            Preparing
          </span>
        );
      case 'shipped':
        return (
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
            <TruckIcon className="h-3 w-3 mr-1" />
            Shipped
          </span>
        );
      case 'delivered':
        return (
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
            <CheckCircleIcon className="h-3 w-3 mr-1" />
            Delivered
          </span>
        );
      case 'claimed':
        return (
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
            <GiftIcon className="h-3 w-3 mr-1" />
            Claimed
          </span>
        );
      default:
        return (
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
            Unknown
          </span>
        );
    }
  };

  const handleTrackingClick = (winning: WinningItem) => {
    setSelectedWinning(winning);
    setShowTrackingDialog(true);
  };

  return (
    <>
      <div className="bg-white rounded-lg shadow-sm border p-6">
        <div className="space-y-6">
          {/* Header */}
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-yellow-500 text-white rounded-lg">
                <TrophyIcon className="h-6 w-6" />
              </div>
              <div>
                <h3 className="text-xl font-semibold text-gray-900">
                  My Winnings
                </h3>
                <p className="text-sm text-gray-600">
                  Items you've won in raffles
                </p>
              </div>
            </div>
          </div>

          {/* Total value display */}
          <div className="text-center p-6 bg-gradient-to-br from-green-50 to-emerald-50 border-2 border-green-200 rounded-lg">
            <div className="text-3xl font-bold text-green-600 mb-2">
              ${totalWinningsValue.toLocaleString()}
            </div>
            <div className="text-lg text-gray-600 mb-1">
              Total Value Won
            </div>
            <div className="text-sm text-gray-500">
              {winnings.length} item{winnings.length !== 1 ? 's' : ''} won across all raffles
            </div>
          </div>

          {/* Winnings list */}
          <div className="space-y-3">
            {winnings.map((winning) => (
              <div key={winning.id} className="border border-gray-200 rounded-lg p-4">
                <div className="flex items-center space-x-4">
                  <img
                    src={winning.imageUrl}
                    alt={winning.title}
                    className="w-16 h-16 rounded-lg object-cover"
                  />
                  
                  <div className="flex-1">
                    <h4 className="font-medium text-gray-900 truncate">
                      {winning.title}
                    </h4>
                    <p className="text-sm text-gray-600">
                      Won on {new Date(winning.wonDate).toLocaleDateString()}
                    </p>
                    <p className="text-sm font-medium text-green-600">
                      Value: ${winning.value}
                    </p>
                  </div>

                  <div className="flex flex-col items-end space-y-2">
                    {getStatusBadge(winning.status)}
                    
                    <div className="flex space-x-1">
                      <button
                        className="p-1 text-gray-400 hover:text-gray-600 rounded"
                        title="View Details"
                      >
                        <EyeIcon className="h-4 w-4" />
                      </button>
                      
                      {winning.trackingNumber && (
                        <button
                          className="p-1 text-gray-400 hover:text-gray-600 rounded"
                          title="Track Shipment"
                          onClick={() => handleTrackingClick(winning)}
                        >
                          <TruckIcon className="h-4 w-4" />
                        </button>
                      )}
                      
                      <button
                        className="p-1 text-gray-400 hover:text-gray-600 rounded"
                        title="Share"
                      >
                        <ShareIcon className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {winnings.length === 0 && (
            <div className="text-center py-8">
              <TrophyIcon className="h-16 w-16 text-gray-300 mx-auto mb-4" />
              <h4 className="text-lg font-medium text-gray-900 mb-2">
                No winnings yet
              </h4>
              <p className="text-gray-600">
                Keep participating in raffles to win amazing items!
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Tracking Dialog */}
      {showTrackingDialog && selectedWinning && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-md w-full max-h-[90vh] overflow-auto">
            <div className="flex justify-between items-center p-6 border-b">
              <div className="flex items-center space-x-3">
                <TruckIcon className="h-6 w-6 text-blue-600" />
                <div>
                  <h3 className="text-lg font-semibold text-gray-900">
                    Shipment Tracking
                  </h3>
                  <p className="text-sm text-gray-600">
                    {selectedWinning.title}
                  </p>
                </div>
              </div>
              <button
                onClick={() => setShowTrackingDialog(false)}
                className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100 transition-colors"
              >
                <XMarkIcon className="h-5 w-5" />
              </button>
            </div>
            
            <div className="p-6 space-y-6">
              <div>
                <p className="text-sm text-gray-600 mb-1">Tracking Number</p>
                <p className="text-lg font-mono font-semibold text-gray-900">
                  {selectedWinning.trackingNumber}
                </p>
              </div>

              {selectedWinning.estimatedDelivery && (
                <div>
                  <p className="text-sm text-gray-600 mb-1">Estimated Delivery</p>
                  <p className="text-base text-gray-900">
                    {new Date(selectedWinning.estimatedDelivery).toLocaleDateString()}
                  </p>
                </div>
              )}

              <div>
                <h4 className="text-sm font-medium text-gray-900 mb-4">Shipping Progress</h4>
                <div className="space-y-4">
                  {selectedWinning.shippingSteps.map((step, index) => (
                    <div key={index} className="flex items-start space-x-3">
                      <div className={`flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center ${
                        step.completed 
                          ? 'bg-green-500 text-white' 
                          : 'bg-gray-200 text-gray-400'
                      }`}>
                        {step.completed ? (
                          <CheckCircleIcon className="h-4 w-4" />
                        ) : (
                          <div className="w-2 h-2 bg-current rounded-full" />
                        )}
                      </div>
                      <div className="flex-1">
                        <p className={`text-sm font-medium ${
                          step.completed ? 'text-gray-900' : 'text-gray-500'
                        }`}>
                          {step.label}
                        </p>
                        {step.date && (
                          <p className="text-xs text-gray-500">
                            {new Date(step.date).toLocaleDateString()}
                          </p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
            
            <div className="flex justify-end p-6 border-t">
              <Button
                variant="secondary"
                onClick={() => setShowTrackingDialog(false)}
              >
                Close
              </Button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export default WinningsDisplay;