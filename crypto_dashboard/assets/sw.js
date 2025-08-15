/**
 * Service Worker for Crypto Dashboard PWA
 * Handles caching, background sync, and push notifications
 */

const CACHE_NAME = 'crypto-dashboard-v1';
const API_CACHE_NAME = 'crypto-api-v1';

// Files to cache for offline functionality
const STATIC_ASSETS = [
    '/',
    '/static/css/style.css',
    '/static/css/auto_upload.css',
    '/static/js/main.js',
    '/static/js/auto_update.js',
    '/static/js/modules/websocket-client.js',
    '/static/js/modules/status-manager.js',
    '/static/js/modules/progress-tracker.js',
    '/static/js/modules/api-client.js',
    '/static/js/modules/log-manager.js',
    '/static/js/modules/ui-controller.js',
    '/static/js/modules/pwa-manager.js',
    'https://cdn.tailwindcss.com',
    'https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css',
    'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap'
];

// API endpoints to cache
const API_ENDPOINTS = [
    '/scheduler-status',
    '/api/dashboard-summary',
    '/reports'
];

/**
 * Install Event - Cache static assets
 */
self.addEventListener('install', (event) => {
    console.log('[SW] Installing service worker...');
    
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then((cache) => {
                console.log('[SW] Caching static assets');
                return cache.addAll(STATIC_ASSETS);
            })
            .then(() => {
                console.log('[SW] Static assets cached successfully');
                return self.skipWaiting();
            })
            .catch((error) => {
                console.error('[SW] Failed to cache static assets:', error);
            })
    );
});

/**
 * Activate Event - Clean up old caches
 */
self.addEventListener('activate', (event) => {
    console.log('[SW] Activating service worker...');
    
    event.waitUntil(
        caches.keys()
            .then((cacheNames) => {
                return Promise.all(
                    cacheNames.map((cacheName) => {
                        if (cacheName !== CACHE_NAME && cacheName !== API_CACHE_NAME) {
                            console.log('[SW] Deleting old cache:', cacheName);
                            return caches.delete(cacheName);
                        }
                    })
                );
            })
            .then(() => {
                console.log('[SW] Service worker activated');
                return self.clients.claim();
            })
    );
});

/**
 * Fetch Event - Handle requests with caching strategy
 */
self.addEventListener('fetch', (event) => {
    const request = event.request;
    const url = new URL(request.url);
    
    // Skip non-GET requests
    if (request.method !== 'GET') {
        return;
    }
    
    // Handle API requests
    if (url.pathname.startsWith('/api/') || API_ENDPOINTS.includes(url.pathname)) {
        event.respondWith(handleApiRequest(request));
        return;
    }
    
    // Handle static assets
    if (STATIC_ASSETS.some(asset => url.pathname.endsWith(asset) || url.href.includes(asset))) {
        event.respondWith(handleStaticAsset(request));
        return;
    }
    
    // Handle navigation requests
    if (request.mode === 'navigate') {
        event.respondWith(handleNavigation(request));
        return;
    }
    
    // Default: network first, then cache
    event.respondWith(
        fetch(request)
            .catch(() => caches.match(request))
    );
});

/**
 * Handle API requests with cache-first strategy for specific endpoints
 */
async function handleApiRequest(request) {
    const url = new URL(request.url);
    
    try {
        // For dashboard data, try network first, then cache
        if (url.pathname.includes('dashboard-summary') || url.pathname.includes('scheduler-status')) {
            return await networkFirstStrategy(request, API_CACHE_NAME);
        }
        
        // For other API requests, network only
        return await fetch(request);
        
    } catch (error) {
        console.log('[SW] API request failed, trying cache:', url.pathname);
        return await caches.match(request);
    }
}

/**
 * Handle static assets with cache-first strategy
 */
async function handleStaticAsset(request) {
    try {
        const cachedResponse = await caches.match(request);
        
        if (cachedResponse) {
            // Update cache in background
            fetch(request)
                .then(response => {
                    if (response.ok) {
                        caches.open(CACHE_NAME)
                            .then(cache => cache.put(request, response.clone()));
                    }
                })
                .catch(() => {
                    // Ignore network errors for background updates
                });
            
            return cachedResponse;
        }
        
        // If not in cache, fetch and cache
        const response = await fetch(request);
        
        if (response.ok) {
            const cache = await caches.open(CACHE_NAME);
            cache.put(request, response.clone());
        }
        
        return response;
        
    } catch (error) {
        console.error('[SW] Failed to handle static asset:', error);
        return new Response('Asset not available offline', { status: 503 });
    }
}

/**
 * Handle navigation requests - return cached page or offline page
 */
async function handleNavigation(request) {
    try {
        // Try network first
        const response = await fetch(request);
        
        if (response.ok) {
            // Cache successful responses
            const cache = await caches.open(CACHE_NAME);
            cache.put(request, response.clone());
        }
        
        return response;
        
    } catch (error) {
        // If network fails, try cache
        const cachedResponse = await caches.match(request);
        
        if (cachedResponse) {
            return cachedResponse;
        }
        
        // Return offline page
        return await caches.match('/') || new Response('Offline - Please check your connection', {
            status: 503,
            headers: { 'Content-Type': 'text/html' }
        });
    }
}

