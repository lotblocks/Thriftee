import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';
import { configureStore } from '@reduxjs/toolkit';
import MobileRaffleGrid from '../MobileRaffleGrid';
import { Raffle } from '../../../types/raffle';
import authSlice from '../../../store/slices/authSlice';

// Mock the hooks
jest.mock('../../../hooks/useRaffleRealTime', () => ({
  useRaffleRealTime: () => ({
    isConnected: true,
    isConnecting: false,
    activeUsers: 5,
  }),
}));

jest.mock('../../../hooks/useResponsive', () => ({
  useResponsive: () => ({
    screenWidth: 375,
    screenHeight: 667,
    orientation: 'portrait',
  }),
  useTouchDevice: () => true,
}));

// Mock react-toastify
jest.mock('react-toastify', () => ({
  toast: {
    success: jest.fn(),
    error: jest.fn(),
    info: jest.fn(),
  },
}));

// Create a mock store
const createMockStore = (initialState = {}) => {
  return configureStore({
    reducer: {
      auth: authSlice,
      credit: (state = { balance: 100 }) => state,
    },
    preloadedState: {
      auth: {
        user: { id: '1', email: 'test@example.com' },
        isAuthenticated: true,
        isLoading: false,
        error: null,
      },
      credit: { balance: 100 },
      ...initialState,
    },
  });
};

// Mock raffle data
const mockRaffle: Raffle = {
  id: '1',
  sellerId: 'seller1',
  item: {
    id: 'item1',
    title: 'iPhone 15 Pro',
    description: 'Latest iPhone with amazing features',
    price: 999,
    imageUrls: ['https://example.com/iphone.jpg'],
    category: 'Electronics',
    condition: 'new',
    sellerId: 'seller1',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  totalBoxes: 100,
  boxPrice: 10,
  totalWinners: 1,
  status: 'active',
  boxesSold: 25,
  participants: [
    {
      id: 'p1',
      userId: 'user1',
      raffleId: '1',
      boxNumber: 1,
      purchaseDate: new Date().toISOString(),
      transactionId: 'tx1',
    },
  ],
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
  endDate: new Date(Date.now() + 86400000).toISOString(), // 24 hours from now
};

const renderWithProviders = (
  ui: React.ReactElement,
  { store = createMockStore(), ...renderOptions } = {}
) => {
  const Wrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <Provider store={store}>
      <BrowserRouter>
        {children}
      </BrowserRouter>
    </Provider>
  );

  return render(ui, { wrapper: Wrapper, ...renderOptions });
};

describe('MobileRaffleGrid', () => {
  const mockOnBoxPurchase = jest.fn();
  const mockOnRaffleUpdate = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    // Mock navigator.vibrate
    Object.defineProperty(navigator, 'vibrate', {
      value: jest.fn(),
      writable: true,
    });
  });

  it('renders mobile raffle grid correctly', () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />
    );

    expect(screen.getByText('iPhone 15 Pro')).toBeInTheDocument();
    expect(screen.getByText('25 / 100 sold')).toBeInTheDocument();
    expect(screen.getByText('$10')).toBeInTheDocument();
  });

  it('displays progress bar correctly', () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />
    );

    const progressText = screen.getByText('25%');
    expect(progressText).toBeInTheDocument();
  });

  it('shows connection status', () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />
    );

    // Should show connected status (WiFi icon)
    const connectionButton = screen.getByRole('button', { name: /connection/i });
    expect(connectionButton).toHaveClass('text-green-600');
  });

  it('handles box selection on mobile', async () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />
    );

    // Find an available box (not box 1 which is sold)
    const box2 = screen.getByLabelText('Box 2 - available');
    
    // Simulate touch interaction
    fireEvent.touchStart(box2);
    fireEvent.touchEnd(box2);

    await waitFor(() => {
      expect(box2).toHaveAttribute('aria-pressed', 'true');
    });
  });

  it('shows mobile menu when menu button is clicked', async () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />
    );

    const menuButton = screen.getByRole('button', { name: /menu/i });
    fireEvent.click(menuButton);

    await waitFor(() => {
      expect(screen.getByText('Raffle Details')).toBeInTheDocument();
    });
  });

  it('handles purchase flow correctly', async () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />
    );

    // Select a box
    const box2 = screen.getByLabelText('Box 2 - available');
    fireEvent.touchStart(box2);
    fireEvent.touchEnd(box2);

    await waitFor(() => {
      expect(screen.getByText('1 Box Selected')).toBeInTheDocument();
    });

    // Click purchase button
    const purchaseButton = screen.getByRole('button', { name: /purchase/i });
    fireEvent.click(purchaseButton);

    expect(mockOnBoxPurchase).toHaveBeenCalledWith([2]);
  });

  it('shows loading state correctly', () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
        isLoading={true}
      />
    );

    const purchaseButton = screen.getByRole('button', { name: /processing/i });
    expect(purchaseButton).toBeDisabled();
  });

  it('handles disabled state correctly', () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
        disabled={true}
      />
    );

    const box2 = screen.getByLabelText('Box 2 - available');
    fireEvent.touchStart(box2);
    fireEvent.touchEnd(box2);

    // Box should not be selected when disabled
    expect(box2).toHaveAttribute('aria-pressed', 'false');
  });

  it('shows login prompt for unauthenticated users', () => {
    const storeWithoutAuth = createMockStore({
      auth: {
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: null,
      },
    });

    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />,
      { store: storeWithoutAuth }
    );

    expect(screen.getByText(/log in.*or.*sign up.*to participate/i)).toBeInTheDocument();
  });

  it('handles zoom controls', () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />
    );

    const zoomInButton = screen.getByRole('button', { name: /zoom in/i });
    const zoomOutButton = screen.getByRole('button', { name: /zoom out/i });

    expect(zoomInButton).toBeInTheDocument();
    expect(zoomOutButton).toBeInTheDocument();

    // Test zoom functionality
    fireEvent.click(zoomInButton);
    fireEvent.click(zoomOutButton);
  });

  it('displays sold boxes correctly', () => {
    renderWithProviders(
      <MobileRaffleGrid
        raffle={mockRaffle}
        onBoxPurchase={mockOnBoxPurchase}
        onRaffleUpdate={mockOnRaffleUpdate}
      />
    );

    const soldBox = screen.getByLabelText('Box 1 - sold');
    expect(soldBox).toHaveAttribute('aria-disabled', 'true');
    expect(soldBox).toHaveClass('cursor-not-allowed');
  });
});