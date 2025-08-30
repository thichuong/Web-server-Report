/**
 * Market Indicators Dashboard JavaScript
 * Handles real-time market data display without charts
 * Integrates with WebSo            this.websocket.onclose = (event) => {
                debugLog('🔌 Market Indicators WebSocket disconnected:', event.code, event.reason);
                this.isConnected = false;
                this.stopHeartbeat(); // Stop heartbeat when connection closes
                this.websocket = null;
                
                if (event.code !== 1000) {
                    this.updateConnectionStatus('disconnected');
                    this.scheduleReconnect();
                }
            };ive updates
 */

// Debug mode - set to true for debugging
const DEBUG_MODE = true;

// Debug logging wrapper
function debugLog(...args) {
    if (DEBUG_MODE) {
        console.log(...args);
    }
}

function debugError(...args) {
    if (DEBUG_MODE) {
        console.error(...args);
    }
}

class MarketIndicatorsDashboard {
    constructor() {
        this.websocket = null;
        this.isConnected = false;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 8; // Increased from 5 to 8 attempts
        this.reconnectDelay = 500; // Faster initial reconnect (500ms instead of 1000ms)
        this.updateAnimationDuration = 300;
        this.heartbeatInterval = null;
        this.lastDataUpdate = Date.now(); // Track last data received
        this.dataTimeoutInterval = null; // For checking data staleness
        
        // Data cache
        this.cachedData = {
            btcPrice: null,
            marketCap: null,
            volume24h: null,
            fearGreedIndex: null,
            btcDominance: null,
            ethDominance: null,
            usStockIndices: null
        };
        
        this.init();
    }

    init() {
        debugLog('🚀 Initializing Market Indicators Dashboard');
        this.initializeElements();
        
        // Request initial data immediately before establishing WebSocket
        this.requestInitialData();
        
        this.connectWebSocket();
        this.startDataRefresh();
    }

    initializeElements() {
        // Get all indicator elements
        this.elements = {
            btcPrice: document.getElementById('btc-price-indicator'),
            marketCap: document.getElementById('market-cap-indicator'),
            volume24h: document.getElementById('volume-24h-indicator'),
            fearGreed: document.getElementById('fear-greed-indicator'),
            btcDominance: document.getElementById('btc-dominance-indicator'),
            ethDominance: document.getElementById('eth-dominance-indicator'),
            connectionStatus: document.getElementById('connection-status')
        };

        // Debug: Log which elements were found
        debugLog('🔍 Element search results:');
        Object.entries(this.elements).forEach(([key, element]) => {
            if (element) {
                debugLog(`  ✅ ${key}: found`);
            } else {
                debugLog(`  ❌ ${key}: NOT found`);
            }
        });

        // Remove any existing skeletons after a short delay
        setTimeout(() => {
            this.removeSkeleton();
        }, 2000);
    }

    removeSkeleton() {
        Object.values(this.elements).forEach(element => {
            if (element) {
                const skeleton = element.querySelector('.skeleton-loader');
                if (skeleton && !this.cachedData.btcPrice) {
                    // Only remove skeleton if we don't have data yet
                    skeleton.style.display = 'none';
                }
            }
        });
    }

    async requestInitialData() {
        debugLog('🔄 Requesting initial market data...');
        
        try {
            // Try to fetch initial data from the same server
            const response = await fetch('/api/dashboard/data', {
                method: 'GET',
                headers: {
                    'Accept': 'application/json',
                    'Content-Type': 'application/json'
                },
                timeout: 10000 // 10 second timeout
            });

            if (response.ok) {
                const data = await response.json();
                debugLog('✅ Initial data received:', data);
                
                // Update dashboard with initial data
                if (data) {
                    this.updateMarketData(data);
                    debugLog('✅ Dashboard updated with initial data');
                }
            } else {
                debugLog('⚠️ Failed to fetch initial data:', response.status, response.statusText);
            }
        } catch (error) {
            debugLog('⚠️ Initial data request failed, will rely on WebSocket:', error.message);
            // Don't throw error - just continue with WebSocket initialization
            // The WebSocket will provide data once connected
        }
    }

