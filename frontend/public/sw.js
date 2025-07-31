// Service Worker for Raffle Shopping Platform PWA
const CACHE_NAME = 'raffle-platform-v1';
const STATIC_CACHE_NAME = 'raffle-platform-static-v1';
const DYNAMIC_CACHE_NAME = 'raffle-platform-dynamic-v1';

// Assets to cache immediately
const STATIC_ASSETS = [
  '/',
  '/index.html',
  '/static/js/bundle.js',
  '/static/css/main.css',
  '/manifest.json',
  '/favicon.ico',
  // Add other critical assets
];

// API endpoints to cache
const API_CACHE_PATTERNS = [
  /\/api\/raffles/,
  /\/api\/items/,
  /\/api\/auth\/me/,
];

// Assets that should always be fetched from network
const NETWORK_FIRST_PATTERNS = [
  /\/api\/auth/,
  /\/api\/payments/,
  /\/api\/websocket/,
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
  console.log('Service Worker: Installing...');
  
  event.waitUntil(
    caches.open(STATIC_CACHE_NAME)
      .then((cache) => {
        console.log('Service Worker: Caching static assets');
        return cache.addAll(STATIC_ASSETS);
      })
      .then(() => {
        console.log('Service Worker: Static assets cached');
        return self.skipWaiting();
      })
      .catch((error) => {
        console.error('Service Worker: Failed to cache static assets', error);
      })
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('Service Worker: Activating...');
  
  event.waitUntil(
    caches.keys()
      .then((cacheNames) => {
        return Promise.all(
          cacheNames.map((cacheName) => {
            if (cacheName !== STATIC_CACHE_NAME && 
                cacheName !== DYNAMIC_CACHE_NAME &&
                cacheName !== CACHE_NAME) {
              console.log('Service Worker: Deleting old cache', cacheName);
              return caches.delete(cacheName);
            }
          })
        );
      })
      .then(() => {
        console.log('Service Worker: Activated');
        return self.clients.claim();
      })
  );
});

// Fetch event - handle requests with different strategies
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }

  // Skip chrome-extension requests
  if (url.protocol === 'chrome-extension:') {
    return;
  }

  // Handle different types of requests
  if (isStaticAsset(request)) {
    event.respondWith(cacheFirstStrategy(request, STATIC_CACHE_NAME));
  } else if (isAPIRequest(request)) {
    if (isNetworkFirstAPI(request)) {
      event.respondWith(networkFirstStrategy(request, DYNAMIC_CACHE_NAME));
    } else {
      event.respondWith(staleWhileRevalidateStrategy(request, DYNAMIC_CACHE_NAME));
    }
  } else if (isImageRequest(request)) {
    event.respondWith(cacheFirstStrategy(request, DYNAMIC_CACHE_NAME));
  } else {
    event.respondWith(networkFirstStrategy(request, DYNAMIC_CACHE_NAME));
  }
});

// Cache-first strategy (for static assets)
async function cacheFirstStrategy(request, cacheName) {
  try {
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }

    const networkResponse = await fetch(request);
    if (networkResponse.ok) {
      const cache = await caches.open(cacheName);
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    console.error('Cache-first strategy failed:', error);
    return new Response('Offline', { status: 503 });
  }
}

