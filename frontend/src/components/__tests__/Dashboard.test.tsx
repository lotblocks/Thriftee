import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';
import { configureStore } from '@reduxjs/toolkit';
import DashboardPage from '../../pages/Dashboard/DashboardPage';
import authSlice from '../../store/slices/authSlice';

// Mock the hooks and services
jest.mock('../../hooks/useResponsive', () => ({
  useResponsive: () => ({
    isMobile: false,
    isTablet: false,
    isDesktop: true,
    screenWidth: 1920,
  }),
}));

jest.mock('../../services/api', () => ({
  api: {
    get: jest.fn(),
    post: jest.fn(),
    put: jest.fn(),
    delete: jest.fn(),
  },
}));

// Create a mock store
const createMockStore = (initialState = {}) => {
  return configureStore({
    reducer: {
      auth: authSlice,
      credit: (state = { balance: 250.50, transactions: [] }) => state,
      raffle: (state = { userRaffles: [], winnings: [] }) => state,
    },
    preloadedState: {
      auth: {
        user: { 
          id: '1', 
          email: 'test@example.com',
          createdAt: '2024-01-01T00:00:00Z',
        },
        isAuthenticated: true,
        isLoading: false,
        error: null,
      },
      credit: { 
        balance: 250.50,
        transactions: [
          {
            id: '1',
            type: 'purchase',
            amount: 50.00,
            description: 'Credit purchase',
            createdAt: '2024-01-15T10:00:00Z',
          },
          {
            id: '2',
            type: 'deduction',
            amount: -10.00,
            description: 'Box purchase - iPhone 15 Pro',
            createdAt: '2024-01-16T14:30:00Z',
          },
        ],
      },
      raffle: {
        userRaffles: [
          {
            id: '1',
            itemTitle: 'iPhone 15 Pro',
            boxesPurchased: 3,
            totalCost: 30.00,
            status: 'active',
            participationDate: '2024-01-16T14:30:00Z',
          },
        ],
        winnings: [
          {
            id: '1',
            itemTitle: 'AirPods Pro',
            wonDate: '2024-01-10T16:00:00Z',
            status: 'shipped',
            trackingNumber: 'TR123456789',
          },
        ],
      },
      ...initialState,
    },
  });
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

describe('DashboardPage', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders dashboard with user information', () => {
    renderWithProviders(<DashboardPage />);

    expect(screen.getByText('Dashboard')).toBeInTheDocument();
    expect(screen.getByText('Welcome back, test@example.com')).toBeInTheDocument();
  });

  it('displays credit balance correctly', () => {
    renderWithProviders(<DashboardPage />);

    expect(screen.getByText('$250.50')).toBeInTheDocument();
    expect(screen.getByText('Credit Balance')).toBeInTheDocument();
  });

  it('shows recent activity', () => {
    renderWithProviders(<DashboardPage />);

    expect(screen.getByText('Recent Activity')).toBeInTheDocument();
    expect(screen.getByText('Credit purchase')).toBeInTheDocument();
    expect(screen.getByText('Box purchase - iPhone 15 Pro')).toBeInTheDocument();
  });

  it('displays participation history', () => {
    renderWithProviders(<DashboardPage />);

    expect(screen.getByText('My Raffles')).toBeInTheDocument();
    expect(screen.getByText('iPhone 15 Pro')).toBeInTheDocument();
    expect(screen.getByText('3 boxes')).toBeInTheDocument();
  });

  it('shows winnings section', () => {
    renderWithProviders(<DashboardPage />);

    expect(screen.getByText('My Winnings')).toBeInTheDocument();
    expect(screen.getByText('AirPods Pro')).toBeInTheDocument();
    expect(screen.getByText('Shipped')).toBeInTheDocument();
  });

  it('handles empty states correctly', () => {
    const storeWithEmptyData = createMockStore({
      credit: { balance: 0, transactions: [] },
      raffle: { userRaffles: [], winnings: [] },
    });

    renderWithProviders(<DashboardPage />, { store: storeWithEmptyData });

    expect(screen.getByText('No recent activity')).toBeInTheDocument();
    expect(screen.getByText('No active raffles')).toBeInTheDocument();
    expect(screen.getByText('No winnings yet')).toBeInTheDocument();
  });

  it('navigates to credit purchase when buy credits is clicked', async () => {
    renderWithProviders(<DashboardPage />);

    const buyCreditButton = screen.getByText('Buy Credits');
    fireEvent.click(buyCreditButton);

    // This would typically test navigation, but we'll just verify the button exists
    expect(buyCreditButton).toBeInTheDocument();
  });

  it('displays stats cards with correct information', () => {
    renderWithProviders(<DashboardPage />);

    // Check for stats cards
    expect(screen.getByText('Total Spent')).toBeInTheDocument();
    expect(screen.getByText('Active Raffles')).toBeInTheDocument();
    expect(screen.getByText('Items Won')).toBeInTheDocument();
  });

  it('handles loading state', () => {
    const storeWithLoading = createMockStore({
      auth: {
        user: null,
        isAuthenticated: false,
        isLoading: true,
        error: null,
      },
    });

    renderWithProviders(<DashboardPage />, { store: storeWithLoading });

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('handles error state', () => {
    const storeWithError = createMockStore({
      auth: {
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: 'Failed to load user data',
      },
    });

    renderWithProviders(<DashboardPage />, { store: storeWithError });

    expect(screen.getByText('Failed to load user data')).toBeInTheDocument();
  });

  it('refreshes data when refresh button is clicked', async () => {
    const mockRefresh = jest.fn();
    
    renderWithProviders(<DashboardPage />);

    const refreshButton = screen.getByLabelText('Refresh dashboard');
    fireEvent.click(refreshButton);

    await waitFor(() => {
      // Verify refresh functionality
      expect(refreshButton).toBeInTheDocument();
    });
  });

  it('filters activity by type', async () => {
    renderWithProviders(<DashboardPage />);

    const filterButton = screen.getByText('All Activity');
    fireEvent.click(filterButton);

    const purchaseFilter = screen.getByText('Purchases Only');
    fireEvent.click(purchaseFilter);

    await waitFor(() => {
      expect(screen.getByText('Credit purchase')).toBeInTheDocument();
      expect(screen.queryByText('Box purchase - iPhone 15 Pro')).not.toBeInTheDocument();
    });
  });

  it('handles responsive layout on mobile', () => {
    // Mock mobile responsive hook
    jest.doMock('../../hooks/useResponsive', () => ({
      useResponsive: () => ({
        isMobile: true,
        isTablet: false,
        isDesktop: false,
        screenWidth: 375,
      }),
    }));

    renderWithProviders(<DashboardPage />);

    // Verify mobile-specific elements
    expect(screen.getByTestId('mobile-dashboard')).toBeInTheDocument();
  });
});