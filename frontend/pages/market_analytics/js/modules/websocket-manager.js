/**
 * Binance WebSocket Manager (Dual Independent Streams for Spot & Futures)
 * With Robust Lifecycle Management, Timer Disposal & Zero-Leak Disconnect
 */

export class BinanceWebSocketManager {
    constructor(config = {}) {
        this.spotWs = null;
        this.futuresWs = null;
        this.currentSymbol = null;
        this.currentTimeframe = null;

        this.spotConnected = false;
        this.futuresConnected = false;
        this.spotMsgCount = 0;
        this.futuresMsgCount = 0;
        this.spotReconnectAttempts = 0;
        this.futuresReconnectAttempts = 0;
        this.maxReconnectAttempts = 12;

        this.spotReconnectTimer = null;
        this.futuresReconnectTimer = null;

        this.onSpotKline = config.onSpotKline || (() => {});
        this.onFuturesKline = config.onFuturesKline || (() => {});
        this.onSpotTicker = config.onSpotTicker || (() => {});
        this.onFuturesTicker = config.onFuturesTicker || (() => {});
        this.onStatusChange = config.onStatusChange || (() => {});
        this.onLog = config.onLog || (() => {});
    }

    subscribe(symbol, timeframe) {
        this.currentSymbol = symbol.toLowerCase();
        this.currentTimeframe = timeframe;
        this.disconnect();
        this.connectSpot();
        this.connectFutures();
    }

    clearReconnectTimers() {
        if (this.spotReconnectTimer) {
            clearTimeout(this.spotReconnectTimer);
            this.spotReconnectTimer = null;
        }
        if (this.futuresReconnectTimer) {
            clearTimeout(this.futuresReconnectTimer);
            this.futuresReconnectTimer = null;
        }
    }

    connectSpot() {
        if (!this.currentSymbol || !this.currentTimeframe) return;
        const sym = this.currentSymbol;
        const tf = this.currentTimeframe;

        this.onLog('spot', `Initiating Spot WS connection: ${sym}@kline_${tf} & ${sym}@ticker`);
        this.onStatusChange('spot', 'connecting');

        try {
            const url = `wss://stream.binance.com:9443/stream?streams=${sym}@kline_${tf}/${sym}@ticker`;
            this.spotWs = new WebSocket(url);

            this.spotWs.onopen = () => {
                this.spotConnected = true;
                this.spotReconnectAttempts = 0;
                this.onLog('spot', '🟢 Spot WebSocket connected successfully');
                this.onStatusChange('spot', 'connected');
            };

            this.spotWs.onmessage = (event) => {
                this.spotMsgCount++;
                try {
                    const parsed = JSON.parse(event.data);
                    const data = parsed.data || parsed;
                    const streamName = parsed.stream || '';

                    if (data.e === 'kline' && data.k) {
                        this.onSpotKline(data.k);
                    } else if (data.e === '24hrTicker' || data.e === '24hrMiniTicker' || streamName.includes('@ticker')) {
                        this.onSpotTicker(data);
                    }
                } catch (err) {
                    this.onLog('spot', `❌ Error parsing Spot message: ${err.message}`);
                }
            };

            this.spotWs.onerror = (err) => {
                this.onLog('spot', `⚠️ Spot WS error: ${err.message || 'Connection failed'}`);
            };

            this.spotWs.onclose = (event) => {
                this.spotConnected = false;
                this.onLog('spot', `🔌 Spot WS disconnected (code: ${event.code})`);
                this.onStatusChange('spot', 'disconnected');
                if (event.code !== 1000) {
                    this.scheduleSpotReconnect();
                }
            };
        } catch (e) {
            this.onLog('spot', `❌ Spot WS Init Exception: ${e.message}`);
            this.scheduleSpotReconnect();
        }
    }

