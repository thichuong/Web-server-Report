// WebSocket connection manager for real-time dashboard updates
class DashboardWebSocket {
    constructor() {
        this.socket = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000; // Start with 1 second
        this.isConnecting = false;
    }

    connect() {
        if (this.isConnecting || (this.socket && this.socket.readyState === WebSocket.CONNECTING)) {
            return;
        }

        this.isConnecting = true;
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;

        console.log('🔌 Connecting to WebSocket:', wsUrl);

        try {
            this.socket = new WebSocket(wsUrl);
            
            this.socket.onopen = () => {
                console.log('✅ WebSocket connected');
                this.reconnectAttempts = 0;
                this.reconnectDelay = 1000;
                this.isConnecting = false;
                
                // Send ping to keep connection alive
                this.startHeartbeat();
            };

            this.socket.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleMessage(message);
                } catch (error) {
                    console.error('❌ Error parsing WebSocket message:', error);
                }
            };

            this.socket.onclose = (event) => {
                console.log('🔌 WebSocket disconnected:', event.code, event.reason);
                this.isConnecting = false;
                this.stopHeartbeat();
                
                // Attempt to reconnect if not manually closed
                if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
                    setTimeout(() => this.reconnect(), this.reconnectDelay);
                }
            };

            this.socket.onerror = (error) => {
                console.error('❌ WebSocket error:', error);
                this.isConnecting = false;
            };

        } catch (error) {
            console.error('❌ Failed to create WebSocket connection:', error);
            this.isConnecting = false;
        }
    }

    reconnect() {
        this.reconnectAttempts++;
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000); // Max 30 seconds
        
        console.log(`🔄 Reconnecting... Attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts}`);
        this.connect();
    }

    handleMessage(message) {
        console.log('📨 Received WebSocket message:', message.type);
        
        if (message.type === 'dashboard_update' && message.data) {
            // Cache the data for language switching
            window.dashboardSummaryCache = message.data;
            
            // Update dashboard UI
            this.updateDashboardUI(message.data);
        }
    }

    updateDashboardUI(data) {
        try {
            // Update market cap
            const marketCapContainer = selectDashboardElementByLang('market-cap-container');
            if (marketCapContainer) {
                marketCapContainer.innerHTML = `
                    <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(data.market_cap)}</p>
                    <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
                marketCapContainer.dataset.marketCap = String(data.market_cap);
            }

            // Update volume 24h
            const volumeContainer = selectDashboardElementByLang('volume-24h-container');
            if (volumeContainer) {
                volumeContainer.innerHTML = `
                    <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(data.volume_24h)}</p>
                    <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
                volumeContainer.dataset.volume24h = String(data.volume_24h);
            }

            // Update BTC price
            const btcContainer = selectDashboardElementByLang('btc-price-container');
            if (btcContainer) {
                const change = data.btc_change_24h;
                const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
                btcContainer.innerHTML = `
                    <p class="text-3xl font-bold text-gray-900">${'$' + (data.btc_price_usd ? data.btc_price_usd.toLocaleString('en-US') : 'N/A')}</p>
                    <p class="text-sm font-semibold ${changeClass}">${change !== null ? change.toFixed(2) : 'N/A'}% (24h)</p>`;
                btcContainer.dataset.btcPriceUsd = String(data.btc_price_usd);
                btcContainer.dataset.btcChange24h = String(data.btc_change_24h);
            }

            // Update Fear & Greed Index
            const fngContainer = selectDashboardElementByLang('fear-greed-container');
            const fngValue = parseInt(data.fng_value, 10);
            if (!isNaN(fngValue) && fngContainer) {
                const fngConfig = {
                    min: 0, max: 100,
                    segments: [
                        { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                        { limit: 49, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                        { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                        { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                        { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
                    ]
                };
                createGauge(fngContainer, fngValue, fngConfig);
                fngContainer.dataset.value = String(fngValue);
            }

            // Update RSI
            const rsiContainer = selectDashboardElementByLang('rsi-container');
            const rsiValue = data.rsi_14;
            if (rsiValue !== null && rsiValue !== undefined && rsiContainer) {
                const rsiConfig = {
                    min: 0, max: 100,
                    segments: [
                        { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                        { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                        { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
                    ]
                };
                createGauge(rsiContainer, rsiValue, rsiConfig);
                rsiContainer.dataset.value = String(rsiValue);
            }

            console.log('✅ Dashboard UI updated via WebSocket');

        } catch (error) {
            console.error('❌ Error updating dashboard UI:', error);
        }
    }

    startHeartbeat() {
        this.heartbeatInterval = setInterval(() => {
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                this.socket.send('ping');
            }
        }, 30000); // Ping every 30 seconds
    }

    stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }
    }

    disconnect() {
        this.stopHeartbeat();
        if (this.socket) {
            this.socket.close(1000, 'Manual disconnect');
            this.socket = null;
        }
    }
}

// Global WebSocket instance
let dashboardWebSocket = null;
