/**
 * websocket-manager.js - Manages WebSocket connection and message handling
 */

import {
    WS_DEBUG,
    getTranslatedText,
    updateWebSocketStatus
} from './utils.js';

import { updateDashboardFromData } from './ui-updaters.js';

export class DashboardWebSocket {
    constructor() {
        this.socket = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 10;
        this.reconnectDelay = 1000; // Start with 1s
        this.heartbeatInterval = null;
        this.pingInterval = 30000; // 30s
        this.isConnecting = false;
        this.domUpdatePending = false;
        this.pendingUpdateTask = null;
    }

    /**
     * Connects to the WebSocket server.
     */
    connect() {
        if (this.isConnecting || (this.socket && this.socket.readyState === WebSocket.CONNECTING)) {
            if (WS_DEBUG) console.log('🔍 [DEBUG] WebSocket already connecting, skipping...');
            return;
        }

        this.isConnecting = true;

        // Use WebSocket URL injected from server or fallback to same-host
        let wsUrl;
        if (window.WEBSOCKET_URL) {
            wsUrl = window.WEBSOCKET_URL;

            // SAFETY: Auto-upgrade to wss:// if page is on https://
            if (window.location.protocol === 'https:' && wsUrl.startsWith('ws://')) {
                wsUrl = wsUrl.replace('ws://', 'wss://');
                if (WS_DEBUG) console.log('🔒 Auto-upgraded WebSocket to secure protocol:', wsUrl);
            }

            // Ensure wsUrl doesn't end with / if we are adding /ws
            if (!wsUrl.endsWith('/ws')) {
                wsUrl = wsUrl.replace(/\/$/, '') + '/ws';
            }

            if (WS_DEBUG) console.log('🔗 Using injected WebSocket URL:', wsUrl);
        } else {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            wsUrl = `${protocol}//${window.location.host}/ws`;
            if (WS_DEBUG) console.log('⚠️ No injected WebSocket URL, using same-host fallback');
        }

        console.log('🔌 Connecting to WebSocket:', wsUrl);
        updateWebSocketStatus('connecting', getTranslatedText('connecting') || 'Đang kết nối...');

        try {
            this.socket = new WebSocket(wsUrl);

            this.socket.onopen = () => {
                console.log('✅ WebSocket connected');
                this.reconnectAttempts = 0;
                this.reconnectDelay = 1000;
                this.isConnecting = false;

                // Update status indicator
                updateWebSocketStatus('connected', getTranslatedText('real-time-connected') || 'Kết nối thời gian thực');

                // Send ping to keep connection alive
                this.startHeartbeat();
            };

            this.socket.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handleMessage(data);
                } catch (error) {
                    if (WS_DEBUG) console.error('❌ Failed to parse WebSocket message:', error, event.data);
                    // Handle non-JSON message
                    this.handleMessage({ type: 'text', message: event.data });
                }
            };

            this.socket.onclose = (event) => {
                console.log(`🔌 WebSocket disconnected (code: ${event.code})`);
                this.isConnecting = false;
                this.stopHeartbeat();

                // Clean close: don't attempt reconnect for specific codes if needed
                if (event.code !== 1000 && event.code !== 1001) {
                    updateWebSocketStatus('disconnected', getTranslatedText('disconnected') || 'Ngắt kết nối, đang thử lại...');
                    this.reconnect();
                } else {
                    updateWebSocketStatus('disconnected', getTranslatedText('disconnected') || 'Ngắt kết nối');
                }
            };

            this.socket.onerror = (error) => {
                console.error('❌ WebSocket error:', error);
                this.isConnecting = false;
                updateWebSocketStatus('error', getTranslatedText('connection-error') || 'Lỗi kết nối');
            };
        } catch (error) {
            console.error('❌ Failed to create WebSocket connection:', error);
            this.isConnecting = false;
            this.reconnect();
        }
    }

    /**
     * Handles reconnection with exponential backoff.
     */
    reconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('❌ Maximum WebSocket reconnect attempts reached');
            updateWebSocketStatus('error', getTranslatedText('connection-failed') || 'Không thể kết nối');
            return;
        }

        this.reconnectAttempts++;
        console.log(`🔄 Reconnect attempt ${this.reconnectAttempts} in ${this.reconnectDelay}ms...`);

        setTimeout(() => {
            this.connect();
            // Increase delay for next attempt: 1s, 2s, 4s, 8s, up to 30s
            this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);
        }, this.reconnectDelay);
    }

    /**
     * Starts the heartbeat mechanism.
     */
    startHeartbeat() {
        this.stopHeartbeat();
        this.heartbeatInterval = setInterval(() => {
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                if (WS_DEBUG) console.log('📡 Sending heartbeat (ping)...');
                this.socket.send(JSON.stringify({ type: 'ping' }));
            }
        }, this.pingInterval);
    }

    /**
     * Stops the heartbeat mechanism.
     */
    stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }
    }

    /**
     * Processes incoming WebSocket messages.
     * @param {Object} message - Parsed message data.
     */
    handleMessage(message) {
        if (WS_DEBUG) console.log('📨 Received WebSocket message:', message.type || 'no-type');

        // Handle special control messages
        if (message.type === 'connected') {
            if (WS_DEBUG) console.log('✅ WebSocket connection confirmed:', message.message);
            return;
        }

        if (message.type === 'pong') {
            if (WS_DEBUG) console.log('🏓 Pong received at:', message.timestamp);
            return;
        }

        // For all other messages, just check if data exists and update
        if (message.data) {
            if (WS_DEBUG) console.log('📊 Dashboard data received, updating UI...');

            // Cache the data for language switching
            window.dashboardSummaryCache = message.data;

            // Batch update entire dashboard UI with real-time data
            this.scheduleDOMUpdate(() => {
                updateDashboardFromData(message.data);
                console.log('✅ Dashboard updated from WebSocket data');
            });
        } else {
            if (WS_DEBUG) {
                // If it's a known non-data message, log quietly
                if (message.type === 'text' || message.type === 'info') {
                    console.log('ℹ️ WebSocket info:', message.message);
                } else {
                    console.log('⚠️ Message has no data field:', message);
                }
            }
        }
    }

    /**
     * Schedules a DOM update using requestAnimationFrame for performance.
     * @param {Function} task - The update logic to execute.
     */
    scheduleDOMUpdate(task) {
        this.pendingUpdateTask = task;
        if (!this.domUpdatePending) {
            this.domUpdatePending = true;
            requestAnimationFrame(() => {
                if (this.pendingUpdateTask) {
                    this.pendingUpdateTask();
                    this.pendingUpdateTask = null;
                }
                this.domUpdatePending = false;
            });
        }
    }
}