/**
 * Network-first strategy with cache fallback
 */
async function networkFirstStrategy(request, cacheName) {
    try {
        const response = await fetch(request);
        
        if (response.ok) {
            // Cache successful responses
            const cache = await caches.open(cacheName);
            cache.put(request, response.clone());
        }
        
        return response;
        
    } catch (error) {
        // Network failed, try cache
        console.log('[SW] Network failed, trying cache for:', request.url);
        return await caches.match(request);
    }
}

/**
 * Push Notification Event
 */
self.addEventListener('push', (event) => {
    console.log('[SW] Push notification received');
    
    const defaultOptions = {
        icon: '/static/icons/icon-192x192.png',
        badge: '/static/icons/badge-72x72.png',
        vibrate: [200, 100, 200],
        requireInteraction: true,
        actions: [
            {
                action: 'view',
                title: 'View Dashboard',
                icon: '/static/icons/view-icon.png'
            },
            {
                action: 'dismiss',
                title: 'Dismiss',
                icon: '/static/icons/dismiss-icon.png'
            }
        ]
    };
    
    let notificationData = defaultOptions;
    
    if (event.data) {
        try {
            const data = event.data.json();
            notificationData = {
                ...defaultOptions,
                title: data.title || 'Crypto Dashboard Update',
                body: data.body || 'New update available',
                data: data.data || {}
            };
        } catch (error) {
            console.error('[SW] Failed to parse push data:', error);
            notificationData.title = 'Crypto Dashboard Update';
            notificationData.body = event.data.text() || 'New update available';
        }
    } else {
        notificationData.title = 'Crypto Dashboard Update';
        notificationData.body = 'New update available';
    }
    
    event.waitUntil(
        self.registration.showNotification(notificationData.title, notificationData)
    );
});

/**
 * Notification Click Event
 */
self.addEventListener('notificationclick', (event) => {
    console.log('[SW] Notification clicked:', event);
    
    event.notification.close();
    
    const action = event.action;
    const notificationData = event.notification.data || {};
    
    if (action === 'dismiss') {
        return;
    }
    
    // Default action or 'view' action
    event.waitUntil(
        clients.matchAll({ type: 'window' })
            .then((clientList) => {
                // Try to focus existing window
                for (const client of clientList) {
                    if (client.url.includes(self.location.origin) && 'focus' in client) {
                        return client.focus();
                    }
                }
                
                // Open new window if no existing window found
                if (clients.openWindow) {
                    const targetUrl = notificationData.url || '/';
                    return clients.openWindow(targetUrl);
                }
            })
    );
});

/**
 * Background Sync Event
 */
self.addEventListener('sync', (event) => {
    console.log('[SW] Background sync triggered:', event.tag);
    
    if (event.tag === 'sync-dashboard-data') {
        event.waitUntil(syncDashboardData());
    }
    
    if (event.tag === 'sync-reports') {
        event.waitUntil(syncReports());
    }
});

/**
 * Sync dashboard data in background
 */
async function syncDashboardData() {
    try {
        console.log('[SW] Syncing dashboard data...');
        
        const response = await fetch('/api/dashboard-summary');
        
        if (response.ok) {
            const cache = await caches.open(API_CACHE_NAME);
            cache.put('/api/dashboard-summary', response.clone());
            
            // Notify clients about updated data
            const clients = await self.clients.matchAll();
            clients.forEach(client => {
                client.postMessage({
                    type: 'DASHBOARD_DATA_SYNCED',
                    timestamp: Date.now()
                });
            });
        }
        
    } catch (error) {
        console.error('[SW] Failed to sync dashboard data:', error);
    }
}

/**
 * Sync reports in background
 */
async function syncReports() {
    try {
        console.log('[SW] Syncing reports...');
        
        const response = await fetch('/reports');
        
        if (response.ok) {
            const cache = await caches.open(API_CACHE_NAME);
            cache.put('/reports', response.clone());
            
            // Notify clients about updated reports
            const clients = await self.clients.matchAll();
            clients.forEach(client => {
                client.postMessage({
                    type: 'REPORTS_SYNCED',
                    timestamp: Date.now()
                });
            });
        }
        
    } catch (error) {
        console.error('[SW] Failed to sync reports:', error);
    }
}

/**
 * Message Event - Handle messages from clients
 */
self.addEventListener('message', (event) => {
    console.log('[SW] Message received:', event.data);
    
    const { type, payload } = event.data;
    
    switch (type) {
        case 'SKIP_WAITING':
            self.skipWaiting();
            break;
            
        case 'CACHE_REPORT':
            if (payload && payload.url) {
                caches.open(API_CACHE_NAME)
                    .then(cache => cache.add(payload.url))
                    .catch(error => console.error('[SW] Failed to cache report:', error));
            }
            break;
            
        case 'CLEAR_CACHE':
            caches.keys()
                .then(cacheNames => {
                    return Promise.all(
                        cacheNames.map(cacheName => caches.delete(cacheName))
                    );
                })
                .then(() => {
                    event.ports[0].postMessage({ success: true });
                })
                .catch(error => {
                    console.error('[SW] Failed to clear cache:', error);
                    event.ports[0].postMessage({ success: false, error: error.message });
                });
            break;
    }
});

console.log('[SW] Service Worker loaded successfully');
