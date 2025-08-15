/**
 * PWA Manager - Handles Progressive Web App functionality
 * Including service worker registration, caching, and push notifications
 */
export class PWAManager {
    constructor() {
        this.serviceWorker = null;
        this.isOnline = navigator.onLine;
        this.syncTags = new Set();
        this.installPrompt = null;
        this.pushSubscription = null;
        
        // VAPID public key for push notifications (replace with your key)
        this.vapidPublicKey = 'BEl62iUYgUivxIkv69yViEuiBIa40HcCWLrdpjPuxh...'; // This should be set from server
        
        this.init();
    }
    
    async init() {
        console.log('[PWA] Initializing PWA Manager...');
        
        // Check if service workers are supported
        if (!('serviceWorker' in navigator)) {
            console.warn('[PWA] Service Workers not supported');
            return;
        }
        
        // Register service worker
        await this.registerServiceWorker();
        
        // Setup online/offline detection
        this.setupNetworkDetection();
        
        // Setup install prompt handling
        this.setupInstallPrompt();
        
        // Setup push notifications
        await this.setupPushNotifications();
        
        // Setup background sync
        this.setupBackgroundSync();
        
        // Listen for service worker messages
        this.setupServiceWorkerMessages();
        
        console.log('[PWA] PWA Manager initialized successfully');
    }
    