    requestFreshData() {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            debugLog('📤 Requesting fresh data via WebSocket...');
            try {
                // Send structured request for dashboard data
                const request = {
                    type: 'request_dashboard_data',
                    timestamp: Date.now()
                };
                this.websocket.send(JSON.stringify(request));
                debugLog('✅ Data request sent via WebSocket');
            } catch (error) {
                debugLog('❌ Failed to send data request via WebSocket:', error);
                // Fallback to simple string request
                try {
                    this.websocket.send('request_update');
                    debugLog('✅ Fallback data request sent');
                } catch (fallbackError) {
                    debugError('❌ Both WebSocket request methods failed:', fallbackError);
                }
            }
        } else {
            debugLog('⚠️ Cannot request fresh data - WebSocket not ready');
        }
    }

    connectWebSocket() {
        if (this.isConnected || this.websocket) {
            return;
        }

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        debugLog('🔌 Connecting to WebSocket for market indicators:', wsUrl);
        this.updateConnectionStatus('connecting');

        try {
            this.websocket = new WebSocket(wsUrl);

            this.websocket.onopen = () => {
                debugLog('✅ Market Indicators WebSocket connected');
                this.isConnected = true;
                this.reconnectAttempts = 0;
                this.reconnectDelay = 500; // Reset to faster initial delay
                this.updateConnectionStatus('connected');
                
                // Request fresh data immediately after WebSocket connects
                this.requestFreshData();
                
                // Start heartbeat to keep connection alive
                this.startHeartbeat();
            };

            this.websocket.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleWebSocketMessage(message);
                } catch (error) {
                    console.error('❌ Error parsing WebSocket message:', error);
                }
            };

            this.websocket.onclose = (event) => {
                debugLog('🔌 Market Indicators WebSocket disconnected:', event.code);
                this.isConnected = false;
                this.websocket = null;
                
                if (event.code !== 1000) {
                    this.updateConnectionStatus('disconnected');
                    this.scheduleReconnect();
                }
            };

            this.websocket.onerror = (error) => {
                console.error('❌ Market Indicators WebSocket error:', error);
                this.updateConnectionStatus('offline');
            };

        } catch (error) {
            console.error('❌ Failed to create WebSocket connection:', error);
            this.updateConnectionStatus('offline');
        }
    }

    scheduleReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            this.reconnectDelay = Math.min(this.reconnectDelay * 2, 3000); // Max 3s instead of 5s for faster reconnect
            
            debugLog(`🔄 Scheduling reconnect... Attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts} (delay: ${this.reconnectDelay}ms)`);
            setTimeout(() => this.connectWebSocket(), this.reconnectDelay);
        } else {
            debugLog('❌ Max reconnect attempts reached');
            this.updateConnectionStatus('offline');
        }
    }

    handleWebSocketMessage(message) {
        debugLog('📨 Received WebSocket message type:', message.type);
        debugLog('📨 Full message data:', message);
        
        try {
            switch (message.type) {
                case 'dashboard_data':
                case 'dashboard_update':
                    if (message.data) {
                        debugLog('📊 Received market data update:', message.data);
                        this.updateMarketData(message.data);
                    }
                    break;
                    
                case 'btc_price_update':
                    if (message.data) {
                        debugLog('₿ Received BTC price update:', message.data);
                        this.updateBtcPrice(message.data);
                    }
                    break;
                    
                case 'market_update':
                    if (message.data) {
                        debugLog('📈 Received market update:', message.data);
                        this.updateMarketData(message.data);
                    }
                    break;
                    
                case 'connected':
                    debugLog('✅ WebSocket connection confirmed');
                    break;
                    
                case 'pong':
                    debugLog('🏓 Pong received');
                    break;
                    
                default:
                    debugLog('❓ Unknown WebSocket message type:', message.type);
                    // Try to handle as generic market data if it has the expected fields
                    if (message.btc_price_usd || message.market_cap_usd || message.fng_value) {
                        debugLog('🔄 Treating unknown message as market data:', message);
                        this.updateMarketData(message);
                    }
                    break;
            }
        } catch (error) {
            debugError('❌ Error handling WebSocket message:', error);
            debugError('❌ Problematic message:', message);
            // Don't let one bad message break the WebSocket connection
            // Continue processing future messages
        }
    }

    updateMarketData(data) {
        debugLog('🔄 Updating market indicators with data:', data);

        try {
            // Update BTC Price
            if (data.btc_price_usd !== undefined) {
                this.updateBtcPrice({
                    price_usd: data.btc_price_usd,
                    change_24h: data.btc_change_24h || 0
                });
            }

            // Update Market Cap - use market_cap_usd directly
            if (data.market_cap_usd !== undefined) {
                this.updateMarketCap({
                    value: data.market_cap_usd,
                    change: data.market_cap_change_percentage_24h_usd || 0
                });
            }

            // Update Volume 24h - use volume_24h_usd if available
            if (data.volume_24h_usd !== undefined) {
                this.updateVolume24h({
                    value: data.volume_24h_usd,
                    change: 0 // Volume change not available from current API
                });
            }

            // Update Fear & Greed Index - use fng_value
            if (data.fng_value !== undefined) {
                this.updateFearGreedIndex(data.fng_value);
            }

            // Update BTC Dominance - use btc_market_cap_percentage directly
            if (data.btc_market_cap_percentage !== undefined) {
                this.updateBtcDominance(data.btc_market_cap_percentage);
            }

            // Update ETH Dominance - use eth_market_cap_percentage directly
            if (data.eth_market_cap_percentage !== undefined) {
                this.updateEthDominance(data.eth_market_cap_percentage);
            }

            // Update US Stock Indices
            if (data.us_stock_indices) {
                this.updateUSStockIndices(data.us_stock_indices);
            }

            // Track last data update time for staleness checking
            this.lastDataUpdate = Date.now();

            debugLog('✅ Market indicators update completed successfully');
        } catch (error) {
            debugError('❌ Error updating market data:', error);
            debugError('❌ Problematic data:', data);
            // Don't let update errors break the WebSocket connection
        }
    }

    updateBtcPrice(data) {
        try {
            const element = this.elements.btcPrice;
            if (!element) {
                debugError('❌ BTC price element not found');
                return;
            }

            const price = parseFloat(data.price_usd || data.btc_price_usd) || 0;
            const change = parseFloat(data.change_24h || data.btc_change_24h) || 0;

            // Check if data has actually changed
            const cachedBtc = this.cachedData.btcPrice;
            if (cachedBtc && cachedBtc.price === price && cachedBtc.change === change) {
                debugLog('⏭️ BTC price unchanged, skipping update:', { price, change });
                return;
            }

            this.cachedData.btcPrice = { price, change };

            const changeClass = change >= 0 ? 'positive' : 'negative';
            const changeIcon = change >= 0 ? '📈' : '📉';
            const changeSign = change >= 0 ? '+' : '';

            element.innerHTML = `
                <div class="market-value">$${price.toLocaleString('en-US')}</div>
                <div class="market-change ${changeClass}">
                    <span class="change-icon">${changeIcon}</span>
                    ${changeSign}${change.toFixed(2)}% (24h)
                </div>
            `;

            this.animateUpdate(element);
            debugLog('✅ BTC price updated:', { price, change });
        } catch (error) {
            debugError('❌ Error updating BTC price:', error);
        }
    }

    updateMarketCap(data) {
        const element = this.elements.marketCap;
        if (!element) return;

        const value = parseFloat(data.value || data.total_market_cap || data.market_cap_usd) || 0;
        const change = parseFloat(data.change || data.market_cap_change_percentage_24h_usd) || 0;

        // Check if data has actually changed
        const cachedMarketCap = this.cachedData.marketCap;
        if (cachedMarketCap && cachedMarketCap.value === value && cachedMarketCap.change === change) {
            debugLog('⏭️ Market cap unchanged, skipping update:', { value, change });
            return;
        }

        this.cachedData.marketCap = { value, change };

        const changeClass = change >= 0 ? 'positive' : 'negative';
        const changeIcon = change >= 0 ? '📈' : '📉';
        const changeSign = change >= 0 ? '+' : '';

        element.innerHTML = `
            <div class="market-value">$${this.formatLargeNumber(value)}</div>
            <div class="market-change ${changeClass}">
                <span class="change-icon">${changeIcon}</span>
                ${changeSign}${change.toFixed(2)}% (24h)
            </div>
        `;

        this.animateUpdate(element);
        debugLog('✅ Market cap updated:', { value, change });
    }

    updateVolume24h(data) {
        const element = this.elements.volume24h;
        if (!element) return;

        const value = parseFloat(data.value || data.total_volume_24h) || 0;
        const change = parseFloat(data.change || data.volume_change_24h) || 0;

        // Check if data has actually changed
        const cachedVolume = this.cachedData.volume24h;
        if (cachedVolume && cachedVolume.value === value && cachedVolume.change === change) {
            debugLog('⏭️ Volume 24h unchanged, skipping update:', { value, change });
            return;
        }

        this.cachedData.volume24h = { value, change };

        const changeClass = change >= 0 ? 'positive' : 'negative';
        const changeIcon = change >= 0 ? '📈' : '📉';
        const changeSign = change >= 0 ? '+' : '';

        element.innerHTML = `
            <div class="market-value">$${this.formatLargeNumber(value)}</div>
            <div class="market-change ${changeClass}">
                <span class="change-icon">${changeIcon}</span>
                ${changeSign}${change.toFixed(2)}% (24h)
            </div>
        `;

        this.animateUpdate(element);
        debugLog('✅ Volume 24h updated:', { value, change });
    }

    updateFearGreedIndex(value) {
        const element = this.elements.fearGreed;
        if (!element) return;

        const index = parseInt(value) || 0;
        
        // Check if data has actually changed
        const cachedFearGreed = this.cachedData.fearGreedIndex;
        if (cachedFearGreed === index) {
            debugLog('⏭️ Fear & Greed Index unchanged, skipping update:', { index });
            return;
        }
        
        this.cachedData.fearGreedIndex = index;

        let indexClass = 'neutral';
        let indexLabelKey, indexDescriptionKey;
        
        if (index <= 24) {
            indexClass = 'fear';
            indexLabelKey = 'extreme-fear';
            indexDescriptionKey = 'extreme-fear-desc';
        } else if (index <= 44) {
            indexClass = 'fear';
            indexLabelKey = 'fear';
            indexDescriptionKey = 'fear-desc';
        } else if (index <= 55) {
            indexClass = 'neutral';
            indexLabelKey = 'neutral';
            indexDescriptionKey = 'neutral-desc';
        } else if (index <= 74) {
            indexClass = 'greed';
            indexLabelKey = 'greed';
            indexDescriptionKey = 'greed-desc';
        } else {
            indexClass = 'greed';
            indexLabelKey = 'extreme-greed';
            indexDescriptionKey = 'extreme-greed-desc';
        }

        element.innerHTML = `
            <div class="index-display">
                <div class="index-value ${indexClass}">${index}</div>
                <div class="index-label" data-i18n="${indexLabelKey}">Loading...</div>
                <div class="index-description" data-i18n="${indexDescriptionKey}">Loading...</div>
            </div>
        `;

        // Trigger translation update if available
        this.updateTranslations(element);
        
        this.animateUpdate(element);
        debugLog('✅ Fear & Greed Index updated:', { index, class: indexClass });
    }

    updateBtcDominance(value) {
        const element = this.elements.btcDominance;
        if (!element) return;

        const dominance = parseFloat(value) || 0;
        
        // Check if data has actually changed (with small tolerance for floating point comparison)
        const cachedBtcDominance = this.cachedData.btcDominance;
        if (cachedBtcDominance !== null && Math.abs(cachedBtcDominance - dominance) < 0.01) {
            debugLog('⏭️ BTC dominance unchanged, skipping update:', { dominance });
            return;
        }
        
        this.cachedData.btcDominance = dominance;

        element.innerHTML = `
            <div class="index-display">
                <div class="index-value">${dominance.toFixed(1)}%</div>
                <div class="index-label" data-i18n="btc-market-share">Thị phần BTC</div>
            </div>
        `;

        this.animateUpdate(element);
        debugLog('✅ BTC dominance updated:', { dominance: dominance.toFixed(1) });
    }

    updateEthDominance(value) {
        const element = this.elements.ethDominance;
        if (!element) return;

        const dominance = parseFloat(value) || 0;
        
        // Check if data has actually changed (with small tolerance for floating point comparison)
        const cachedEthDominance = this.cachedData.ethDominance;
        if (cachedEthDominance !== null && Math.abs(cachedEthDominance - dominance) < 0.01) {
            debugLog('⏭️ ETH dominance unchanged, skipping update:', { dominance });
            return;
        }
        
        this.cachedData.ethDominance = dominance;

        element.innerHTML = `
            <div class="index-display">
                <div class="index-value">${dominance.toFixed(1)}%</div>
                <div class="index-label" data-i18n="eth-market-share">Thị phần ETH</div>
            </div>
        `;

        this.animateUpdate(element);
        debugLog('✅ ETH dominance updated:', { dominance: dominance.toFixed(1) });
    }

    updateConnectionStatus(status) {
        const element = this.elements.connectionStatus;
        if (!element) return;

        element.className = `connection-indicator ${status}`;
        
        const statusTexts = {
            connecting: 'Đang kết nối...',
            connected: 'Kết nối thành công',
            disconnected: 'Mất kết nối',
            offline: 'Ngoại tuyến'
        };

        const statusTextElement = element.querySelector('.status-text');
        if (statusTextElement) {
            statusTextElement.textContent = statusTexts[status] || status;
        }
    }

    updateTranslations(element) {
        // Try to update translations if translation system is available
        try {
            if (typeof window.updateTranslationsForElement === 'function') {
                window.updateTranslationsForElement(element);
            } else if (typeof window.updateTranslations === 'function') {
                window.updateTranslations();
            } else {
                // Fallback: manually update elements with data-i18n
                const elementsWithI18n = element.querySelectorAll('[data-i18n]');
                elementsWithI18n.forEach(el => {
                    const key = el.getAttribute('data-i18n');
                    if (window.translations_data && window.translations_data[key]) {
                        const currentLang = window.current_language || 'vi';
                        if (window.translations_data[key][currentLang]) {
                            el.textContent = window.translations_data[key][currentLang];
                        }
                    }
                });
            }
        } catch (error) {
            console.log('Translation update not available:', error);
        }
    }

    animateUpdate(element) {
        if (!element) return;

        // Remove any existing animation classes
        element.classList.remove('pulse-update', 'fade-in');
        
        // Force reflow
        element.offsetHeight;
        
        // Add animation class
        element.classList.add('pulse-update');
        
        // Remove animation class after animation completes
        setTimeout(() => {
            element.classList.remove('pulse-update');
        }, this.updateAnimationDuration);
    }

    formatLargeNumber(num) {
        if (num >= 1e12) {
            return (num / 1e12).toFixed(2) + 'T';
        }
        if (num >= 1e9) {
            return (num / 1e9).toFixed(2) + 'B';
        }
        if (num >= 1e6) {
            return (num / 1e6).toFixed(2) + 'M';
        }
        if (num >= 1e3) {
            return (num / 1e3).toFixed(2) + 'K';
        }
        return num.toLocaleString('en-US');
    }

    startDataRefresh() {
        // WebSocket health check every 10 seconds (increased frequency)
        setInterval(() => {
            const now = Date.now();
            const timeSinceLastUpdate = now - this.lastDataUpdate;
            
            debugLog('🔍 WebSocket Health Check:', {
                connected: this.isConnected,
                websocket: this.websocket !== null,
                readyState: this.websocket ? this.websocket.readyState : 'null',
                reconnectAttempts: this.reconnectAttempts,
                timeSinceLastUpdate: `${Math.round(timeSinceLastUpdate / 1000)}s`
            });
            
            // If no data received for 60 seconds and connected, something may be wrong
            if (this.isConnected && timeSinceLastUpdate > 60000) {
                debugLog('⚠️ No data received for 60+ seconds, requesting fresh data');
                this.requestFreshData(); // Use the new method instead of direct send
            }
            
            if (this.websocket) {
                const state = this.websocket.readyState;
                const states = ['CONNECTING', 'OPEN', 'CLOSING', 'CLOSED'];
                debugLog(`🔍 WebSocket State: ${states[state]} (${state})`);
                
                // If WebSocket is in bad state, try to reconnect
                if (state === WebSocket.CLOSED && this.isConnected) {
                    debugLog('🔄 WebSocket closed unexpectedly, scheduling reconnect');
                    this.isConnected = false;
                    this.scheduleReconnect();
                }
            } else if (!this.isConnected) {
                debugLog('🔄 No WebSocket connection, attempting to connect');
                this.connectWebSocket();
            }
        }, 10000); // Every 10 seconds (faster monitoring)
    }

    // US Stock Indices Update Methods
    updateUSStockIndices(indices) {
        debugLog('📊 Updating US Stock Indices:', indices);
        
        if (!indices || typeof indices !== 'object') {
            debugError('❌ Invalid US stock indices data:', indices);
            return;
        }
        
        this.cachedData.usStockIndices = indices;
        
        // Update individual indices
        if (indices.DIA) {
            this.updateStockIndex('dia', indices.DIA, 'DJIA');
        }
        
        if (indices.SPY) {
            this.updateStockIndex('spy', indices.SPY, 'S&P 500');
        }
        
        if (indices.QQQM) {
            this.updateStockIndex('qqq', indices.QQQM, 'Nasdaq 100');
        }
    }
    
    updateStockIndex(elementId, indexData, displayName) {
        const element = document.getElementById(`${elementId}-indicator`);
        if (!element) {
            debugError(`❌ Element not found: ${elementId}-indicator`);
            return;
        }
        
        if (!indexData || indexData.status !== 'success') {
            // Handle error or loading state
            const status = indexData ? indexData.status : 'no data';
            element.innerHTML = `
                <div class="stock-value">--</div>
                <div class="stock-change neutral">
                    <span class="stock-status error">${status}</span>
                </div>
            `;
            return;
        }
        
        const price = parseFloat(indexData.price) || 0;
        const change = parseFloat(indexData.change) || 0;
        const changePercent = parseFloat(indexData.change_percent) || 0;
        
        // Check if data has actually changed for this specific stock index
        const cacheKey = `${elementId}_stock_index`;
        const cachedStock = this.cachedData[cacheKey];
        if (cachedStock && 
            cachedStock.price === price && 
            cachedStock.change === change && 
            cachedStock.changePercent === changePercent) {
            debugLog(`⏭️ ${displayName} unchanged, skipping update:`, { price, change, changePercent });
            return;
        }
        
        // Cache the new data
        this.cachedData[cacheKey] = { price, change, changePercent };
        
        const changeClass = changePercent >= 0 ? 'positive' : 'negative';
        const changeIcon = changePercent >= 0 ? '📈' : '📉';
        const changeSign = change >= 0 ? '+' : '';
        const percentSign = changePercent >= 0 ? '+' : '';
        
        element.innerHTML = `
            <div class="stock-value">$${price.toFixed(2)}</div>
            <div class="stock-change ${changeClass}">
                <span class="change-icon">${changeIcon}</span>
                ${changeSign}$${Math.abs(change).toFixed(2)} (${percentSign}${changePercent.toFixed(2)}%)
            </div>
            <div class="stock-status success">
                ✅ Live
            </div>
        `;
        
        this.animateUpdate(element);
        debugLog(`✅ Updated ${displayName}: $${price.toFixed(2)} (${percentSign}${changePercent.toFixed(2)}%)`);
    }

    startHeartbeat() {
        // Send ping every 15 seconds to keep connection alive (increased frequency)
        this.heartbeatInterval = setInterval(() => {
            if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
                debugLog('🏓 Sending heartbeat ping');
                this.websocket.send('ping');
            } else {
                this.stopHeartbeat();
            }
        }, 5000); // Every 5 seconds (faster heartbeat)
    }

    stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }
    }

    destroy() {
        this.stopHeartbeat();
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        this.isConnected = false;
    }
}

