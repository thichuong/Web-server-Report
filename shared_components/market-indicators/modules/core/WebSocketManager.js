/**
 * WebSocketManager - Manages WebSocket connection and reconnection logic
 * 
 * Responsibilities:
 * - Establish and maintain WebSocket connection
 * - Handle reconnection with exponential backoff
 * - Manage heartbeat mechanism
 * - Emit events for message handling
 */

const DEBUG_MODE = true;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class WebSocketManager {
    constructor(config = {}) {
        this.websocket = null;
        this.isConnected = false;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = config.maxReconnectAttempts || 8;
        this.reconnectDelay = config.initialReconnectDelay || 500;
        this.heartbeatInterval = null;
        this.lastDataUpdate = Date.now();
        
        // Event handlers
        this.onMessage = config.onMessage || (() => {});
        this.onConnected = config.onConnected || (() => {});
        this.onDisconnected = config.onDisconnected || (() => {});
        this.onError = config.onError || (() => {});
        
        // WebSocket URL
        this.wsUrl = this.getWebSocketUrl();
    }
    
    /**
     * Get WebSocket URL - uses injected URL from server or falls back to same-host
     */
    getWebSocketUrl() {
        // Use WebSocket URL injected from server environment configuration
        if (window.WEBSOCKET_URL) {
            debugLog('🔗 Using injected WebSocket URL:', window.WEBSOCKET_URL);
            return window.WEBSOCKET_URL + '/ws';
        }

        // Fallback to same-host for development/backward compatibility
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host;
        const fallbackUrl = `${protocol}//${host}/ws`;
        debugLog('⚠️ No injected WebSocket URL, using fallback:', fallbackUrl);
        return fallbackUrl;
    }
    
    /**
     * Connect to WebSocket server
     */
    connect() {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            debugLog('🔌 WebSocket already connected');
            return;
        }
        
        try {
            debugLog(`🔌 Connecting to WebSocket: ${this.wsUrl}`);
            this.websocket = new WebSocket(this.wsUrl);
            
            this.websocket.onopen = this.handleOpen.bind(this);
            this.websocket.onmessage = this.handleMessage.bind(this);
            this.websocket.onclose = this.handleClose.bind(this);
            this.websocket.onerror = this.handleError.bind(this);
            
        } catch (error) {
            console.error('❌ Failed to create WebSocket connection:', error);
            this.onError(error);
        }
    }
    
    /**
     * Handle WebSocket open event
     */
    handleOpen() {
        debugLog('✅ Market Indicators WebSocket connected');
        this.isConnected = true;
        this.reconnectAttempts = 0;
        this.reconnectDelay = 500;
        
        // Send initial ping
        this.send('ping');
        debugLog('🏓 Sent initial ping on connection');
        
        // Start heartbeat
        this.startHeartbeat();
        
        // Notify connected
        this.onConnected();
    }
    
    /**
     * Handle WebSocket message event
     */
    handleMessage(event) {
        try {
            const message = JSON.parse(event.data);
            this.lastDataUpdate = Date.now();
            this.onMessage(message);
        } catch (error) {
            console.error('❌ Error parsing WebSocket message:', error);
            this.onError(error);
        }
    }
    
    /**
     * Handle WebSocket close event
     */
    handleClose(event) {
        debugLog('🔌 Market Indicators WebSocket disconnected:', event.code);
        this.isConnected = false;
        this.websocket = null;
        this.stopHeartbeat();
        
        if (event.code !== 1000) {
            this.onDisconnected();
            this.scheduleReconnect();
        }
    }
    
    /**
     * Handle WebSocket error event
     */
    handleError(error) {
        console.error('❌ Market Indicators WebSocket error:', error);
        this.onError(error);
    }
    
    /**
     * Schedule reconnection with exponential backoff
     */
    scheduleReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            this.reconnectDelay = Math.min(this.reconnectDelay * 2, 3000);
            
            debugLog(`🔄 Scheduling reconnect... Attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts} (delay: ${this.reconnectDelay}ms)`);
            setTimeout(() => this.connect(), this.reconnectDelay);
        } else {
            debugLog('❌ Max reconnect attempts reached');
            this.onError(new Error('Max reconnect attempts reached'));
        }
    }
    
    /**
     * Send message through WebSocket
     */
    send(message) {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            const data = typeof message === 'string' ? message : JSON.stringify(message);
            this.websocket.send(data);
            return true;
        }
        return false;
    }
    
    /**
     * Start heartbeat to keep connection alive
     */
    startHeartbeat() {
        this.stopHeartbeat();
        
        this.heartbeatInterval = setInterval(() => {
            if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
                debugLog('🏓 Sending heartbeat ping');
                this.send('ping');
            } else {
                debugLog('⚠️ WebSocket not open, stopping heartbeat');
                this.stopHeartbeat();
            }
        }, 30000); // Every 30 seconds
    }
    
    /**
     * Stop heartbeat
     */
    stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
            debugLog('🛑 Heartbeat stopped');
        }
    }
    
    /**
     * Close WebSocket connection
     */
    close() {
        debugLog('🔌 Closing WebSocket connection');
        this.stopHeartbeat();
        
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        this.isConnected = false;
    }
    
    /**
     * Check if WebSocket is connected
     */
    get connected() {
        return this.isConnected && this.websocket && this.websocket.readyState === WebSocket.OPEN;
    }
}
