import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Raffle, RaffleFilters } from '../../types/raffle';

interface RaffleState {
  raffles: Raffle[];
  currentRaffle: Raffle | null;
  filters: RaffleFilters;
  isLoading: boolean;
  error: string | null;
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}

const initialState: RaffleState = {
  raffles: [],
  currentRaffle: null,
  filters: {
    sortBy: 'newest',
  },
  isLoading: false,
  error: null,
  pagination: {
    page: 1,
    limit: 12,
    total: 0,
    totalPages: 0,
  },
};

const raffleSlice = createSlice({
  name: 'raffle',
  initialState,
  reducers: {
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload;
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
    setRaffles: (state, action: PayloadAction<Raffle[]>) => {
      state.raffles = action.payload;
    },
    addRaffle: (state, action: PayloadAction<Raffle>) => {
      state.raffles.unshift(action.payload);
    },
    updateRaffle: (state, action: PayloadAction<Raffle>) => {
      const index = state.raffles.findIndex(r => r.id === action.payload.id);
      if (index !== -1) {
        state.raffles[index] = action.payload;
      }
      if (state.currentRaffle?.id === action.payload.id) {
        state.currentRaffle = action.payload;
      }
    },
    setCurrentRaffle: (state, action: PayloadAction<Raffle | null>) => {
      state.currentRaffle = action.payload;
    },
    setFilters: (state, action: PayloadAction<Partial<RaffleFilters>>) => {
      state.filters = { ...state.filters, ...action.payload };
    },
    setPagination: (state, action: PayloadAction<Partial<RaffleState['pagination']>>) => {
      state.pagination = { ...state.pagination, ...action.payload };
    },
    clearError: (state) => {
      state.error = null;
    },
  },
});

export const {
  setLoading,
  setError,
  setRaffles,
  addRaffle,
  updateRaffle,
  setCurrentRaffle,
  setFilters,
  setPagination,
  clearError,
} = raffleSlice.actions;

export default raffleSlice.reducer;