    /**
     * Register service worker
     */
    async registerServiceWorker() {
        try {
            const registration = await navigator.serviceWorker.register('/static/sw.js', {
                scope: '/'
            });
            
            this.serviceWorker = registration;
            
            console.log('[PWA] Service Worker registered:', registration.scope);
            
            // Handle service worker updates
            registration.addEventListener('updatefound', () => {
                const newWorker = registration.installing;
                
                newWorker.addEventListener('statechange', () => {
                    if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
                        console.log('[PWA] New service worker available');
                        this.showUpdateAvailable();
                    }
                });
            });
            
            return registration;
            
        } catch (error) {
            console.error('[PWA] Service Worker registration failed:', error);
            throw error;
        }
    }
    
    /**
     * Setup network online/offline detection
     */
    setupNetworkDetection() {
        const updateOnlineStatus = () => {
            const wasOnline = this.isOnline;
            this.isOnline = navigator.onLine;
            
            console.log(`[PWA] Network status: ${this.isOnline ? 'Online' : 'Offline'}`);
            
            // Show/hide offline indicator
            this.updateOfflineIndicator();
            
            // If back online, trigger background sync
            if (!wasOnline && this.isOnline) {
                this.triggerBackgroundSync();
            }
            
            // Dispatch custom event
            window.dispatchEvent(new CustomEvent('network-status-change', {
                detail: { isOnline: this.isOnline }
            }));
        };
        
        window.addEventListener('online', updateOnlineStatus);
        window.addEventListener('offline', updateOnlineStatus);
        
        // Initial status
        updateOnlineStatus();
    }
    
    /**
     * Setup install prompt handling
     */
    setupInstallPrompt() {
        window.addEventListener('beforeinstallprompt', (event) => {
            console.log('[PWA] Install prompt available');
            
            // Prevent the default prompt
            event.preventDefault();
            
            // Store the event for later use
            this.installPrompt = event;
            
            // Show custom install button
            this.showInstallButton();
        });
        
        // Handle app installation
        window.addEventListener('appinstalled', () => {
            console.log('[PWA] App installed');
            this.installPrompt = null;
            this.hideInstallButton();
            
            // Track installation
            this.trackEvent('pwa_installed');
        });
    }
    
    /**
     * Setup push notifications
     */
    async setupPushNotifications() {
        if (!('PushManager' in window)) {
            console.warn('[PWA] Push notifications not supported');
            return;
        }
        
        if (!this.serviceWorker) {
            console.warn('[PWA] Service worker not available for push notifications');
            return;
        }
        
        try {
            // Get existing subscription
            this.pushSubscription = await this.serviceWorker.pushManager.getSubscription();
            
            if (this.pushSubscription) {
                console.log('[PWA] Existing push subscription found');
                await this.sendSubscriptionToServer(this.pushSubscription);
            }
            
        } catch (error) {
            console.error('[PWA] Failed to setup push notifications:', error);
        }
    }
    
    /**
     * Request permission for push notifications
     */
    async requestNotificationPermission() {
        if (!('Notification' in window)) {
            console.warn('[PWA] Notifications not supported');
            return false;
        }
        
        if (Notification.permission === 'granted') {
            await this.subscribeToPush();
            return true;
        }
        
        if (Notification.permission === 'denied') {
            console.warn('[PWA] Notification permission denied');
            return false;
        }
        
        // Request permission
        const permission = await Notification.requestPermission();
        
        if (permission === 'granted') {
            console.log('[PWA] Notification permission granted');
            await this.subscribeToPush();
            return true;
        }
        
        console.warn('[PWA] Notification permission not granted');
        return false;
    }
    
    /**
     * Subscribe to push notifications
     */
    async subscribeToPush() {
        if (!this.serviceWorker) {
            console.error('[PWA] Service worker not available');
            return;
        }
        
        try {
            const subscription = await this.serviceWorker.pushManager.subscribe({
                userVisibleOnly: true,
                applicationServerKey: this.urlBase64ToUint8Array(this.vapidPublicKey)
            });
            
            this.pushSubscription = subscription;
            console.log('[PWA] Push subscription created');
            
            // Send subscription to server
            await this.sendSubscriptionToServer(subscription);
            
            return subscription;
            
        } catch (error) {
            console.error('[PWA] Failed to subscribe to push notifications:', error);
            throw error;
        }
    }
    
    /**
     * Send push subscription to server
     */
    async sendSubscriptionToServer(subscription) {
        try {
            const response = await fetch('/api/push-subscription', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(subscription)
            });
            
            if (response.ok) {
                console.log('[PWA] Push subscription sent to server');
            } else {
                console.error('[PWA] Failed to send subscription to server');
            }
            
        } catch (error) {
            console.error('[PWA] Error sending subscription to server:', error);
        }
    }
    
    /**
     * Setup background sync
     */
    setupBackgroundSync() {
        if (!('serviceWorker' in navigator) || !('sync' in window.ServiceWorkerRegistration.prototype)) {
            console.warn('[PWA] Background sync not supported');
            return;
        }
        
        console.log('[PWA] Background sync available');
    }
    
    /**
     * Trigger background sync
     */
    async triggerBackgroundSync() {
        if (!this.serviceWorker || !('sync' in window.ServiceWorkerRegistration.prototype)) {
            return;
        }
        
        try {
            // Register sync for dashboard data
            await this.serviceWorker.sync.register('sync-dashboard-data');
            console.log('[PWA] Background sync registered for dashboard data');
            
            // Register sync for reports
            await this.serviceWorker.sync.register('sync-reports');
            console.log('[PWA] Background sync registered for reports');
            
        } catch (error) {
            console.error('[PWA] Failed to register background sync:', error);
        }
    }
    
    /**
     * Setup service worker message handling
     */
    setupServiceWorkerMessages() {
        navigator.serviceWorker.addEventListener('message', (event) => {
            const { type, timestamp } = event.data;
            
            console.log('[PWA] Message from service worker:', event.data);
            
            switch (type) {
                case 'DASHBOARD_DATA_SYNCED':
                    this.handleDashboardDataSynced(timestamp);
                    break;
                    
                case 'REPORTS_SYNCED':
                    this.handleReportsSynced(timestamp);
                    break;
            }
        });
    }
    
    /**
     * Handle dashboard data sync completion
     */
    handleDashboardDataSynced(timestamp) {
        console.log('[PWA] Dashboard data synced at:', new Date(timestamp));
        
        // Refresh dashboard if visible
        if (document.visibilityState === 'visible') {
            window.dispatchEvent(new CustomEvent('dashboard-data-updated', {
                detail: { timestamp }
            }));
        }
    }
    
    /**
     * Handle reports sync completion
     */
    handleReportsSynced(timestamp) {
        console.log('[PWA] Reports synced at:', new Date(timestamp));
        
        // Refresh reports list if on reports page
        if (window.location.pathname.includes('reports')) {
            window.dispatchEvent(new CustomEvent('reports-updated', {
                detail: { timestamp }
            }));
        }
    }
    
    /**
     * Show install button
     */
    showInstallButton() {
        const installButton = document.getElementById('pwa-install-button');
        if (installButton) {
            installButton.style.display = 'block';
            installButton.onclick = () => this.installApp();
        }
    }
    
    /**
     * Hide install button
     */
    hideInstallButton() {
        const installButton = document.getElementById('pwa-install-button');
        if (installButton) {
            installButton.style.display = 'none';
        }
    }
    
    /**
     * Install the app
     */
    async installApp() {
        if (!this.installPrompt) {
            console.warn('[PWA] Install prompt not available');
            return;
        }
        
        // Show the install prompt
        this.installPrompt.prompt();
        
        // Wait for user response
        const choiceResult = await this.installPrompt.userChoice;
        
        console.log('[PWA] Install choice:', choiceResult.outcome);
        
        if (choiceResult.outcome === 'accepted') {
            this.trackEvent('pwa_install_accepted');
        } else {
            this.trackEvent('pwa_install_dismissed');
        }
        
        // Clear the prompt
        this.installPrompt = null;
    }
    
    /**
     * Show update available notification
     */
    showUpdateAvailable() {
        // Create update notification
        const notification = document.createElement('div');
        notification.id = 'pwa-update-notification';
        notification.className = 'pwa-update-notification';
        notification.innerHTML = `
            <div class="notification-content">
                <span>🔄 New version available!</span>
                <button onclick="window.pwaManager.updateApp()">Update</button>
                <button onclick="this.parentElement.parentElement.remove()">Later</button>
            </div>
        `;
        
        document.body.appendChild(notification);
        
        // Auto-hide after 10 seconds
        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 10000);
    }
    
    /**
     * Update the app
     */
    async updateApp() {
        if (!this.serviceWorker || !this.serviceWorker.waiting) {
            return;
        }
        
        // Tell the waiting service worker to become active
        this.serviceWorker.waiting.postMessage({ type: 'SKIP_WAITING' });
        
        // Reload the page to activate new service worker
        window.location.reload();
    }
    
    /**
     * Update offline indicator
     */
    updateOfflineIndicator() {
        let indicator = document.getElementById('offline-indicator');
        
        if (!this.isOnline) {
            if (!indicator) {
                indicator = document.createElement('div');
                indicator.id = 'offline-indicator';
                indicator.className = 'offline-indicator';
                indicator.innerHTML = '📡 You are offline';
                document.body.appendChild(indicator);
            }
        } else {
            if (indicator) {
                indicator.remove();
            }
        }
    }
    
    /**
     * Clear all caches
     */
    async clearCache() {
        if (!this.serviceWorker) {
            return;
        }
        
        return new Promise((resolve, reject) => {
            const messageChannel = new MessageChannel();
            
            messageChannel.port1.onmessage = (event) => {
                if (event.data.success) {
                    console.log('[PWA] Cache cleared successfully');
                    resolve();
                } else {
                    console.error('[PWA] Failed to clear cache:', event.data.error);
                    reject(new Error(event.data.error));
                }
            };
            
            this.serviceWorker.active.postMessage(
                { type: 'CLEAR_CACHE' },
                [messageChannel.port2]
            );
        });
    }
    
    /**
     * Track PWA events
     */
    trackEvent(eventName, data = {}) {
        console.log(`[PWA] Event tracked: ${eventName}`, data);
        
        // Send to analytics if available
        if (window.gtag) {
            window.gtag('event', eventName, data);
        }
    }
    
    /**
     * Utility: Convert VAPID key
     */
    urlBase64ToUint8Array(base64String) {
        const padding = '='.repeat((4 - base64String.length % 4) % 4);
        const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
        const rawData = window.atob(base64);
        const outputArray = new Uint8Array(rawData.length);
        
        for (let i = 0; i < rawData.length; ++i) {
            outputArray[i] = rawData.charCodeAt(i);
        }
        
        return outputArray;
    }
    
    /**
     * Get PWA status
     */
    getStatus() {
        return {
            isOnline: this.isOnline,
            serviceWorkerActive: !!this.serviceWorker,
            pushSubscriptionActive: !!this.pushSubscription,
            installPromptAvailable: !!this.installPrompt,
            notificationPermission: 'Notification' in window ? Notification.permission : 'not-supported'
        };
    }
}

// Export singleton instance
export const pwaManager = new PWAManager();
