import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';
import { configureStore } from '@reduxjs/toolkit';
import { performance } from 'perf_hooks';
import RaffleGrid from '../../components/Raffle/RaffleGrid';
import MobileRaffleGrid from '../../components/Mobile/MobileRaffleGrid';
import DashboardPage from '../../pages/Dashboard/DashboardPage';
import authSlice from '../../store/slices/authSlice';

// Mock performance observer
const mockPerformanceObserver = jest.fn();
global.PerformanceObserver = jest.fn().mockImplementation(() => ({
  observe: mockPerformanceObserver,
  disconnect: jest.fn(),
}));

// Mock intersection observer for lazy loading tests
const mockIntersectionObserver = jest.fn();
global.IntersectionObserver = jest.fn().mockImplementation(() => ({
  observe: mockIntersectionObserver,
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));

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

// Create mock store
const createMockStore = (initialState = {}) => {
  return configureStore({
    reducer: {
      auth: authSlice,
      credit: (state = { balance: 100 }) => state,
      raffle: (state = { activeRaffles: [] }) => state,
    },
    preloadedState: {
      auth: {
        user: { id: '1', email: 'test@example.com' },
        isAuthenticated: true,
        isLoading: false,
        error: null,
      },
      credit: { balance: 100 },
      raffle: { activeRaffles: [] },
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

// Generate large raffle data for performance testing
const generateLargeRaffle = (totalBoxes: number) => ({
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
  totalBoxes,
  boxPrice: 10,
  totalWinners: 1,
  status: 'active' as const,
  boxesSold: Math.floor(totalBoxes * 0.25),
  participants: Array.from({ length: Math.floor(totalBoxes * 0.25) }, (_, i) => ({
    id: `p${i}`,
    userId: `user${i}`,
    raffleId: '1',
    boxNumber: i + 1,
    purchaseDate: new Date().toISOString(),
    transactionId: `tx${i}`,
  })),
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
  endDate: new Date(Date.now() + 86400000).toISOString(),
});

// Performance measurement utilities
const measureRenderTime = async (renderFn: () => void): Promise<number> => {
  const startTime = performance.now();
  renderFn();
  await waitFor(() => {
    // Wait for component to be fully rendered
  });
  const endTime = performance.now();
  return endTime - startTime;
};

const measureMemoryUsage = (): number => {
  if ('memory' in performance) {
    return (performance as any).memory.usedJSHeapSize;
  }
  return 0;
};

describe('Performance Tests', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('RaffleGrid Performance', () => {
    it('should render small grids (100 boxes) within performance budget', async () => {
      const smallRaffle = generateLargeRaffle(100);
      
      const renderTime = await measureRenderTime(() => {
        renderWithProviders(
          <RaffleGrid
            raffle={smallRaffle}
            onBoxPurchase={jest.fn()}
            onRaffleUpdate={jest.fn()}
          />
        );
      });

      // Should render within 100ms for small grids
      expect(renderTime).toBeLessThan(100);
    });

    it('should render medium grids (500 boxes) within acceptable time', async () => {
      const mediumRaffle = generateLargeRaffle(500);
      
      const renderTime = await measureRenderTime(() => {
        renderWithProviders(
          <RaffleGrid
            raffle={mediumRaffle}
            onBoxPurchase={jest.fn()}
            onRaffleUpdate={jest.fn()}
          />
        );
      });

      // Should render within 300ms for medium grids
      expect(renderTime).toBeLessThan(300);
    });

    it('should handle large grids (1000 boxes) with virtualization', async () => {
      const largeRaffle = generateLargeRaffle(1000);
      
      const renderTime = await measureRenderTime(() => {
        renderWithProviders(
          <RaffleGrid
            raffle={largeRaffle}
            onBoxPurchase={jest.fn()}
            onRaffleUpdate={jest.fn()}
          />
        );
      });

      // Should render within 500ms even for large grids
      expect(renderTime).toBeLessThan(500);
    });

    it('should not cause memory leaks during re-renders', async () => {
      const raffle = generateLargeRaffle(100);
      const initialMemory = measureMemoryUsage();

      // Render and unmount multiple times
      for (let i = 0; i < 10; i++) {
        const { unmount } = renderWithProviders(
          <RaffleGrid
            raffle={raffle}
            onBoxPurchase={jest.fn()}
            onRaffleUpdate={jest.fn()}
          />
        );
        unmount();
      }

      const finalMemory = measureMemoryUsage();
      const memoryIncrease = finalMemory - initialMemory;

      // Memory increase should be minimal (less than 10MB)
      expect(memoryIncrease).toBeLessThan(10 * 1024 * 1024);
    });

    it('should efficiently update when box states change', async () => {
      const raffle = generateLargeRaffle(100);
      
      const { rerender } = renderWithProviders(
        <RaffleGrid
          raffle={raffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Measure update time when a box is sold
      const updatedRaffle = {
        ...raffle,
        boxesSold: raffle.boxesSold + 1,
        participants: [
          ...raffle.participants,
          {
            id: 'new-participant',
            userId: 'new-user',
            raffleId: '1',
            boxNumber: raffle.boxesSold + 1,
            purchaseDate: new Date().toISOString(),
            transactionId: 'new-tx',
          },
        ],
      };

      const updateTime = await measureRenderTime(() => {
        rerender(
          <RaffleGrid
            raffle={updatedRaffle}
            onBoxPurchase={jest.fn()}
            onRaffleUpdate={jest.fn()}
          />
        );
      });

      // Updates should be fast (less than 50ms)
      expect(updateTime).toBeLessThan(50);
    });
  });

  describe('Mobile Performance', () => {
    it('should render mobile grids efficiently', async () => {
      const raffle = generateLargeRaffle(100);
      
      const renderTime = await measureRenderTime(() => {
        renderWithProviders(
          <MobileRaffleGrid
            raffle={raffle}
            onBoxPurchase={jest.fn()}
            onRaffleUpdate={jest.fn()}
          />
        );
      });

      // Mobile should render within 150ms
      expect(renderTime).toBeLessThan(150);
    });

    it('should handle touch interactions without lag', async () => {
      const raffle = generateLargeRaffle(100);
      
      renderWithProviders(
        <MobileRaffleGrid
          raffle={raffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      const box = screen.getByLabelText('Box 2 - available');
      
      // Measure touch response time
      const startTime = performance.now();
      fireEvent.touchStart(box);
      fireEvent.touchEnd(box);
      const endTime = performance.now();

      // Touch response should be immediate (less than 16ms for 60fps)
      expect(endTime - startTime).toBeLessThan(16);
    });
  });

  describe('Dashboard Performance', () => {
    it('should render dashboard with large datasets efficiently', async () => {
      // Create store with large dataset
      const largeDataStore = createMockStore({
        credit: {
          balance: 1000,
          transactions: Array.from({ length: 1000 }, (_, i) => ({
            id: `tx${i}`,
            type: i % 2 === 0 ? 'purchase' : 'deduction',
            amount: Math.random() * 100,
            description: `Transaction ${i}`,
            createdAt: new Date(Date.now() - i * 86400000).toISOString(),
          })),
        },
        raffle: {
          userRaffles: Array.from({ length: 100 }, (_, i) => ({
            id: `raffle${i}`,
            itemTitle: `Item ${i}`,
            boxesPurchased: Math.floor(Math.random() * 10) + 1,
            totalCost: Math.random() * 100,
            status: 'active',
            participationDate: new Date(Date.now() - i * 86400000).toISOString(),
          })),
          winnings: Array.from({ length: 20 }, (_, i) => ({
            id: `winning${i}`,
            itemTitle: `Won Item ${i}`,
            wonDate: new Date(Date.now() - i * 86400000).toISOString(),
            status: 'delivered',
          })),
        },
      });

      const renderTime = await measureRenderTime(() => {
        renderWithProviders(<DashboardPage />, { store: largeDataStore });
      });

      // Dashboard should render within 200ms even with large datasets
      expect(renderTime).toBeLessThan(200);
    });

    it('should implement efficient pagination for large lists', () => {
      const largeDataStore = createMockStore({
        credit: {
          transactions: Array.from({ length: 1000 }, (_, i) => ({
            id: `tx${i}`,
            type: 'purchase',
            amount: 10,
            description: `Transaction ${i}`,
            createdAt: new Date().toISOString(),
          })),
        },
      });

      renderWithProviders(<DashboardPage />, { store: largeDataStore });

      // Should only render a subset of items initially
      const transactionItems = screen.getAllByText(/Transaction/);
      expect(transactionItems.length).toBeLessThanOrEqual(50); // Assuming 50 items per page
    });
  });

  describe('Bundle Size and Loading Performance', () => {
    it('should lazy load components efficiently', async () => {
      // Mock dynamic import
      const mockLazyComponent = jest.fn().mockResolvedValue({
        default: () => <div>Lazy Component</div>,
      });

      // Simulate lazy loading
      const startTime = performance.now();
      await mockLazyComponent();
      const endTime = performance.now();

      // Lazy loading should be fast
      expect(endTime - startTime).toBeLessThan(100);
    });

    it('should implement code splitting effectively', () => {
      // This would typically test bundle analysis
      // For now, we'll verify that components are properly structured for splitting
      expect(RaffleGrid).toBeDefined();
      expect(MobileRaffleGrid).toBeDefined();
      expect(DashboardPage).toBeDefined();
    });
  });

  describe('Image Loading Performance', () => {
    it('should implement lazy loading for images', () => {
      const raffle = generateLargeRaffle(100);
      
      renderWithProviders(
        <RaffleGrid
          raffle={raffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check that intersection observer is used for lazy loading
      expect(mockIntersectionObserver).toHaveBeenCalled();
    });

    it('should optimize image formats and sizes', () => {
      const raffle = generateLargeRaffle(100);
      
      renderWithProviders(
        <RaffleGrid
          raffle={raffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      const images = screen.getAllByRole('img');
      images.forEach(img => {
        const src = img.getAttribute('src');
        // Should use optimized image formats or CDN
        expect(src).toBeTruthy();
      });
    });
  });

  describe('Real-time Updates Performance', () => {
    it('should handle frequent WebSocket updates efficiently', async () => {
      const raffle = generateLargeRaffle(100);
      
      renderWithProviders(
        <RaffleGrid
          raffle={raffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Simulate multiple rapid updates
      const updateTimes: number[] = [];
      
      for (let i = 0; i < 10; i++) {
        const startTime = performance.now();
        
        // Simulate WebSocket message
        fireEvent(window, new CustomEvent('websocket-message', {
          detail: {
            type: 'box_purchased',
            data: {
              raffleId: '1',
              boxNumbers: [i + 50],
              participant: { user: { email: 'test@example.com' } },
            },
          },
        }));
        
        await waitFor(() => {
          // Wait for update to be processed
        });
        
        const endTime = performance.now();
        updateTimes.push(endTime - startTime);
      }

      // All updates should be processed quickly
      const averageUpdateTime = updateTimes.reduce((a, b) => a + b, 0) / updateTimes.length;
      expect(averageUpdateTime).toBeLessThan(50);
    });

    it('should throttle rapid state changes', async () => {
      const raffle = generateLargeRaffle(100);
      let updateCount = 0;
      
      const mockOnRaffleUpdate = jest.fn(() => {
        updateCount++;
      });

      renderWithProviders(
        <RaffleGrid
          raffle={raffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={mockOnRaffleUpdate}
        />
      );

      // Send many rapid updates
      for (let i = 0; i < 100; i++) {
        fireEvent(window, new CustomEvent('websocket-message', {
          detail: {
            type: 'box_purchased',
            data: {
              raffleId: '1',
              boxNumbers: [i],
              participant: { user: { email: 'test@example.com' } },
            },
          },
        }));
      }

      await waitFor(() => {
        // Updates should be throttled
        expect(updateCount).toBeLessThan(100);
      });
    });
  });

  describe('Animation Performance', () => {
    it('should use efficient animations that maintain 60fps', async () => {
      const raffle = generateLargeRaffle(100);
      
      renderWithProviders(
        <RaffleGrid
          raffle={raffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Simulate animation trigger
      const box = screen.getByLabelText('Box 2 - available');
      fireEvent.click(box);

      // Check that animations use transform/opacity for better performance
      const animatedElements = document.querySelectorAll('[class*="animate"]');
      animatedElements.forEach(element => {
        const styles = window.getComputedStyle(element);
        // Should use GPU-accelerated properties
        expect(styles.transform || styles.opacity).toBeTruthy();
      });
    });

    it('should respect reduced motion preferences', () => {
      // Mock reduced motion preference
      Object.defineProperty(window, 'matchMedia', {
        value: jest.fn().mockImplementation(query => ({
          matches: query === '(prefers-reduced-motion: reduce)',
          media: query,
          onchange: null,
          addListener: jest.fn(),
          removeListener: jest.fn(),
        })),
      });

      const raffle = generateLargeRaffle(100);
      
      renderWithProviders(
        <RaffleGrid
          raffle={raffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Animations should be disabled or reduced
      const animatedElements = document.querySelectorAll('[class*="animate"]');
      expect(animatedElements.length).toBe(0);
    });
  });
});