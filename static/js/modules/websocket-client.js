/**
 * WebSocket Client Manager for Real-time Updates
 * Handles connection, reconnection, and message routing
 */
export class WebSocketClient {
    constructor() {
        this.socket = null;
        this.isConnected = false;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectInterval = 1000;
        this.subscriptions = new Set();
        this.messageHandlers = new Map();
        this.connectionListeners = new Set();
        
        // Auto-reconnect settings
        this.autoReconnect = true;
        this.reconnectTimeout = null;
        
        // Connection state
        this.clientId = null;
        this.lastPing = null;
        this.pingInterval = null;
    }
    
    /**
     * Initialize WebSocket connection
     */
    async connect() {
        if (this.socket && this.isConnected) {
            console.log('[WebSocket] Already connected');
            return;
        }
        
        try {
            // Import Socket.IO client
            const { io } = await import('https://cdn.socket.io/4.7.2/socket.io.esm.min.js');
            
            console.log('[WebSocket] Connecting to server...');
            
            this.socket = io({
                transports: ['websocket', 'polling'],
                upgrade: true,
                rememberUpgrade: true,
                timeout: 5000
            });
            
            this.setupEventHandlers();
            
        } catch (error) {
            console.error('[WebSocket] Connection failed:', error);
            this.handleConnectionError(error);
        }
    }
    
    /**
     * Setup Socket.IO event handlers
     */
    setupEventHandlers() {
        // Connection established
        this.socket.on('connect', () => {
            console.log('[WebSocket] Connected to server');
            this.isConnected = true;
            this.reconnectAttempts = 0;
            
            // Start ping interval
            this.startPingInterval();
            
            // Notify connection listeners
            this.notifyConnectionListeners('connected');
            
            // Re-subscribe to previous channels
            this.resubscribeToChannels();
        });
        
        // Connection failed
        this.socket.on('connect_error', (error) => {
            console.error('[WebSocket] Connection error:', error);
            this.isConnected = false;
            this.handleConnectionError(error);
        });
        
        // Disconnected
        this.socket.on('disconnect', (reason) => {
            console.log('[WebSocket] Disconnected:', reason);
            this.isConnected = false;
            this.stopPingInterval();
            
            // Notify connection listeners
            this.notifyConnectionListeners('disconnected', reason);
            
            // Auto-reconnect if enabled
            if (this.autoReconnect && reason !== 'io client disconnect') {
                this.scheduleReconnect();
            }
        });
        
        // Welcome message with client ID
        this.socket.on('connected', (data) => {
            console.log('[WebSocket] Welcome message:', data);
            this.clientId = data.client_id;
            
            // Store client ID in localStorage for persistence
            localStorage.setItem('ws_client_id', this.clientId);
        });
        
        // Subscription confirmations
        this.socket.on('subscribed', (data) => {
            console.log('[WebSocket] Subscribed to channel:', data.channel);
        });
        
        this.socket.on('unsubscribed', (data) => {
            console.log('[WebSocket] Unsubscribed from channel:', data.channel);
        });
        
        // Ping/Pong for keepalive
        this.socket.on('pong', (data) => {
            this.lastPing = new Date(data.timestamp);
        });
        
        // Error handling
        this.socket.on('error', (error) => {
            console.error('[WebSocket] Server error:', error);
        });
        
        // Data message handlers
        this.socket.on('status_update', (data) => {
            this.handleMessage('status_update', data);
        });
        
        this.socket.on('progress_update', (data) => {
            this.handleMessage('progress_update', data);
        });
        
        this.socket.on('report_completed', (data) => {
            this.handleMessage('report_completed', data);
        });
    }
    
    /**
     * Handle incoming messages
     */
    handleMessage(eventType, data) {
        console.log(`[WebSocket] Received ${eventType}:`, data);
        
        // Call registered message handlers
        if (this.messageHandlers.has(eventType)) {
            const handlers = this.messageHandlers.get(eventType);
            handlers.forEach(handler => {
                try {
                    handler(data);
                } catch (error) {
                    console.error(`[WebSocket] Handler error for ${eventType}:`, error);
                }
            });
        }
    }
    
