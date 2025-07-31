import React from 'react';
import {
  ShieldCheckIcon,
  EyeIcon,
  TrendingUpIcon,
  CubeIcon,
  WalletIcon,
  CheckBadgeIcon,
} from '@heroicons/react/24/outline';
import { useNavigate } from 'react-router-dom';

import { Button } from '../components/ui/Button';
import { useAppSelector } from '../store';

const HomePage: React.FC = () => {
  const navigate = useNavigate();
  const { isAuthenticated } = useAppSelector(state => state.auth);

  const features = [
    {
      icon: <ShieldCheckIcon className="h-12 w-12 text-indigo-600" />,
      title: 'No-Loss Guarantee',
      description: 'Never lose money with our credit system. Get full value back if you don\'t win.',
    },
    {
      icon: <EyeIcon className="h-12 w-12 text-indigo-600" />,
      title: 'Blockchain Transparency',
      description: 'Verifiable fairness through smart contracts and Chainlink VRF randomness.',
    },
    {
      icon: <TrendingUpIcon className="h-12 w-12 text-indigo-600" />,
      title: 'Exciting Experience',
      description: 'Interactive visual grid creates engagement and community around every raffle.',
    },
  ];

  const stats = [
    { label: 'Active Raffles', value: '150+' },
    { label: 'Happy Users', value: '10K+' },
    { label: 'Items Won', value: '5K+' },
    { label: 'Total Value', value: '$2M+' },
  ];

  const steps = [
    {
      number: '1',
      title: 'Browse & Choose',
      description: 'Explore our curated selection of items and choose the raffle you want to join.',
    },
    {
      number: '2',
      title: 'Purchase Boxes',
      description: 'Buy boxes using credits, crypto, or traditional payment methods.',
    },
    {
      number: '3',
      title: 'Fair Selection',
      description: 'Winners are selected using blockchain-verified randomness when all boxes are sold.',
    },
    {
      number: '4',
      title: 'Win or Get Credits',
      description: 'Winners get the item shipped. Non-winners receive credits equal to their purchase value.',
    },
  ];

  return (
    <div>
      {/* Hero Section */}
      <div className="bg-gradient-to-br from-indigo-500 via-purple-600 to-purple-700 text-white">
        <div className="container mx-auto px-4 py-16 md:py-24 text-center">
          <h1 className="text-4xl md:text-6xl font-bold mb-6 leading-tight">
            Transparent Raffle Shopping
          </h1>
          <p className="text-xl md:text-2xl mb-8 opacity-90 max-w-3xl mx-auto leading-relaxed">
            Experience the future of online shopping with blockchain-powered fairness
            and our unique no-loss guarantee system.
          </p>
          
          <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
            {isAuthenticated ? (
              <>
                <Button
                  onClick={() => navigate('/raffles')}
                  className="bg-white text-indigo-600 hover:bg-gray-100 px-8 py-3 text-lg font-semibold"
                >
                  Browse Raffles
                </Button>
                <Button
                  variant="secondary"
                  onClick={() => navigate('/dashboard')}
                  className="border-white text-white hover:bg-white hover:bg-opacity-10 px-8 py-3 text-lg font-semibold"
                >
                  Go to Dashboard
                </Button>
              </>
            ) : (
              <>
                <Button
                  onClick={() => navigate('/register')}
                  className="bg-white text-indigo-600 hover:bg-gray-100 px-8 py-3 text-lg font-semibold"
                >
                  Get Started
                </Button>
                <Button
                  variant="secondary"
                  onClick={() => navigate('/raffles')}
                  className="border-white text-white hover:bg-white hover:bg-opacity-10 px-8 py-3 text-lg font-semibold"
                >
                  Browse Raffles
                </Button>
              </>
            )}
          </div>

          {/* Stats */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 max-w-4xl mx-auto">
            {stats.map((stat, index) => (
              <div key={index} className="text-center">
                <div className="text-3xl md:text-4xl font-bold mb-2">
                  {stat.value}
                </div>
                <div className="text-sm md:text-base opacity-80">
                  {stat.label}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Features Section */}
      <div className="py-16 md:py-24 bg-white">
        <div className="container mx-auto px-4">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold text-gray-900 mb-4">
              Why Choose Unit Shop?
            </h2>
            <p className="text-xl text-gray-600 max-w-3xl mx-auto">
              We've revolutionized online shopping with transparency, fairness, and excitement.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {features.map((feature, index) => (
              <div
                key={index}
                className="text-center p-8 rounded-lg border border-gray-200 hover:shadow-lg transition-shadow duration-300"
              >
                <div className="flex justify-center mb-4">
                  {feature.icon}
                </div>
                <h3 className="text-xl font-semibold text-gray-900 mb-4">
                  {feature.title}
                </h3>
                <p className="text-gray-600 leading-relaxed">
                  {feature.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Demo Grid Section */}
      <div className="py-16 md:py-24 bg-gray-50">
        <div className="container mx-auto px-4">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold text-gray-900 mb-4">
              Interactive Raffle Experience
            </h2>
            <p className="text-xl text-gray-600 max-w-3xl mx-auto">
              Watch as boxes are purchased in real-time and the image is gradually revealed.
            </p>
          </div>

          <div className="max-w-4xl mx-auto">
            <div className="bg-white rounded-lg shadow-lg p-8">
              <div className="text-center mb-8">
                <h3 className="text-2xl font-semibold text-gray-900 mb-2">
                  Sample Raffle Grid
                </h3>
                <p className="text-gray-600">
                  This is how our interactive raffle grids work
                </p>
              </div>
              
              {/* Demo Grid */}
              <div className="grid grid-cols-8 gap-2 max-w-md mx-auto mb-8">
                {Array.from({ length: 64 }, (_, index) => {
                  const boxNumber = index + 1;
                  const isSold = Math.random() > 0.7; // 30% chance of being sold
                  
                  return (
                    <div
                      key={boxNumber}
                      className={`aspect-square border-2 rounded flex items-center justify-center text-xs font-semibold transition-all duration-200 ${
                        isSold
                          ? 'bg-red-50 border-red-300 text-red-600'
                          : 'bg-white border-gray-300 text-gray-600 hover:border-indigo-400 hover:bg-indigo-50 cursor-pointer'
                      }`}
                    >
                      {isSold ? '✓' : boxNumber}
                    </div>
                  );
                })}
              </div>

              <div className="text-center">
                <div className="inline-flex items-center space-x-4 text-sm text-gray-600">
                  <div className="flex items-center space-x-2">
                    <div className="w-4 h-4 bg-white border-2 border-gray-300 rounded"></div>
                    <span>Available</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <div className="w-4 h-4 bg-red-50 border-2 border-red-300 rounded flex items-center justify-center text-red-600 text-xs">✓</div>
                    <span>Sold</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* How It Works Section */}
      <div className="py-16 md:py-24 bg-white">
        <div className="container mx-auto px-4">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold text-gray-900 mb-4">
              How It Works
            </h2>
            <p className="text-xl text-gray-600">
              Simple, transparent, and fair - here's how our platform works.
            </p>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
            <div className="space-y-8">
              {steps.map((step, index) => (
                <div key={index} className="flex items-start space-x-4">
                  <div className="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center font-semibold">
                    {step.number}
                  </div>
                  <div>
                    <h3 className="text-xl font-semibold text-gray-900 mb-2">
                      {step.title}
                    </h3>
                    <p className="text-gray-600 leading-relaxed">
                      {step.description}
                    </p>
                  </div>
                </div>
              ))}
            </div>

            <div className="flex justify-center">
              <div className="bg-gray-50 rounded-lg p-8 text-center max-w-md">
                <CubeIcon className="h-20 w-20 text-indigo-600 mx-auto mb-4" />
                <h3 className="text-xl font-semibold text-gray-900 mb-2">
                  Interactive Raffle Grid
                </h3>
                <p className="text-gray-600">
                  Watch the excitement build as boxes are purchased and the raffle fills up!
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* CTA Section */}
      <div className="bg-gradient-to-br from-purple-600 via-indigo-600 to-indigo-700 text-white py-16 md:py-24">
        <div className="container mx-auto px-4 text-center">
          <div className="max-w-3xl mx-auto">
            <CheckBadgeIcon className="h-16 w-16 mx-auto mb-6" />
            <h2 className="text-3xl md:text-4xl font-bold mb-4">
              Ready to Start?
            </h2>
            <p className="text-xl mb-8 opacity-90">
              Join thousands of users who trust our transparent, fair, and exciting platform.
            </p>

            {!isAuthenticated && (
              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button
                  onClick={() => navigate('/register')}
                  className="bg-white text-indigo-600 hover:bg-gray-100 px-8 py-3 text-lg font-semibold"
                >
                  Create Account
                </Button>
                <Button
                  variant="secondary"
                  onClick={() => navigate('/login')}
                  className="border-white text-white hover:bg-white hover:bg-opacity-10 px-8 py-3 text-lg font-semibold"
                >
                  Sign In
                </Button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default HomePage;