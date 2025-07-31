module.exports = {
  // Test environment
  testEnvironment: 'jsdom',
  
  // Setup files
  setupFilesAfterEnv: [
    '<rootDir>/src/setupTests.ts'
  ],
  
  // Module name mapping for CSS and asset files
  moduleNameMapping: {
    '\\.(css|less|scss|sass)$': 'identity-obj-proxy',
    '\\.(jpg|jpeg|png|gif|eot|otf|webp|svg|ttf|woff|woff2|mp4|webm|wav|mp3|m4a|aac|oga)$': '<rootDir>/src/__mocks__/fileMock.js'
  },
  
  // Path mapping to match tsconfig.json
  moduleDirectories: ['node_modules', '<rootDir>/src'],
  
  // Test file patterns
  testMatch: [
    '<rootDir>/src/**/__tests__/**/*.{js,jsx,ts,tsx}',
    '<rootDir>/src/**/*.{test,spec}.{js,jsx,ts,tsx}'
  ],
  
  // Files to ignore
  testPathIgnorePatterns: [
    '<rootDir>/node_modules/',
    '<rootDir>/build/',
    '<rootDir>/dist/'
  ],
  
  // Transform files
  transform: {
    '^.+\\.(js|jsx|ts|tsx)$': ['babel-jest', {
      presets: [
        ['@babel/preset-env', { targets: { node: 'current' } }],
        ['@babel/preset-react', { runtime: 'automatic' }],
        '@babel/preset-typescript'
      ]
    }]
  },
  
  // Coverage configuration
  collectCoverageFrom: [
    'src/**/*.{js,jsx,ts,tsx}',
    '!src/**/*.d.ts',
    '!src/index.tsx',
    '!src/reportWebVitals.ts',
    '!src/setupTests.ts',
    '!src/**/__tests__/**',
    '!src/**/*.test.{js,jsx,ts,tsx}',
    '!src/**/*.spec.{js,jsx,ts,tsx}',
    '!src/**/*.stories.{js,jsx,ts,tsx}',
    '!src/vite-env.d.ts'
  ],
  
  // Coverage thresholds
  coverageThreshold: {
    global: {
      branches: 70,
      functions: 70,
      lines: 70,
      statements: 70
    },
    // Specific thresholds for critical components
    './src/components/Raffle/': {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    },
    './src/components/Mobile/': {
      branches: 75,
      functions: 75,
      lines: 75,
      statements: 75
    },
    './src/services/': {
      branches: 85,
      functions: 85,
      lines: 85,
      statements: 85
    },
    './src/hooks/': {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  },
  
  // Coverage reporters
  coverageReporters: [
    'text',
    'text-summary',
    'html',
    'lcov',
    'json-summary'
  ],
  
  // Coverage directory
  coverageDirectory: '<rootDir>/coverage',
  
  // Test timeout
  testTimeout: 10000,
  
  // Global setup and teardown
  globalSetup: '<rootDir>/src/__tests__/setup/globalSetup.ts',
  globalTeardown: '<rootDir>/src/__tests__/setup/globalTeardown.ts',
  
  // Test environment options
  testEnvironmentOptions: {
    url: 'http://localhost:3000'
  },
  
  // Verbose output
  verbose: true,
  
  // Clear mocks between tests
  clearMocks: true,
  
  // Restore mocks after each test
  restoreMocks: true,
  
  // Error on deprecated features
  errorOnDeprecated: true,
  
  // Max worker processes
  maxWorkers: '50%',
  
  // Test result processor for custom reporting
  testResultsProcessor: '<rootDir>/src/__tests__/utils/testResultsProcessor.js',
  
  // Custom reporters
  reporters: [
    'default',
    ['jest-junit', {
      outputDirectory: '<rootDir>/test-results',
      outputName: 'junit.xml',
      classNameTemplate: '{classname}',
      titleTemplate: '{title}',
      ancestorSeparator: ' › ',
      usePathForSuiteName: true
    }],
    ['jest-html-reporters', {
      publicPath: '<rootDir>/test-results',
      filename: 'report.html',
      expand: true,
      hideIcon: false,
      pageTitle: 'Raffle Platform Test Report'
    }]
  ],
  
  // Watch plugins
  watchPlugins: [
    'jest-watch-typeahead/filename',
    'jest-watch-typeahead/testname'
  ],
  
  // Projects for different test types
  projects: [
    {
      displayName: 'unit',
      testMatch: [
        '<rootDir>/src/**/__tests__/**/*.test.{js,jsx,ts,tsx}',
        '<rootDir>/src/**/?(*.)(test).{js,jsx,ts,tsx}'
      ],
      testPathIgnorePatterns: [
        '<rootDir>/src/__tests__/integration/',
        '<rootDir>/src/__tests__/e2e/',
        '<rootDir>/src/__tests__/visual/',
        '<rootDir>/src/__tests__/accessibility/',
        '<rootDir>/src/__tests__/performance/'
      ]
    },
    {
      displayName: 'integration',
      testMatch: [
        '<rootDir>/src/__tests__/integration/**/*.test.{js,jsx,ts,tsx}'
      ],
      setupFilesAfterEnv: [
        '<rootDir>/src/setupTests.ts',
        '<rootDir>/src/__tests__/setup/integrationSetup.ts'
      ]
    },
    {
      displayName: 'accessibility',
      testMatch: [
        '<rootDir>/src/__tests__/accessibility/**/*.test.{js,jsx,ts,tsx}'
      ],
      setupFilesAfterEnv: [
        '<rootDir>/src/setupTests.ts',
        '<rootDir>/src/__tests__/setup/accessibilitySetup.ts'
      ]
    },
    {
      displayName: 'performance',
      testMatch: [
        '<rootDir>/src/__tests__/performance/**/*.test.{js,jsx,ts,tsx}'
      ],
      setupFilesAfterEnv: [
        '<rootDir>/src/setupTests.ts',
        '<rootDir>/src/__tests__/setup/performanceSetup.ts'
      ]
    },
    {
      displayName: 'visual',
      testMatch: [
        '<rootDir>/src/__tests__/visual/**/*.test.{js,jsx,ts,tsx}'
      ],
      setupFilesAfterEnv: [
        '<rootDir>/src/setupTests.ts',
        '<rootDir>/src/__tests__/setup/visualSetup.ts'
      ]
    }
  ]
};