export interface PaymentMethod {
  id: string;
  type: 'card' | 'bank_account' | 'crypto_wallet';
  last4?: string;
  brand?: string;
  expiryMonth?: number;
  expiryYear?: number;
  bankName?: string;
  walletAddress?: string;
  walletType?: 'metamask' | 'coinbase' | 'walletconnect';
  isDefault: boolean;
  isVerified: boolean;
  createdAt: string;
}

export interface Payment {
  id: string;
  userId: string;
  amount: number;
  currency: string;
  status: PaymentStatus;
  type: PaymentType;
  paymentMethodId?: string;
  stripePaymentIntentId?: string;
  transactionHash?: string;
  blockNumber?: number;
  gasUsed?: string;
  gasPrice?: string;
  raffleId?: string;
  description: string;
  metadata?: Record<string, any>;
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
}

export type PaymentStatus = 
  | 'pending' 
  | 'processing' 
  | 'completed' 
  | 'failed' 
  | 'cancelled' 
  | 'refunded' 
  | 'partially_refunded';

export type PaymentType = 
  | 'credit_purchase' 
  | 'box_purchase' 
  | 'subscription' 
  | 'refund' 
  | 'withdrawal';

export interface CreatePaymentRequest {
  amount: number;
  currency: string;
  type: PaymentType;
  paymentMethodId?: string;
  raffleId?: string;
  description?: string;
  metadata?: Record<string, any>;
}

export interface PaymentIntent {
  id: string;
  clientSecret: string;
  amount: number;
  currency: string;
  status: string;
}

export interface CryptoPaymentRequest {
  amount: number;
  currency: string;
  walletAddress: string;
  type: PaymentType;
  raffleId?: string;
}

export interface PaymentStatistics {
  totalSpent: number;
  totalRefunded: number;
  successfulPayments: number;
  failedPayments: number;
  averagePaymentAmount: number;
  paymentsByMonth: Array<{
    month: string;
    amount: number;
    count: number;
  }>;
}