    /**
     * Subscribe to a channel
     */
    subscribe(channel) {
        if (!this.isConnected) {
            console.warn('[WebSocket] Not connected, storing subscription for later');
            this.subscriptions.add(channel);
            return;
        }
        
        console.log(`[WebSocket] Subscribing to channel: ${channel}`);
        this.socket.emit('subscribe', { channel });
        this.subscriptions.add(channel);
    }
    
    /**
     * Unsubscribe from a channel
     */
    unsubscribe(channel) {
        if (!this.isConnected) {
            this.subscriptions.delete(channel);
            return;
        }
        
        console.log(`[WebSocket] Unsubscribing from channel: ${channel}`);
        this.socket.emit('unsubscribe', { channel });
        this.subscriptions.delete(channel);
    }
    
    /**
     * Re-subscribe to all stored channels
     */
    resubscribeToChannels() {
        this.subscriptions.forEach(channel => {
            this.socket.emit('subscribe', { channel });
        });
    }
    
    /**
     * Register message handler for specific event type
     */
    onMessage(eventType, handler) {
        if (!this.messageHandlers.has(eventType)) {
            this.messageHandlers.set(eventType, new Set());
        }
        
        this.messageHandlers.get(eventType).add(handler);
        
        // Return unsubscribe function
        return () => {
            const handlers = this.messageHandlers.get(eventType);
            if (handlers) {
                handlers.delete(handler);
            }
        };
    }
    
    /**
     * Register connection state listener
     */
    onConnectionChange(listener) {
        this.connectionListeners.add(listener);
        
        // Return unsubscribe function
        return () => {
            this.connectionListeners.delete(listener);
        };
    }
    
    /**
     * Notify connection listeners
     */
    notifyConnectionListeners(state, data = null) {
        this.connectionListeners.forEach(listener => {
            try {
                listener(state, data);
            } catch (error) {
                console.error('[WebSocket] Connection listener error:', error);
            }
        });
    }
    
    /**
     * Start ping interval for keepalive
     */
    startPingInterval() {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
        }
        
        this.pingInterval = setInterval(() => {
            if (this.isConnected) {
                this.socket.emit('ping');
            }
        }, 30000); // Ping every 30 seconds
    }
    
    /**
     * Stop ping interval
     */
    stopPingInterval() {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
            this.pingInterval = null;
        }
    }
    
    /**
     * Handle connection errors
     */
    handleConnectionError(error) {
        this.isConnected = false;
        this.notifyConnectionListeners('error', error);
        
        if (this.autoReconnect) {
            this.scheduleReconnect();
        }
    }
    
    /**
     * Schedule reconnection attempt
     */
    scheduleReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('[WebSocket] Max reconnection attempts reached');
            this.notifyConnectionListeners('max_reconnect_attempts');
            return;
        }
        
        const delay = Math.min(this.reconnectInterval * Math.pow(2, this.reconnectAttempts), 30000);
        this.reconnectAttempts++;
        
        console.log(`[WebSocket] Scheduling reconnect attempt ${this.reconnectAttempts} in ${delay}ms`);
        
        this.reconnectTimeout = setTimeout(() => {
            console.log(`[WebSocket] Attempting reconnection ${this.reconnectAttempts}/${this.maxReconnectAttempts}`);
            this.connect();
        }, delay);
    }
    
    /**
     * Manually disconnect
     */
    disconnect() {
        this.autoReconnect = false;
        
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }
        
        this.stopPingInterval();
        
        if (this.socket) {
            this.socket.disconnect();
        }
        
        this.isConnected = false;
        this.subscriptions.clear();
        this.messageHandlers.clear();
        this.connectionListeners.clear();
        
        console.log('[WebSocket] Manually disconnected');
    }
    
    /**
     * Get connection status
     */
    getStatus() {
        return {
            isConnected: this.isConnected,
            clientId: this.clientId,
            reconnectAttempts: this.reconnectAttempts,
            subscriptions: Array.from(this.subscriptions),
            lastPing: this.lastPing
        };
    }
}

// Export singleton instance
export const wsClient = new WebSocketClient();
