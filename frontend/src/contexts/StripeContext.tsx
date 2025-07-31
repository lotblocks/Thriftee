import React, { createContext, useContext, ReactNode } from 'react';
import { loadStripe, Stripe } from '@stripe/stripe-js';
import { Elements } from '@stripe/react-stripe-js';

interface StripeContextType {
  stripe: Promise<Stripe | null>;
}

const StripeContext = createContext<StripeContextType | undefined>(undefined);

interface StripeProviderProps {
  children: ReactNode;
}

// Initialize Stripe
const stripePromise = loadStripe(
  process.env.REACT_APP_STRIPE_PUBLISHABLE_KEY || ''
);

export const StripeProvider: React.FC<StripeProviderProps> = ({ children }) => {
  const value: StripeContextType = {
    stripe: stripePromise,
  };

  return (
    <StripeContext.Provider value={value}>
      <Elements stripe={stripePromise}>
        {children}
      </Elements>
    </StripeContext.Provider>
  );
};

export const useStripe = (): StripeContextType => {
  const context = useContext(StripeContext);
  if (context === undefined) {
    throw new Error('useStripe must be used within a StripeProvider');
  }
  return context;
};