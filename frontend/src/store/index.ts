import { configureStore } from '@reduxjs/toolkit';
import { TypedUseSelectorHook, useDispatch, useSelector } from 'react-redux';

import authReducer from './slices/authSlice';
import creditReducer from './slices/creditSlice';
import raffleReducer from './slices/raffleSlice';
import paymentReducer from './slices/paymentSlice';
import walletReducer from './slices/walletSlice';

export const store = configureStore({
  reducer: {
    auth: authReducer,
    credit: creditReducer,
    raffle: raffleReducer,
    payment: paymentReducer,
    wallet: walletReducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: ['persist/PERSIST', 'persist/REHYDRATE'],
      },
    }),
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

// Typed hooks
export const useAppDispatch = () => useDispatch<AppDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;