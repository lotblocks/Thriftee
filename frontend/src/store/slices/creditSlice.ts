import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface Credit {
  id: string;
  userId: string;
  amount: number;
  type: 'general' | 'item_specific';
  itemId?: string;
  expiresAt?: string;
  createdAt: string;
}

interface CreditTransaction {
  id: string;
  userId: string;
  type: 'earned' | 'spent' | 'expired' | 'refunded';
  amount: number;
  description: string;
  raffleId?: string;
  createdAt: string;
}

interface CreditState {
  balance: number;
  credits: Credit[];
  transactions: CreditTransaction[];
  isLoading: boolean;
  error: string | null;
}

const initialState: CreditState = {
  balance: 0,
  credits: [],
  transactions: [],
  isLoading: false,
  error: null,
};

const creditSlice = createSlice({
  name: 'credit',
  initialState,
  reducers: {
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload;
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
    setBalance: (state, action: PayloadAction<number>) => {
      state.balance = action.payload;
    },
    setCredits: (state, action: PayloadAction<Credit[]>) => {
      state.credits = action.payload;
    },
    addCredit: (state, action: PayloadAction<Credit>) => {
      state.credits.push(action.payload);
      state.balance += action.payload.amount;
    },
    removeCredit: (state, action: PayloadAction<string>) => {
      const credit = state.credits.find(c => c.id === action.payload);
      if (credit) {
        state.balance -= credit.amount;
        state.credits = state.credits.filter(c => c.id !== action.payload);
      }
    },
    setTransactions: (state, action: PayloadAction<CreditTransaction[]>) => {
      state.transactions = action.payload;
    },
    addTransaction: (state, action: PayloadAction<CreditTransaction>) => {
      state.transactions.unshift(action.payload);
    },
    clearError: (state) => {
      state.error = null;
    },
  },
});

export const {
  setLoading,
  setError,
  setBalance,
  setCredits,
  addCredit,
  removeCredit,
  setTransactions,
  addTransaction,
  clearError,
} = creditSlice.actions;

export default creditSlice.reducer;