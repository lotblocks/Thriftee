import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface PaymentMethod {
  id: string;
  type: 'card' | 'bank_account' | 'crypto_wallet';
  last4?: string;
  brand?: string;
  expiryMonth?: number;
  expiryYear?: number;
  isDefault: boolean;
  createdAt: string;
}

interface Payment {
  id: string;
  userId: string;
  amount: number;
  currency: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled' | 'refunded';
  type: 'credit_purchase' | 'box_purchase' | 'refund';
  paymentMethodId?: string;
  stripePaymentIntentId?: string;
  transactionHash?: string;
  raffleId?: string;
  createdAt: string;
  updatedAt: string;
}

interface PaymentState {
  paymentMethods: PaymentMethod[];
  payments: Payment[];
  isLoading: boolean;
  error: string | null;
  processingPayment: boolean;
}

const initialState: PaymentState = {
  paymentMethods: [],
  payments: [],
  isLoading: false,
  error: null,
  processingPayment: false,
};

const paymentSlice = createSlice({
  name: 'payment',
  initialState,
  reducers: {
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload;
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
    setProcessingPayment: (state, action: PayloadAction<boolean>) => {
      state.processingPayment = action.payload;
    },
    setPaymentMethods: (state, action: PayloadAction<PaymentMethod[]>) => {
      state.paymentMethods = action.payload;
    },
    addPaymentMethod: (state, action: PayloadAction<PaymentMethod>) => {
      state.paymentMethods.push(action.payload);
    },
    removePaymentMethod: (state, action: PayloadAction<string>) => {
      state.paymentMethods = state.paymentMethods.filter(pm => pm.id !== action.payload);
    },
    updatePaymentMethod: (state, action: PayloadAction<PaymentMethod>) => {
      const index = state.paymentMethods.findIndex(pm => pm.id === action.payload.id);
      if (index !== -1) {
        state.paymentMethods[index] = action.payload;
      }
    },
    setPayments: (state, action: PayloadAction<Payment[]>) => {
      state.payments = action.payload;
    },
    addPayment: (state, action: PayloadAction<Payment>) => {
      state.payments.unshift(action.payload);
    },
    updatePayment: (state, action: PayloadAction<Payment>) => {
      const index = state.payments.findIndex(p => p.id === action.payload.id);
      if (index !== -1) {
        state.payments[index] = action.payload;
      }
    },
    clearError: (state) => {
      state.error = null;
    },
  },
});

export const {
  setLoading,
  setError,
  setProcessingPayment,
  setPaymentMethods,
  addPaymentMethod,
  removePaymentMethod,
  updatePaymentMethod,
  setPayments,
  addPayment,
  updatePayment,
  clearError,
} = paymentSlice.actions;

export default paymentSlice.reducer;