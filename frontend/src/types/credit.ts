export interface Credit {
  id: string;
  userId: string;
  amount: number;
  type: 'general' | 'item_specific';
  itemId?: string;
  expiresAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreditTransaction {
  id: string;
  userId: string;
  type: 'earned' | 'spent' | 'expired' | 'refunded' | 'purchased';
  amount: number;
  description: string;
  raffleId?: string;
  paymentId?: string;
  createdAt: string;
}

export interface CreditBalance {
  total: number;
  general: number;
  itemSpecific: number;
  expiringSoon: number;
}

export interface CreditPurchaseRequest {
  amount: number;
  paymentMethodId: string;
}

export interface CreditRedemptionRequest {
  creditIds: string[];
  itemId?: string;
}

export interface CreditStatistics {
  totalEarned: number;
  totalSpent: number;
  totalExpired: number;
  currentBalance: number;
  expiringIn30Days: number;
}