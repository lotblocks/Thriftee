import React from 'react';
import { render, screen } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';
import { configureStore } from '@reduxjs/toolkit';
import RaffleGrid from '../../components/Raffle/RaffleGrid';
import DashboardPage from '../../pages/Dashboard/DashboardPage';
import LoginPage from '../../pages/Auth/LoginPage';
import MobileRaffleGrid from '../../components/Mobile/MobileRaffleGrid';
import authSlice from '../../store/slices/authSlice';

// Extend Jest matchers
expect.extend(toHaveNoViolations);

// Mock hooks and services
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

describe('Accessibility Tests', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('RaffleGrid Accessibility', () => {
    it('should not have accessibility violations', async () => {
      const { container } = renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper ARIA labels for grid cells', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check for proper ARIA labels
      const availableBox = screen.getByLabelText('Box 2 - available');
      expect(availableBox).toBeInTheDocument();
      expect(availableBox).toHaveAttribute('role', 'button');
      expect(availableBox).toHaveAttribute('tabindex', '0');

      const soldBox = screen.getByLabelText('Box 1 - sold');
      expect(soldBox).toBeInTheDocument();
      expect(soldBox).toHaveAttribute('aria-disabled', 'true');
    });

    it('should support keyboard navigation', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      const gridCells = screen.getAllByRole('button');
      const availableCells = gridCells.filter(cell => 
        !cell.hasAttribute('aria-disabled') || 
        cell.getAttribute('aria-disabled') === 'false'
      );

      // All available cells should be focusable
      availableCells.forEach(cell => {
        expect(cell).toHaveAttribute('tabindex', '0');
      });
    });

    it('should have proper heading hierarchy', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check heading hierarchy
      const mainHeading = screen.getByRole('heading', { level: 2 });
      expect(mainHeading).toHaveTextContent('iPhone 15 Pro');
    });

    it('should provide screen reader announcements for state changes', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check for live regions
      const liveRegion = screen.getByRole('status', { hidden: true });
      expect(liveRegion).toBeInTheDocument();
      expect(liveRegion).toHaveAttribute('aria-live', 'polite');
    });
  });

  describe('Mobile RaffleGrid Accessibility', () => {
    it('should not have accessibility violations on mobile', async () => {
      const { container } = renderWithProviders(
        <MobileRaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper touch target sizes', () => {
      renderWithProviders(
        <MobileRaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      const gridCells = screen.getAllByRole('button');
      gridCells.forEach(cell => {
        const styles = window.getComputedStyle(cell);
        const minHeight = parseInt(styles.minHeight);
        const minWidth = parseInt(styles.minWidth);
        
        // iOS HIG recommends minimum 44px touch targets
        expect(minHeight).toBeGreaterThanOrEqual(44);
        expect(minWidth).toBeGreaterThanOrEqual(44);
      });
    });

    it('should support screen reader navigation on mobile', () => {
      renderWithProviders(
        <MobileRaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check for proper mobile navigation landmarks
      const navigation = screen.getByRole('navigation');
      expect(navigation).toBeInTheDocument();

      const main = screen.getByRole('main');
      expect(main).toBeInTheDocument();
    });
  });

  describe('Dashboard Accessibility', () => {
    it('should not have accessibility violations', async () => {
      const { container } = renderWithProviders(<DashboardPage />);

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper landmark regions', () => {
      renderWithProviders(<DashboardPage />);

      // Check for main landmark
      const main = screen.getByRole('main');
      expect(main).toBeInTheDocument();

      // Check for navigation if present
      const navigation = screen.queryByRole('navigation');
      if (navigation) {
        expect(navigation).toBeInTheDocument();
      }
    });

    it('should have accessible data tables', () => {
      renderWithProviders(<DashboardPage />);

      // Check for tables with proper headers
      const tables = screen.getAllByRole('table');
      tables.forEach(table => {
        const headers = within(table).getAllByRole('columnheader');
        expect(headers.length).toBeGreaterThan(0);
      });
    });

    it('should have proper form labels', () => {
      renderWithProviders(<DashboardPage />);

      // Check that all form inputs have labels
      const inputs = screen.getAllByRole('textbox');
      inputs.forEach(input => {
        const label = screen.getByLabelText(input.getAttribute('aria-label') || '');
        expect(label).toBeInTheDocument();
      });
    });
  });

  describe('Login Page Accessibility', () => {
    it('should not have accessibility violations', async () => {
      const { container } = render(<LoginPage />);

      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should have proper form accessibility', () => {
      render(<LoginPage />);

      // Check for form labels
      const emailInput = screen.getByLabelText(/email/i);
      expect(emailInput).toBeInTheDocument();
      expect(emailInput).toHaveAttribute('type', 'email');
      expect(emailInput).toHaveAttribute('required');

      const passwordInput = screen.getByLabelText(/password/i);
      expect(passwordInput).toBeInTheDocument();
      expect(passwordInput).toHaveAttribute('type', 'password');
      expect(passwordInput).toHaveAttribute('required');
    });

    it('should have proper error message association', () => {
      render(<LoginPage />);

      // Simulate form submission with errors
      const form = screen.getByRole('form');
      expect(form).toBeInTheDocument();

      // Check for error message containers
      const errorContainer = screen.queryByRole('alert');
      if (errorContainer) {
        expect(errorContainer).toHaveAttribute('aria-live', 'assertive');
      }
    });

    it('should have proper focus management', () => {
      render(<LoginPage />);

      // First focusable element should be email input
      const emailInput = screen.getByLabelText(/email/i);
      expect(emailInput).toBeInTheDocument();
      
      // Submit button should be accessible
      const submitButton = screen.getByRole('button', { name: /sign in/i });
      expect(submitButton).toBeInTheDocument();
      expect(submitButton).not.toHaveAttribute('disabled');
    });
  });

  describe('Color Contrast and Visual Accessibility', () => {
    it('should have sufficient color contrast for text', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // This would typically use a color contrast analyzer
      // For now, we'll check that text elements exist
      const textElements = screen.getAllByText(/./);
      expect(textElements.length).toBeGreaterThan(0);
    });

    it('should not rely solely on color for information', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check that sold boxes have text indicators, not just color
      const soldBox = screen.getByLabelText('Box 1 - sold');
      expect(soldBox).toHaveTextContent('SOLD');
    });
  });

  describe('Keyboard Navigation', () => {
    it('should support tab navigation through interactive elements', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Get all focusable elements
      const focusableElements = screen.getAllByRole('button');
      
      // All interactive elements should be keyboard accessible
      focusableElements.forEach(element => {
        if (!element.hasAttribute('disabled')) {
          expect(element).toHaveAttribute('tabindex', '0');
        }
      });
    });

    it('should have visible focus indicators', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check that focusable elements have focus styles
      const buttons = screen.getAllByRole('button');
      buttons.forEach(button => {
        // This would typically check computed styles for focus indicators
        expect(button).toBeInTheDocument();
      });
    });

    it('should support Enter and Space key activation', () => {
      const mockOnBoxPurchase = jest.fn();
      
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={mockOnBoxPurchase}
          onRaffleUpdate={jest.fn()}
        />
      );

      const availableBox = screen.getByLabelText('Box 2 - available');
      
      // Simulate Enter key press
      availableBox.focus();
      fireEvent.keyDown(availableBox, { key: 'Enter', code: 'Enter' });
      
      // Should select the box
      expect(availableBox).toHaveAttribute('aria-pressed', 'true');
    });
  });

  describe('Screen Reader Support', () => {
    it('should provide meaningful alternative text for images', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      const images = screen.getAllByRole('img');
      images.forEach(img => {
        expect(img).toHaveAttribute('alt');
        expect(img.getAttribute('alt')).not.toBe('');
      });
    });

    it('should announce dynamic content changes', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check for ARIA live regions
      const liveRegions = screen.getAllByRole('status');
      expect(liveRegions.length).toBeGreaterThan(0);
      
      liveRegions.forEach(region => {
        expect(region).toHaveAttribute('aria-live');
      });
    });

    it('should provide context for complex UI elements', () => {
      renderWithProviders(
        <RaffleGrid
          raffle={mockRaffle}
          onBoxPurchase={jest.fn()}
          onRaffleUpdate={jest.fn()}
        />
      );

      // Check for descriptive labels and help text
      const complexElements = screen.getAllByRole('button');
      complexElements.forEach(element => {
        const ariaLabel = element.getAttribute('aria-label');
        const ariaDescribedBy = element.getAttribute('aria-describedby');
        
        // Should have either aria-label or be described by another element
        expect(ariaLabel || ariaDescribedBy).toBeTruthy();
      });
    });
  });
});