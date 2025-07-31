import React from 'react';
import { render } from '@testing-library/react';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';
import { configureStore } from '@reduxjs/toolkit';
import RaffleGrid from '../../components/Raffle/RaffleGrid';
import MobileRaffleGrid from '../../components/Mobile/MobileRaffleGrid';
import DashboardPage from '../../pages/Dashboard/DashboardPage';
import LoginPage from '../../pages/Auth/LoginPage';
import authSlice from '../../store/slices/authSlice';

// Mock hooks
jest.mock('../../hooks/useResponsive', () => ({
  useResponsive: () => ({
    isMobile: false,
    isTablet: false,
    isDesktop: true,
    screenWidth: 1920,
  }),
}));

jest.mock('../../hooks/useRaffleRealTime', () => ({
  useRaffleRealTime: () => ({
    isConnected: true,
    isConnecting: false,
    activeUsers: 5,
  }),
}));

// Mock API
jest.mock('../../services/api', () => ({
  api: {
    get: jest.fn(),
    post: jest.fn(),
  },
}));

// Create mock store
const createMockStore = (initialState = {}) => {
  return configureStore({
    reducer: {
      auth: authSlice,
      credit: (state = { balance: 100, transactions: [] }) => state,
      raffle: (state = { activeRaffles: [], userRaffles: [], winnings: [] }) => state,
    },
    preloadedState: {
      auth: {
        user: { id: '1', email: 'test@example.com' },
        isAuthenticated: true,
        isLoading: false,
        error: null,
      },
      credit: { 
        balance: 100,
        transactions: [
          {
            id: '1',
            type: 'purchase',
            amount: 50,
            description: 'Credit purchase',
            createdAt: '2024-01-15T10:00:00Z',
          },
        ],
      },
      raffle: { 
        activeRaffles: [],
        userRaffles: [
          {
            id: '1',
            itemTitle: 'iPhone 15 Pro',
            boxesPurchased: 3,
            totalCost: 30,
            status: 'active',
            participationDate: '2024-01-16T14:30:00Z',
          },
        ],
        winnings: [],
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

// Mock raffle data
const mockRaffle = {
  id: '1',
  sellerId: 'seller1',
  item: {
    id: 'item1',
    title: 'iPhone 15 Pro',
    description: 'Latest iPhone with amazing features',
    price: 999,
    imageUrls: ['https://example.com/iphone.jpg'],
    category: 'Electronics',
    condition: 'new' as const,
    sellerId: 'seller1',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  totalBoxes: 100,
  boxPrice: 10,
  totalWinners: 1,
  status: 'active' as const,
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
  endDate: new Date(Date.now() + 86400000).toISOString(),
};

// Visual regression test utilities
const takeSnapshot = (container: HTMLElement, testName: string) => {
  // In a real implementation, this would use a visual regression testing tool
  // like Percy, Chromatic, or jest-image-snapshot
  expect(container).toMatchSnapshot(`${testName}.html`);
};

describe('Visual Regression Tests', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Set consistent viewport size
    Object.defineProperty(window, 'innerWidth', { value: 1920 });
    Object.defineProperty(window, 'innerHeight', { value: 1080 });
  });

  describe('RaffleGrid Visual Tests', () => {
    it('should match snapshot for default raffle grid', () => {
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      takeSnapshot(container, 'raffle-grid-default');
    });

    it('should match snapshot for raffle grid with selected boxes', () => {
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Simulate box selection
      const box2 = container.querySelector('[aria-label="Box 2 - available"]');
      if (box2) {
        box2.setAttribute('aria-pressed', 'true');
        box2.classList.add('selected');
      }

      takeSnapshot(container, 'raffle-grid-with-selection');
    });

    it('should match snapshot for raffle grid in loading state', () => {
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
          isLoading={true}
        />
      );

      takeSnapshot(container, 'raffle-grid-loading');
    });

    it('should match snapshot for raffle grid in disabled state', () => {
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
          disabled={true}
        />
      );

      takeSnapshot(container, 'raffle-grid-disabled');
    });

    it('should match snapshot for nearly full raffle', () => {
      const nearlyFullRaffle = {
        ...mockRaffle,
        boxesSold: 95,
        participants: Array.from({ length: 95 }, (_, i) => ({
          id: `p${i}`,
          userId: `user${i}`,
          raffleId: '1',
          boxNumber: i + 1,
          purchaseDate: new Date().toISOString(),
          transactionId: `tx${i}`,
        })),
      };

      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={nearlyFullRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      takeSnapshot(container, 'raffle-grid-nearly-full');
    });

    it('should match snapshot for completed raffle', () => {
      const completedRaffle = {
        ...mockRaffle,
        status: 'completed' as const,
        boxesSold: 100,
        participants: Array.from({ length: 100 }, (_, i) => ({
          id: `p${i}`,
          userId: `user${i}`,
          raffleId: '1',
          boxNumber: i + 1,
          purchaseDate: new Date().toISOString(),
          transactionId: `tx${i}`,
        })),
      };

      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={completedRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      takeSnapshot(container, 'raffle-grid-completed');
    });
  });

  describe('Mobile RaffleGrid Visual Tests', () => {
    beforeEach(() => {
      // Set mobile viewport
      Object.defineProperty(window, 'innerWidth', { value: 375 });
      Object.defineProperty(window, 'innerHeight', { value: 667 });
    });

    it('should match snapshot for mobile raffle grid', () => {
      const { container } = renderWithProviders(
        <MobileRaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      takeSnapshot(container, 'mobile-raffle-grid-default');
    });

    it('should match snapshot for mobile raffle grid with menu open', () => {
      const { container } = renderWithProviders(
        <MobileRaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Simulate menu open
      const menuOverlay = container.querySelector('.mobile-menu-overlay');
      if (menuOverlay) {
        menuOverlay.classList.add('open');
      }

      takeSnapshot(container, 'mobile-raffle-grid-menu-open');
    });

    it('should match snapshot for mobile raffle grid with box selector expanded', () => {
      const { container } = renderWithProviders(
        <MobileRaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Simulate box selection and expanded selector
      const boxSelector = container.querySelector('.mobile-box-selector');
      if (boxSelector) {
        boxSelector.classList.add('expanded');
      }

      takeSnapshot(container, 'mobile-raffle-grid-selector-expanded');
    });
  });

  describe('Dashboard Visual Tests', () => {
    it('should match snapshot for dashboard with data', () => {
      const { container } = renderWithProviders(<DashboardPage />);

      takeSnapshot(container, 'dashboard-with-data');
    });

    it('should match snapshot for dashboard with empty state', () => {
      const emptyStore = createMockStore({
        credit: { balance: 0, transactions: [] },
        raffle: { activeRaffles: [], userRaffles: [], winnings: [] },
      });

      const { container } = renderWithProviders(<DashboardPage />, { 
        store: emptyStore 
      });

      takeSnapshot(container, 'dashboard-empty-state');
    });

    it('should match snapshot for dashboard in loading state', () => {
      const loadingStore = createMockStore({
        auth: {
          user: null,
          isAuthenticated: false,
          isLoading: true,
          error: null,
        },
      });

      const { container } = renderWithProviders(<DashboardPage />, { 
        store: loadingStore 
      });

      takeSnapshot(container, 'dashboard-loading');
    });

    it('should match snapshot for dashboard with error state', () => {
      const errorStore = createMockStore({
        auth: {
          user: null,
          isAuthenticated: false,
          isLoading: false,
          error: 'Failed to load user data',
        },
      });

      const { container } = renderWithProviders(<DashboardPage />, { 
        store: errorStore 
      });

      takeSnapshot(container, 'dashboard-error');
    });
  });

  describe('Login Page Visual Tests', () => {
    it('should match snapshot for login page default state', () => {
      const { container } = render(<LoginPage />);

      takeSnapshot(container, 'login-page-default');
    });

    it('should match snapshot for login page with validation errors', () => {
      const { container } = render(<LoginPage />);

      // Simulate validation errors
      const errorElements = container.querySelectorAll('.error-message');
      errorElements.forEach(element => {
        element.textContent = 'This field is required';
        element.classList.add('visible');
      });

      takeSnapshot(container, 'login-page-with-errors');
    });

    it('should match snapshot for login page in loading state', () => {
      const { container } = render(<LoginPage />);

      // Simulate loading state
      const submitButton = container.querySelector('button[type="submit"]');
      if (submitButton) {
        submitButton.setAttribute('disabled', 'true');
        submitButton.textContent = 'Signing in...';
      }

      takeSnapshot(container, 'login-page-loading');
    });
  });

  describe('Responsive Design Visual Tests', () => {
    const viewports = [
      { name: 'mobile', width: 375, height: 667 },
      { name: 'tablet', width: 768, height: 1024 },
      { name: 'desktop', width: 1920, height: 1080 },
      { name: 'large-desktop', width: 2560, height: 1440 },
    ];

    viewports.forEach(viewport => {
      it(`should match snapshot for raffle grid on ${viewport.name}`, () => {
        Object.defineProperty(window, 'innerWidth', { value: viewport.width });
        Object.defineProperty(window, 'innerHeight', { value: viewport.height });

        const { container } = renderWithProviders(
          <RaffleGrid
            raffle={mockRaffle}
            onBoxPurchase={jest.fn()}
            onRaffleUpdate={jest.fn()}
          />
        );

        takeSnapshot(container, `raffle-grid-${viewport.name}`);
      });

      it(`should match snapshot for dashboard on ${viewport.name}`, () => {
        Object.defineProperty(window, 'innerWidth', { value: viewport.width });
        Object.defineProperty(window, 'innerHeight', { value: viewport.height });

        const { container } = renderWithProviders(<DashboardPage />);

        takeSnapshot(container, `dashboard-${viewport.name}`);
      });
    });
  });

  describe('Theme and Color Scheme Visual Tests', () => {
    it('should match snapshot for light theme', () => {
      document.documentElement.classList.remove('dark');
      
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      takeSnapshot(container, 'raffle-grid-light-theme');
    });

    it('should match snapshot for dark theme', () => {
      document.documentElement.classList.add('dark');
      
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      takeSnapshot(container, 'raffle-grid-dark-theme');
      
      // Cleanup
      document.documentElement.classList.remove('dark');
    });

    it('should match snapshot for high contrast mode', () => {
      document.documentElement.classList.add('high-contrast');
      
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      takeSnapshot(container, 'raffle-grid-high-contrast');
      
      // Cleanup
      document.documentElement.classList.remove('high-contrast');
    });
  });

  describe('Animation State Visual Tests', () => {
    it('should match snapshot for components with animations paused', () => {
      // Mock reduced motion preference
      Object.defineProperty(window, 'matchMedia', {
        value: jest.fn().mockImplementation(query => ({
          matches: query === '(prefers-reduced-motion: reduce)',
          media: query,
        })),
      });

      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      takeSnapshot(container, 'raffle-grid-reduced-motion');
    });

    it('should match snapshot for components mid-animation', () => {
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Simulate animation state
      const animatedElements = container.querySelectorAll('[class*="animate"]');
      animatedElements.forEach(element => {
        element.classList.add('animation-paused');
      });

      takeSnapshot(container, 'raffle-grid-mid-animation');
    });
  });

  describe('Error State Visual Tests', () => {
    it('should match snapshot for network error state', () => {
      const { container } = renderWithProviders(
        <div className="error-container">
          <div className="error-icon">⚠️</div>
          <h2>Network Error</h2>
          <p>Unable to connect to the server. Please check your internet connection.</p>
          <button>Retry</button>
        </div>
      );

      takeSnapshot(container, 'network-error-state');
    });

    it('should match snapshot for 404 error state', () => {
      const { container } = renderWithProviders(
        <div className="error-container">
          <div className="error-icon">🔍</div>
          <h2>Page Not Found</h2>
          <p>The page you're looking for doesn't exist.</p>
          <button>Go Home</button>
        </div>
      );

      takeSnapshot(container, '404-error-state');
    });
  });
});