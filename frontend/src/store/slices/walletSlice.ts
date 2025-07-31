import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface WalletState {
  address: string | null;
  balance: string;
  isConnected: boolean;
  isConnecting: boolean;
  chainId: number | null;
  networkName: string | null;
  error: string | null;
}

const initialState: WalletState = {
  address: null,
  balance: '0',
  isConnected: false,
  isConnecting: false,
  chainId: null,
  networkName: null,
  error: null,
};

const walletSlice = createSlice({
  name: 'wallet',
  initialState,
  reducers: {
    setConnecting: (state, action: PayloadAction<boolean>) => {
      state.isConnecting = action.payload;
    },
    setConnected: (state, action: PayloadAction<{
      address: string;
      chainId: number;
      networkName: string;
    }>) => {
      state.address = action.payload.address;
      state.chainId = action.payload.chainId;
      state.networkName = action.payload.networkName;
      state.isConnected = true;
      state.isConnecting = false;
      state.error = null;
    },
    setDisconnected: (state) => {
      state.address = null;
      state.balance = '0';
      state.isConnected = false;
      state.isConnecting = false;
      state.chainId = null;
      state.networkName = null;
      state.error = null;
    },
    setBalance: (state, action: PayloadAction<string>) => {
      state.balance = action.payload;
    },
    setChainId: (state, action: PayloadAction<number>) => {
      state.chainId = action.payload;
    },
    setNetworkName: (state, action: PayloadAction<string>) => {
      state.networkName = action.payload;
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
      state.isConnecting = false;
    },
    clearError: (state) => {
      state.error = null;
    },
  },
});

export const {
  setConnecting,
  setConnected,
  setDisconnected,
  setBalance,
  setChainId,
  setNetworkName,
  setError,
  clearError,
} = walletSlice.actions;

export default walletSlice.reducer;