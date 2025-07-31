import React from 'react';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';
import { configureStore } from '@reduxjs/toolkit';
import App from '../../App';
import authSlice from '../../store/slices/authSlice';
import { api } from '../../services/api';

// Mock API calls
jest.mock('../../services/api', () => ({
  api: {
    get: jest.fn(),
    post: jest.fn(),
    put: jest.fn(),
    delete: jest.fn(),
  },
}));

// Mock other services
jest.mock('../../services/pwaService', () => ({
  pwaService: {
    registerServiceWorker: jest.fn(),
  },
}));

jest.mock('../../utils/mobileUtils', () => ({
  initializeMobileUtils: jest.fn(),
}));

jest.mock('../../hooks/useResponsive', () => ({
  useResponsive: () => ({
    isMobile: false,
    isTablet: false,
    isDesktop: true,
    screenWidth: 1920,
  }),
}));

// Mock WebSocket
jest.mock('../../hooks/useWebSocket', () => ({
  useWebSocket: () => ({
    isConnected: true,
    sendMessage: jest.fn(),
  }),
}));

const mockApi = api as jest.Mocked<typeof api>;

// Create a mock store
const createMockStore = (initialState = {}) => {
  return configureStore({
    reducer: {
      auth: authSlice,
      credit: (state = { balance: 100 }) => state,
      raffle: (state = { activeRaffles: [] }) => state,
      payment: (state = { isProcessing: false }) => state,
    },
    preloadedState: {
      auth: {
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: null,
      },
      credit: { balance: 100 },
      raffle: { activeRaffles: [] },
      payment: { isProcessing: false },
      ...initialState,
    },
  });
};

const renderApp = (initialState = {}) => {
  const store = createMockStore(initialState);
  return render(
    <Provider store={store}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </Provider>
  );
};