    connectFutures() {
        if (!this.currentSymbol || !this.currentTimeframe) return;
        const sym = this.currentSymbol;
        const tf = this.currentTimeframe;

        this.onLog('futures', `Initiating Futures WS connection: ${sym}@kline_${tf} & ${sym}@ticker`);
        this.onStatusChange('futures', 'connecting');

        try {
            const url = `wss://fstream.binance.com/market/stream?streams=${sym}@kline_${tf}/${sym}@ticker`;
            this.futuresWs = new WebSocket(url);

            this.futuresWs.onopen = () => {
                this.futuresConnected = true;
                this.futuresReconnectAttempts = 0;
                this.onLog('futures', '🟢 Futures WebSocket connected successfully');
                this.onStatusChange('futures', 'connected');
            };

            this.futuresWs.onmessage = (event) => {
                this.futuresMsgCount++;
                try {
                    const parsed = JSON.parse(event.data);
                    const data = parsed.data || parsed;
                    const streamName = parsed.stream || '';

                    if (data.e === 'kline' && data.k) {
                        this.onFuturesKline(data.k);
                    } else if (data.e === '24hrTicker' || data.e === '24hrMiniTicker' || streamName.includes('@ticker')) {
                        this.onFuturesTicker(data);
                    }
                } catch (err) {
                    this.onLog('futures', `❌ Error parsing Futures message: ${err.message}`);
                }
            };

            this.futuresWs.onerror = (err) => {
                this.onLog('futures', `⚠️ Futures WS error: ${err.message || 'Connection failed'}`);
            };

            this.futuresWs.onclose = (event) => {
                this.futuresConnected = false;
                this.onLog('futures', `🔌 Futures WS disconnected (code: ${event.code})`);
                this.onStatusChange('futures', 'disconnected');
                if (event.code !== 1000) {
                    this.scheduleFuturesReconnect();
                }
            };
        } catch (e) {
            this.onLog('futures', `❌ Futures WS Init Exception: ${e.message}`);
            this.scheduleFuturesReconnect();
        }
    }

    scheduleSpotReconnect() {
        if (this.spotReconnectAttempts >= this.maxReconnectAttempts) {
            this.onLog('spot', '❌ Spot WS max reconnect attempts reached');
            return;
        }
        this.spotReconnectAttempts++;
        const delay = Math.min(1000 * Math.pow(1.5, this.spotReconnectAttempts), 10000);
        this.onLog('spot', `🔄 Reconnecting Spot WS in ${Math.round(delay)}ms... (Attempt ${this.spotReconnectAttempts})`);
        
        if (this.spotReconnectTimer) clearTimeout(this.spotReconnectTimer);
        this.spotReconnectTimer = setTimeout(() => {
            this.spotReconnectTimer = null;
            this.connectSpot();
        }, delay);
    }

    scheduleFuturesReconnect() {
        if (this.futuresReconnectAttempts >= this.maxReconnectAttempts) {
            this.onLog('futures', '❌ Futures WS max reconnect attempts reached');
            return;
        }
        this.futuresReconnectAttempts++;
        const delay = Math.min(1000 * Math.pow(1.5, this.futuresReconnectAttempts), 10000);
        this.onLog('futures', `🔄 Reconnecting Futures WS in ${Math.round(delay)}ms... (Attempt ${this.futuresReconnectAttempts})`);
        
        if (this.futuresReconnectTimer) clearTimeout(this.futuresReconnectTimer);
        this.futuresReconnectTimer = setTimeout(() => {
            this.futuresReconnectTimer = null;
            this.connectFutures();
        }, delay);
    }

    disconnect() {
        this.clearReconnectTimers();

        if (this.spotWs) {
            this.spotWs.onopen = null;
            this.spotWs.onmessage = null;
            this.spotWs.onerror = null;
            this.spotWs.onclose = null;
            try {
                this.spotWs.close(1000, 'Client closed connection');
            } catch (e) {}
            this.spotWs = null;
        }

        if (this.futuresWs) {
            this.futuresWs.onopen = null;
            this.futuresWs.onmessage = null;
            this.futuresWs.onerror = null;
            this.futuresWs.onclose = null;
            try {
                this.futuresWs.close(1000, 'Client closed connection');
            } catch (e) {}
            this.futuresWs = null;
        }

        this.spotConnected = false;
        this.futuresConnected = false;
        this.onStatusChange('all', 'disconnected');
    }

    destroy() {
        this.disconnect();
        this.onSpotKline = () => {};
        this.onFuturesKline = () => {};
        this.onSpotTicker = () => {};
        this.onFuturesTicker = () => {};
        this.onStatusChange = () => {};
        this.onLog = () => {};
    }
}
