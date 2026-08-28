/**
 * binance-price-ws.js - High-performance Binance WebSocket client for real-time crypto prices
 * 
 * Features:
 * - Direct connection to Binance Public WebSocket API (sub-second latency)
 * - Supports single or combined ticker streams
 * - Auto-reconnection with exponential backoff & Binance US fallback
 * - Lifecycle management (page unload / tab visibility handling)
 * - Compatible with both ES Modules and standard <script> inclusion
 */

(function (global, factory) {
    if (typeof exports === 'object' && typeof module !== 'undefined') {
        module.exports = factory();
    } else if (typeof define === 'function' && define.amd) {
        define([], factory);
    } else {
        const exports = factory();
        global.BinancePriceWebSocket = exports.BinancePriceWebSocket;
    }
})(typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : this, function () {
    'use strict';

    const PRIMARY_WS_BASE = 'wss://stream.binance.com:9443';
    const FALLBACK_WS_BASE = 'wss://stream.binance.us:9443';

    class BinancePriceWebSocket {
        /**
         * @param {Object} options Configuration options
         * @param {string[]|string} [options.symbols] List of symbols (e.g. ['BTCUSDT', 'ETHUSDT'] or 'BTCUSDT')
         * @param {Function} [options.onPriceUpdate] Callback: (symbol, price, changePercent) => void
         * @param {Function} [options.onStatusChange] Callback: (status, detail) => void ('connecting'|'connected'|'disconnected'|'error')
         * @param {boolean} [options.debug] Enable debug logging
         */
        constructor(options = {}) {
            this.symbols = this.normalizeSymbols(options.symbols || ['BTCUSDT']);
            this.onPriceUpdate = options.onPriceUpdate || (() => {});
            this.onStatusChange = options.onStatusChange || (() => {});
            this.debug = Boolean(options.debug);

            this.ws = null;
            this.isConnected = false;
            this.isConnecting = false;
            this.reconnectAttempts = 0;
            this.maxReconnectAttempts = 15;
            this.reconnectDelay = 500; // ms
            this.reconnectTimer = null;
            this.useFallback = false;
            this.lastMessageTime = 0;
            this.staleCheckTimer = null;

            // Bind lifecycle handlers
            this._handleVisibilityChange = this._handleVisibilityChange.bind(this);
            this._handleBeforeUnload = this._handleBeforeUnload.bind(this);

            if (typeof document !== 'undefined') {
                document.addEventListener('visibilitychange', this._handleVisibilityChange);
            }
            if (typeof window !== 'undefined') {
                window.addEventListener('beforeunload', this._handleBeforeUnload);
                window.addEventListener('pagehide', this._handleBeforeUnload);
            }
        }

        log(...args) {
            if (this.debug) {
                console.log('⚡ [BinancePriceWS]', ...args);
            }
        }

        warn(...args) {
            console.warn('⚠️ [BinancePriceWS]', ...args);
        }

        error(...args) {
            console.error('❌ [BinancePriceWS]', ...args);
        }

        normalizeSymbols(symbols) {
            const list = Array.isArray(symbols) ? symbols : [symbols];
            return list.map(s => {
                let sym = String(s).trim().toUpperCase();
                if (!sym.endsWith('USDT') && !sym.endsWith('BUSD') && !sym.endsWith('USDC')) {
                    sym += 'USDT';
                }
                return sym;
            });
        }

        buildUrl() {
            const base = this.useFallback ? FALLBACK_WS_BASE : PRIMARY_WS_BASE;
            if (this.symbols.length === 1) {
                const stream = `${this.symbols[0].toLowerCase()}@ticker`;
                return `${base}/ws/${stream}`;
            }

            const streams = this.symbols.map(s => `${s.toLowerCase()}@ticker`).join('/');
            return `${base}/stream?streams=${streams}`;
        }

        connect() {
            if (this.isConnected || this.isConnecting) {
                return;
            }

            if (this.reconnectTimer) {
                clearTimeout(this.reconnectTimer);
                this.reconnectTimer = null;
            }

            this.isConnecting = true;
            const url = this.buildUrl();
            this.log(`Connecting to: ${url} (Symbols: ${this.symbols.join(', ')})`);
            this.onStatusChange('connecting', { url, symbols: this.symbols });

            try {
                this.ws = new WebSocket(url);

                this.ws.onopen = () => {
                    this.log('✅ WebSocket connected successfully');
                    this.isConnected = true;
                    this.isConnecting = false;
                    this.reconnectAttempts = 0;
                    this.reconnectDelay = 500;
                    this.lastMessageTime = Date.now();
                    this.onStatusChange('connected', { url });

                    this.startStaleCheck();
                };

                this.ws.onmessage = (event) => {
                    this.lastMessageTime = Date.now();
                    try {
                        const parsed = JSON.parse(event.data);
                        this.handleMessage(parsed);
                    } catch (err) {
                        this.warn('Failed to parse message:', err, event.data);
                    }
                };

                this.ws.onclose = (event) => {
                    this.log(`🔌 WebSocket closed (code: ${event.code})`);
                    this.isConnected = false;
                    this.isConnecting = false;
                    this.stopStaleCheck();

                    this.onStatusChange('disconnected', { code: event.code });

                    // Only reconnect if not a clean intentional close
                    if (event.code !== 1000) {
                        this.scheduleReconnect();
                    }
                };

                this.ws.onerror = (err) => {
                    this.error('WebSocket error:', err);
                    this.isConnecting = false;
                    this.onStatusChange('error', { error: err });
                };
            } catch (err) {
                this.error('Failed to create WebSocket instance:', err);
                this.isConnecting = false;
                this.scheduleReconnect();
            }
        }

        handleMessage(msg) {
            // Handle combined stream wrapper: { stream: "...", data: { ... } }
            const data = msg.data || msg;

            if (!data) return;

            // Full 24hr ticker: e: "24hrTicker", s: "BTCUSDT", c: "95000.00", P: "2.50"
            if (data.s && data.c !== undefined) {
                const symbol = data.s.toUpperCase();
                const price = parseFloat(data.c);
                let changePercent = 0;

                if (data.P !== undefined) {
                    changePercent = parseFloat(data.P);
                } else if (data.o !== undefined && parseFloat(data.o) > 0) {
                    const open = parseFloat(data.o);
                    changePercent = ((price - open) / open) * 100;
                }

                if (!isNaN(price)) {
                    this.onPriceUpdate(symbol, price, isNaN(changePercent) ? 0 : changePercent);
                }
            }
        }

        scheduleReconnect() {
            if (this.reconnectAttempts >= this.maxReconnectAttempts) {
                this.error('Max reconnection attempts reached. Trying fallback URL...');
                // Switch between primary and fallback endpoint
                this.useFallback = !this.useFallback;
                this.reconnectAttempts = 0;
                this.reconnectDelay = 1000;
            }

            this.reconnectAttempts++;
            const delay = Math.min(this.reconnectDelay * 1.5, 10000);
            this.reconnectDelay = delay;

            this.log(`Scheduling reconnect in ${Math.round(delay)}ms (Attempt ${this.reconnectAttempts})`);

            this.reconnectTimer = setTimeout(() => {
                this.reconnectTimer = null;
                this.connect();
            }, delay);
        }

        startStaleCheck() {
            this.stopStaleCheck();
            // Check every 20 seconds if we received any message in the last 40 seconds
            this.staleCheckTimer = setInterval(() => {
                if (this.isConnected && Date.now() - this.lastMessageTime > 40000) {
                    this.warn('No messages received for 40s, reconnecting WebSocket...');
                    this.disconnect();
                    this.connect();
                }
            }, 20000);
        }

        stopStaleCheck() {
            if (this.staleCheckTimer) {
                clearInterval(this.staleCheckTimer);
                this.staleCheckTimer = null;
            }
        }

        _handleVisibilityChange() {
            if (document.visibilityState === 'visible') {
                if (!this.isConnected && !this.isConnecting) {
                    this.log('Tab became visible, reconnecting...');
                    this.connect();
                }
            }
        }

        _handleBeforeUnload() {
            this.destroy();
        }

        disconnect() {
            if (this.reconnectTimer) {
                clearTimeout(this.reconnectTimer);
                this.reconnectTimer = null;
            }
            this.stopStaleCheck();

            if (this.ws) {
                this.ws.onopen = null;
                this.ws.onmessage = null;
                this.ws.onerror = null;
                this.ws.onclose = null;
                try {
                    this.ws.close(1000, 'Normal Closure');
                } catch (e) {}
                this.ws = null;
            }

            this.isConnected = false;
            this.isConnecting = false;
        }

        destroy() {
            this.disconnect();

            if (typeof document !== 'undefined') {
                document.removeEventListener('visibilitychange', this._handleVisibilityChange);
            }
            if (typeof window !== 'undefined') {
                window.removeEventListener('beforeunload', this._handleBeforeUnload);
                window.removeEventListener('pagehide', this._handleBeforeUnload);
            }
        }
    }

    return { BinancePriceWebSocket };
});