// Initialize Market Indicators Dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    debugLog('📊 DOM loaded, initializing Market Indicators Dashboard');
    
    // Only initialize if the market indicators container exists
    const container = document.getElementById('market-indicators-dashboard');
    if (container) {
        debugLog('✅ Market indicators container found, creating dashboard instance');
        window.marketIndicatorsDashboard = new MarketIndicatorsDashboard();
        
        // Add a global function to manually update with data (for debugging)
        window.updateMarketIndicators = function(data) {
            if (window.marketIndicatorsDashboard) {
                debugLog('🔧 Manual update triggered with data:', data);
                window.marketIndicatorsDashboard.updateMarketData(data);
            }
        };
    } else {
        debugLog('❌ Market indicators container not found');
    }
});

// Clean up on page unload
window.addEventListener('beforeunload', function() {
    if (window.marketIndicatorsDashboard) {
        window.marketIndicatorsDashboard.destroy();
    }
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = MarketIndicatorsDashboard;
}

// Global debugging helpers (only active in DEBUG_MODE)
window.debugMarketIndicators = function() {
    if (!DEBUG_MODE) {
        console.log('Debug mode is disabled. Set DEBUG_MODE = true to enable debugging.');
        return;
    }
    
    console.log('=== Market Indicators Debug Info ===');
    
    if (window.marketIndicatorsDashboard) {
        console.log('✅ Dashboard instance exists');
        console.log('🔍 Elements status:', window.marketIndicatorsDashboard.elements);
        console.log('💾 Cached data:', window.marketIndicatorsDashboard.cachedData);
        console.log('🔗 WebSocket connected:', window.marketIndicatorsDashboard.isConnected);
    } else {
        console.log('❌ No dashboard instance found');
    }
};

// Test function with sample data (only active in DEBUG_MODE)
window.testMarketIndicators = function() {
    const sampleData = {
        btc_change_24h: -1.1761369473535623,
        btc_price_usd: 110349,
        fng_value: 48,
        market_cap_usd: 3872941106289.462,
        rsi_14: 38.4490332215743,
        us_stock_indices: {
            DIA: {
                name: "SPDR Dow Jones Industrial Average ETF",
                price: 454.49,
                change: 1.42,
                change_percent: 0.31,
                status: "success"
            },
            SPY: {
                name: "SPDR S&P 500 ETF Trust",
                price: 645.16,
                change: 2.69,
                change_percent: 0.42,
                status: "success"
            },
            QQQM: {
                name: "INVESCO NASDAQ 100 ETF",
                price: 572.61,
                change: 2.29,
                change_percent: 0.40,
                status: "success"
            }
        }
    };
    
    debugLog('🧪 Testing with sample data (including US stock indices):', sampleData);
    if (window.updateMarketIndicators) {
        window.updateMarketIndicators(sampleData);
    } else {
        debugLog('❌ updateMarketIndicators function not available');
    }
};
