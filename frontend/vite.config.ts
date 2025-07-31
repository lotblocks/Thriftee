import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import { splitVendorChunkPlugin } from 'vite'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({
      // Enable React Fast Refresh
      fastRefresh: true,
    }),
    splitVendorChunkPlugin(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/pages': path.resolve(__dirname, './src/pages'),
      '@/hooks': path.resolve(__dirname, './src/hooks'),
      '@/services': path.resolve(__dirname, './src/services'),
      '@/utils': path.resolve(__dirname, './src/utils'),
      '@/types': path.resolve(__dirname, './src/types'),
      '@/contexts': path.resolve(__dirname, './src/contexts'),
      '@/store': path.resolve(__dirname, './src/store'),
    },
  },
  server: {
    port: 3000,
    host: true,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false,
      },
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true,
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: process.env.NODE_ENV !== 'production',
    minify: 'terser',
    target: 'es2020',
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Vendor chunks
          if (id.includes('node_modules')) {
            // React ecosystem
            if (id.includes('react') || id.includes('react-dom')) {
              return 'react-vendor';
            }
            
            // Router
            if (id.includes('react-router')) {
              return 'router-vendor';
            }
            
            // UI libraries
            if (id.includes('@headlessui') || id.includes('@heroicons') || id.includes('framer-motion')) {
              return 'ui-vendor';
            }
            
            // State management
            if (id.includes('redux') || id.includes('@reduxjs') || id.includes('@tanstack/react-query')) {
              return 'state-vendor';
            }
            
            // Charts and visualization
            if (id.includes('recharts') || id.includes('d3')) {
              return 'charts-vendor';
            }
            
            // Utilities
            if (id.includes('axios') || id.includes('date-fns') || id.includes('clsx') || id.includes('tailwind-merge')) {
              return 'utils-vendor';
            }
            
            // Other large libraries
            return 'vendor';
          }
          
          // App chunks
          if (id.includes('/pages/')) {
            const pageName = id.split('/pages/')[1].split('/')[0];
            return `page-${pageName}`;
          }
          
          if (id.includes('/components/')) {
            const componentPath = id.split('/components/')[1];
            const componentGroup = componentPath.split('/')[0];
            return `components-${componentGroup}`;
          }
          
          if (id.includes('/services/')) {
            return 'services';
          }
          
          if (id.includes('/utils/')) {
            return 'utils';
          }
        },
        chunkFileNames: 'js/[name]-[hash].js',
        assetFileNames: (assetInfo) => {
          const info = assetInfo.name!.split('.');
          const ext = info[info.length - 1];
          
          if (/\.(png|jpe?g|gif|svg|webp|avif)$/i.test(assetInfo.name!)) {
            return `images/[name]-[hash][extname]`;
          }
          
          if (/\.(woff2?|eot|ttf|otf)$/i.test(assetInfo.name!)) {
            return `fonts/[name]-[hash][extname]`;
          }
          
          if (ext === 'css') {
            return `css/[name]-[hash][extname]`;
          }
          
          return `assets/[name]-[hash][extname]`;
        },
      },
    },
    terserOptions: {
      compress: {
        drop_console: process.env.NODE_ENV === 'production',
        drop_debugger: process.env.NODE_ENV === 'production',
        pure_funcs: process.env.NODE_ENV === 'production' ? ['console.log', 'console.debug'] : [],
      },
      mangle: {
        safari10: true,
      },
      format: {
        comments: false,
      },
    },
  },
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      'react-router-dom',
      '@reduxjs/toolkit',
      'react-redux',
      '@tanstack/react-query',
    ],
  },
  css: {
    devSourcemap: true,
  },
  esbuild: {
    drop: process.env.NODE_ENV === 'production' ? ['console', 'debugger'] : [],
    treeShaking: true,
  },
  preview: {
    port: 3000,
    host: true,
  },
  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version),
    __BUILD_TIME__: JSON.stringify(new Date().toISOString()),
  },
})