describe('User Workflows Integration Tests', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Reset localStorage
    localStorage.clear();
  });

  describe('User Registration and Login Flow', () => {
    it('allows user to register and login successfully', async () => {
      const user = userEvent.setup();
      
      // Mock successful registration
      mockApi.post.mockResolvedValueOnce({
        data: {
          user: { id: '1', email: 'test@example.com' },
          token: 'mock-token',
        },
      });

      renderApp();

      // Navigate to register page
      const registerLink = screen.getByText('Sign Up');
      await user.click(registerLink);

      // Fill registration form
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const confirmPasswordInput = screen.getByLabelText(/confirm password/i);

      await user.type(emailInput, 'test@example.com');
      await user.type(passwordInput, 'password123');
      await user.type(confirmPasswordInput, 'password123');

      // Submit registration
      const submitButton = screen.getByRole('button', { name: /sign up/i });
      await user.click(submitButton);

      // Verify API call
      expect(mockApi.post).toHaveBeenCalledWith('/auth/register', {
        email: 'test@example.com',
        password: 'password123',
      });

      // Verify redirect to dashboard
      await waitFor(() => {
        expect(screen.getByText('Dashboard')).toBeInTheDocument();
      });
    });

    it('handles registration errors gracefully', async () => {
      const user = userEvent.setup();
      
      // Mock registration error
      mockApi.post.mockRejectedValueOnce({
        response: {
          data: { message: 'Email already exists' },
        },
      });

      renderApp();

      // Navigate to register page
      const registerLink = screen.getByText('Sign Up');
      await user.click(registerLink);

      // Fill and submit form
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const confirmPasswordInput = screen.getByLabelText(/confirm password/i);

      await user.type(emailInput, 'existing@example.com');
      await user.type(passwordInput, 'password123');
      await user.type(confirmPasswordInput, 'password123');

      const submitButton = screen.getByRole('button', { name: /sign up/i });
      await user.click(submitButton);

      // Verify error message
      await waitFor(() => {
        expect(screen.getByText('Email already exists')).toBeInTheDocument();
      });
    });
  });

  describe('Raffle Participation Flow', () => {
    const mockRaffle = {
      id: '1',
      item: {
        id: 'item1',
        title: 'iPhone 15 Pro',
        description: 'Latest iPhone',
        price: 999,
        imageUrls: ['https://example.com/iphone.jpg'],
        category: 'Electronics',
      },
      totalBoxes: 100,
      boxPrice: 10,
      boxesSold: 25,
      totalWinners: 1,
      status: 'active',
      participants: [],
    };

    it('allows authenticated user to participate in raffle', async () => {
      const user = userEvent.setup();
      
      // Mock authenticated state
      const authenticatedState = {
        auth: {
          user: { id: '1', email: 'test@example.com' },
          isAuthenticated: true,
          isLoading: false,
          error: null,
        },
        credit: { balance: 100 },
      };

      // Mock API responses
      mockApi.get.mockResolvedValueOnce({ data: [mockRaffle] }); // Get raffles
      mockApi.get.mockResolvedValueOnce({ data: mockRaffle }); // Get specific raffle
      mockApi.post.mockResolvedValueOnce({ 
        data: { success: true, boxNumbers: [26, 27] } 
      }); // Purchase boxes

      renderApp(authenticatedState);

      // Navigate to raffles page
      const rafflesLink = screen.getByText('Raffles');
      await user.click(rafflesLink);

      // Wait for raffles to load
      await waitFor(() => {
        expect(screen.getByText('iPhone 15 Pro')).toBeInTheDocument();
      });

      // Click on raffle to view details
      const raffleCard = screen.getByText('iPhone 15 Pro');
      await user.click(raffleCard);

      // Wait for raffle details to load
      await waitFor(() => {
        expect(screen.getByText('Select boxes to purchase')).toBeInTheDocument();
      });

      // Select boxes (simulate clicking on grid cells)
      const box26 = screen.getByLabelText('Box 26 - available');
      const box27 = screen.getByLabelText('Box 27 - available');
      
      await user.click(box26);
      await user.click(box27);

      // Verify boxes are selected
      expect(box26).toHaveAttribute('aria-pressed', 'true');
      expect(box27).toHaveAttribute('aria-pressed', 'true');

      // Purchase boxes
      const purchaseButton = screen.getByRole('button', { name: /purchase/i });
      await user.click(purchaseButton);

      // Verify API call
      expect(mockApi.post).toHaveBeenCalledWith('/raffles/1/buy-box', {
        boxNumbers: [26, 27],
      });

      // Verify success message
      await waitFor(() => {
        expect(screen.getByText(/purchase successful/i)).toBeInTheDocument();
      });
    });

    it('prevents unauthenticated users from purchasing boxes', async () => {
      const user = userEvent.setup();
      
      // Mock API response for raffle data
      mockApi.get.mockResolvedValueOnce({ data: mockRaffle });

      renderApp(); // Not authenticated

      // Navigate directly to raffle detail
      window.history.pushState({}, '', '/raffles/1');
      
      // Should redirect to login
      await waitFor(() => {
        expect(screen.getByText('Sign In')).toBeInTheDocument();
      });
    });

    it('handles insufficient credits gracefully', async () => {
      const user = userEvent.setup();
      
      // Mock authenticated state with low balance
      const lowBalanceState = {
        auth: {
          user: { id: '1', email: 'test@example.com' },
          isAuthenticated: true,
          isLoading: false,
          error: null,
        },
        credit: { balance: 5 }, // Less than box price
      };

      mockApi.get.mockResolvedValueOnce({ data: mockRaffle });
      mockApi.post.mockRejectedValueOnce({
        response: {
          data: { message: 'Insufficient credits' },
        },
      });

      renderApp(lowBalanceState);

      // Navigate to raffle
      window.history.pushState({}, '', '/raffles/1');

      // Try to purchase box
      await waitFor(() => {
        expect(screen.getByText('iPhone 15 Pro')).toBeInTheDocument();
      });

      const box26 = screen.getByLabelText('Box 26 - available');
      await user.click(box26);

      const purchaseButton = screen.getByRole('button', { name: /purchase/i });
      await user.click(purchaseButton);

      // Verify error handling
      await waitFor(() => {
        expect(screen.getByText('Insufficient credits')).toBeInTheDocument();
      });
    });
  });

  describe('Credit Purchase Flow', () => {
    it('allows user to purchase credits successfully', async () => {
      const user = userEvent.setup();
      
      // Mock authenticated state
      const authenticatedState = {
        auth: {
          user: { id: '1', email: 'test@example.com' },
          isAuthenticated: true,
          isLoading: false,
          error: null,
        },
        credit: { balance: 50 },
      };

      // Mock Stripe payment intent
      mockApi.post.mockResolvedValueOnce({
        data: { clientSecret: 'pi_test_client_secret' },
      });

      renderApp(authenticatedState);

      // Navigate to wallet
      const walletLink = screen.getByText('Wallet');
      await user.click(walletLink);

      // Click buy credits
      const buyCreditButton = screen.getByText('Buy Credits');
      await user.click(buyCreditButton);

      // Select credit amount
      const creditAmount = screen.getByLabelText('$50');
      await user.click(creditAmount);

      // Fill payment form (mock Stripe elements)
      const cardElement = screen.getByTestId('stripe-card-element');
      fireEvent.change(cardElement, {
        target: { value: '4242424242424242' },
      });

      // Submit payment
      const payButton = screen.getByRole('button', { name: /pay/i });
      await user.click(payButton);

      // Verify payment intent creation
      expect(mockApi.post).toHaveBeenCalledWith('/payments/create-intent', {
        amount: 50,
        currency: 'usd',
      });
    });
  });

  describe('Dashboard and Profile Management', () => {
    it('allows user to update profile information', async () => {
      const user = userEvent.setup();
      
      // Mock authenticated state
      const authenticatedState = {
        auth: {
          user: { 
            id: '1', 
            email: 'test@example.com',
            firstName: 'John',
            lastName: 'Doe',
          },
          isAuthenticated: true,
          isLoading: false,
          error: null,
        },
      };

      mockApi.put.mockResolvedValueOnce({
        data: { 
          user: { 
            id: '1', 
            email: 'test@example.com',
            firstName: 'Jane',
            lastName: 'Smith',
          } 
        },
      });

      renderApp(authenticatedState);

      // Navigate to profile
      const profileLink = screen.getByText('Profile');
      await user.click(profileLink);

      // Edit profile
      const editButton = screen.getByText('Edit Profile');
      await user.click(editButton);

      // Update fields
      const firstNameInput = screen.getByLabelText(/first name/i);
      const lastNameInput = screen.getByLabelText(/last name/i);

      await user.clear(firstNameInput);
      await user.type(firstNameInput, 'Jane');
      
      await user.clear(lastNameInput);
      await user.type(lastNameInput, 'Smith');

      // Save changes
      const saveButton = screen.getByRole('button', { name: /save/i });
      await user.click(saveButton);

      // Verify API call
      expect(mockApi.put).toHaveBeenCalledWith('/users/profile', {
        firstName: 'Jane',
        lastName: 'Smith',
      });

      // Verify success message
      await waitFor(() => {
        expect(screen.getByText(/profile updated/i)).toBeInTheDocument();
      });
    });
  });

  describe('Real-time Updates', () => {
    it('updates raffle grid when other users purchase boxes', async () => {
      const user = userEvent.setup();
      
      // Mock authenticated state
      const authenticatedState = {
        auth: {
          user: { id: '1', email: 'test@example.com' },
          isAuthenticated: true,
          isLoading: false,
          error: null,
        },
      };

      mockApi.get.mockResolvedValueOnce({ data: mockRaffle });

      renderApp(authenticatedState);

      // Navigate to raffle
      window.history.pushState({}, '', '/raffles/1');

      await waitFor(() => {
        expect(screen.getByText('iPhone 15 Pro')).toBeInTheDocument();
      });

      // Simulate real-time update (another user purchased box 30)
      const mockWebSocketMessage = {
        type: 'box_purchased',
        data: {
          raffleId: '1',
          boxNumbers: [30],
          participant: {
            user: { email: 'other@example.com' },
          },
        },
      };

      // Trigger WebSocket message (this would be handled by the WebSocket hook)
      fireEvent(window, new CustomEvent('websocket-message', {
        detail: mockWebSocketMessage,
      }));

      // Verify box 30 is now marked as sold
      await waitFor(() => {
        const box30 = screen.getByLabelText('Box 30 - sold');
        expect(box30).toBeInTheDocument();
      });

      // Verify notification
      expect(screen.getByText(/other@example.com purchased 1 box/i)).toBeInTheDocument();
    });
  });

  describe('Error Handling and Edge Cases', () => {
    it('handles network errors gracefully', async () => {
      const user = userEvent.setup();
      
      // Mock network error
      mockApi.get.mockRejectedValueOnce(new Error('Network Error'));

      renderApp();

      // Navigate to raffles
      const rafflesLink = screen.getByText('Raffles');
      await user.click(rafflesLink);

      // Verify error message
      await waitFor(() => {
        expect(screen.getByText(/failed to load raffles/i)).toBeInTheDocument();
      });

      // Verify retry button
      const retryButton = screen.getByText('Retry');
      expect(retryButton).toBeInTheDocument();
    });

    it('handles session expiration', async () => {
      const user = userEvent.setup();
      
      // Mock authenticated state
      const authenticatedState = {
        auth: {
          user: { id: '1', email: 'test@example.com' },
          isAuthenticated: true,
          isLoading: false,
          error: null,
        },
      };

      // Mock 401 response (session expired)
      mockApi.get.mockRejectedValueOnce({
        response: { status: 401 },
      });

      renderApp(authenticatedState);

      // Try to access protected resource
      const dashboardLink = screen.getByText('Dashboard');
      await user.click(dashboardLink);

      // Should redirect to login
      await waitFor(() => {
        expect(screen.getByText('Your session has expired')).toBeInTheDocument();
        expect(screen.getByText('Sign In')).toBeInTheDocument();
      });
    });
  });
});