// Performance testing setup
import { performance } from 'perf_hooks';

// Mock performance APIs for testing
const mockPerformanceEntries: PerformanceEntry[] = [];

// Enhanced performance mock
Object.defineProperty(global, 'performance', {
  value: {
    ...performance,
    now: jest.fn().mockImplementation(() => Date.now()),
    mark: jest.fn().mockImplementation((name: string) => {
      const entry = {
        name,
        entryType: 'mark',
        startTime: Date.now(),
        duration: 0,
      };
      mockPerformanceEntries.push(entry);
      return entry;
    }),
    measure: jest.fn().mockImplementation((name: string, startMark?: string, endMark?: string) => {
      const entry = {
        name,
        entryType: 'measure',
        startTime: Date.now() - 100,
        duration: 100,
      };
      mockPerformanceEntries.push(entry);
      return entry;
    }),
    getEntriesByName: jest.fn().mockImplementation((name: string) => {
      return mockPerformanceEntries.filter(entry => entry.name === name);
    }),
    getEntriesByType: jest.fn().mockImplementation((type: string) => {
      return mockPerformanceEntries.filter(entry => entry.entryType === type);
    }),
    clearMarks: jest.fn().mockImplementation(() => {
      mockPerformanceEntries.length = 0;
    }),
    clearMeasures: jest.fn().mockImplementation(() => {
      mockPerformanceEntries.length = 0;
    }),
    // Mock memory API
    memory: {
      usedJSHeapSize: 1000000,
      totalJSHeapSize: 2000000,
      jsHeapSizeLimit: 4000000,
    },
  },
  writable: true,
});

// Performance testing utilities
global.performanceUtils = {
  // Measure render time
  measureRenderTime: async (renderFn: () => void): Promise<number> => {
    const startTime = performance.now();
    renderFn();
    // Wait for next tick
    await new Promise(resolve => setTimeout(resolve, 0));
    const endTime = performance.now();
    return endTime - startTime;
  },
  
  // Measure memory usage
  measureMemoryUsage: (): number => {
    return (performance as any).memory?.usedJSHeapSize || 0;
  },
  
  // Create performance budget checker
  createBudgetChecker: (budgets: Record<string, number>) => ({
    check: (metric: string, value: number) => {
      const budget = budgets[metric];
      if (!budget) return { passed: true, message: 'No budget defined' };
      
      const passed = value <= budget;
      return {
        passed,
        message: passed 
          ? `${metric}: ${value} <= ${budget} ✓`
          : `${metric}: ${value} > ${budget} ✗`,
        value,
        budget,
      };
    },
  }),
};

beforeEach(() => {
  mockPerformanceEntries.length = 0;
  jest.clearAllMocks();
});