// Network-first strategy (for critical API calls)
async function networkFirstStrategy(request, cacheName) {
  try {
    const networkResponse = await fetch(request);
    if (networkResponse.ok) {
      const cache = await caches.open(cacheName);
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    console.log('Network failed, trying cache:', error);
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    
    // Return offline page for navigation requests
    if (request.mode === 'navigate') {
      return caches.match('/offline.html') || new Response('Offline', { status: 503 });
    }
    
    return new Response('Offline', { status: 503 });
  }
}

// Stale-while-revalidate strategy (for API data)
async function staleWhileRevalidateStrategy(request, cacheName) {
  const cache = await caches.open(cacheName);
  const cachedResponse = await cache.match(request);

  const fetchPromise = fetch(request).then((networkResponse) => {
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  }).catch(() => {
    // Network failed, return cached version if available
    return cachedResponse;
  });

  // Return cached version immediately if available, otherwise wait for network
  return cachedResponse || fetchPromise;
}

// Helper functions
function isStaticAsset(request) {
  const url = new URL(request.url);
  return url.pathname.startsWith('/static/') || 
         url.pathname === '/' || 
         url.pathname === '/index.html' ||
         url.pathname.endsWith('.js') ||
         url.pathname.endsWith('.css') ||
         url.pathname.endsWith('.ico');
}

function isAPIRequest(request) {
  const url = new URL(request.url);
  return url.pathname.startsWith('/api/');
}

function isNetworkFirstAPI(request) {
  return NETWORK_FIRST_PATTERNS.some(pattern => pattern.test(request.url));
}

function isImageRequest(request) {
  return request.destination === 'image' || 
         request.url.match(/\.(jpg|jpeg|png|gif|webp|svg)$/i);
}

// Background sync for offline actions
self.addEventListener('sync', (event) => {
  console.log('Service Worker: Background sync triggered', event.tag);
  
  if (event.tag === 'background-sync-purchases') {
    event.waitUntil(syncPendingPurchases());
  }
});

// Handle pending purchases when back online
async function syncPendingPurchases() {
  try {
    const pendingPurchases = await getPendingPurchases();
    
    for (const purchase of pendingPurchases) {
      try {
        const response = await fetch('/api/raffles/buy-box', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${purchase.token}`,
          },
          body: JSON.stringify(purchase.data),
        });

        if (response.ok) {
          await removePendingPurchase(purchase.id);
          // Notify user of successful sync
          self.registration.showNotification('Purchase Completed', {
            body: 'Your raffle box purchase has been processed!',
            icon: '/icon-192x192.png',
            badge: '/badge-72x72.png',
            tag: 'purchase-sync',
          });
        }
      } catch (error) {
        console.error('Failed to sync purchase:', error);
      }
    }
  } catch (error) {
    console.error('Background sync failed:', error);
  }
}

// Push notification handling
self.addEventListener('push', (event) => {
  console.log('Service Worker: Push notification received');
  
  let notificationData = {
    title: 'Raffle Platform',
    body: 'You have a new notification',
    icon: '/icon-192x192.png',
    badge: '/badge-72x72.png',
    tag: 'default',
  };

  if (event.data) {
    try {
      const data = event.data.json();
      notificationData = { ...notificationData, ...data };
    } catch (error) {
      console.error('Failed to parse push data:', error);
    }
  }

  event.waitUntil(
    self.registration.showNotification(notificationData.title, {
      body: notificationData.body,
      icon: notificationData.icon,
      badge: notificationData.badge,
      tag: notificationData.tag,
      data: notificationData.data,
      actions: notificationData.actions || [],
      requireInteraction: notificationData.requireInteraction || false,
    })
  );
});

// Notification click handling
self.addEventListener('notificationclick', (event) => {
  console.log('Service Worker: Notification clicked');
  
  event.notification.close();

  const notificationData = event.notification.data || {};
  let urlToOpen = '/';

  // Determine URL based on notification type
  if (notificationData.type === 'raffle_winner') {
    urlToOpen = `/raffles/${notificationData.raffleId}`;
  } else if (notificationData.type === 'raffle_full') {
    urlToOpen = `/raffles/${notificationData.raffleId}`;
  } else if (notificationData.type === 'credit_expiring') {
    urlToOpen = '/wallet';
  } else if (notificationData.url) {
    urlToOpen = notificationData.url;
  }

  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true })
      .then((clientList) => {
        // Check if there's already a window/tab open with the target URL
        for (const client of clientList) {
          if (client.url === urlToOpen && 'focus' in client) {
            return client.focus();
          }
        }
        
        // If no existing window, open a new one
        if (clients.openWindow) {
          return clients.openWindow(urlToOpen);
        }
      })
  );
});

// Message handling from main thread
self.addEventListener('message', (event) => {
  console.log('Service Worker: Message received', event.data);
  
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  } else if (event.data && event.data.type === 'CACHE_PURCHASE') {
    cachePendingPurchase(event.data.purchase);
  }
});

// Utility functions for IndexedDB operations
async function getPendingPurchases() {
  // Implementation would use IndexedDB to store pending purchases
  // For now, return empty array
  return [];
}

async function removePendingPurchase(id) {
  // Implementation would remove purchase from IndexedDB
  console.log('Removing pending purchase:', id);
}

async function cachePendingPurchase(purchase) {
  // Implementation would store purchase in IndexedDB for later sync
  console.log('Caching pending purchase:', purchase);
}

// Error handling
self.addEventListener('error', (event) => {
  console.error('Service Worker error:', event.error);
});

self.addEventListener('unhandledrejection', (event) => {
  console.error('Service Worker unhandled rejection:', event.reason);
});

console.log('Service Worker: Loaded successfully');