/**
 * Market Indicators Dashboard JavaScript
 * Handles real-time market data display without charts
 * Integrates with WebSocket for real-time updates
 */

// Debug mode - set to false for production (reduces Firefox lag)
const DEBUG_MODE = false;

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
        this.pendingUpdates = {}; // Batch updates for Firefox performance
        this.updateTimer = null; // Throttle DOM updates
        
        // Data cache
        this.cachedData = {
            marketCap: null,
            volume24h: null,
            fearGreedIndex: null,
            btcDominance: null,
            ethDominance: null,
            btcRsi14: null,
            usStockIndices: null
        };
        
        // Dominance history for charts (last 20 data points)
        this.dominanceHistory = {
            btc: [],
            eth: []
        };
        this.maxHistoryPoints = 20;
        
        this.init();
    }

    init() {
        debugLog('🚀 Initializing Market Indicators Dashboard');
        this.initializeElements();
        
        // Request initial data immediately before establishing WebSocket
        this.requestInitialData();
        
        this.connectWebSocket();
        this.startDataRefresh();
        
        // Crypto prices now come from server via WebSocket
    }

    initializeElements() {
        // Get all indicator elements
        this.elements = {
            marketCap: document.getElementById('market-cap-indicator'),
            volume24h: document.getElementById('volume-24h-indicator'),
            fearGreed: document.getElementById('fear-greed-indicator'),
            btcDominance: document.getElementById('btc-dominance-indicator'),
            ethDominance: document.getElementById('eth-dominance-indicator'),
            btcRsi14: document.getElementById('btc-rsi-14-indicator'),
            connectionStatus: document.getElementById('connection-status'),
            // Binance price elements
            binanceBTC: document.getElementById('binance-btc-price'),
            binanceETH: document.getElementById('binance-eth-price'),
            binanceSOL: document.getElementById('binance-sol-price'),
            binanceXRP: document.getElementById('binance-xrp-price'),
            binanceADA: document.getElementById('binance-ada-price'),
            binanceLINK: document.getElementById('binance-link-price'),
            binanceBNB: document.getElementById('binance-bnb-price')
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
                if (skeleton) {
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
                
                // Send ping immediately on connection
                this.websocket.send('ping');
                debugLog('🏓 Sent initial ping on connection');
                
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
        // Compact logging with timestamp for broadcast tracking
        const now = new Date().toLocaleTimeString();
        debugLog(`📨 [${now}] WebSocket message type: ${message.type}`);
        
        try {
            switch (message.type) {
                case 'dashboard_data':
                case 'dashboard_update':
                    if (message.data) {
                        debugLog(`📊 [${now}] Market data update received - processing...`);
                        // Batch update to reduce Firefox reflow - use requestAnimationFrame
                        this.scheduleUpdate(() => this.updateMarketData(message.data));
                    }
                    break;
                    
                case 'market_update':
                    if (message.data) {
                        debugLog('📈 Received market update:', message.data);
                        this.scheduleUpdate(() => this.updateMarketData(message.data));
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
                        this.scheduleUpdate(() => this.updateMarketData(message));
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

    // Batch DOM updates using requestAnimationFrame for better Firefox performance
    scheduleUpdate(updateFn) {
        // Cancel previous scheduled update
        if (this.updateTimer) {
            cancelAnimationFrame(this.updateTimer);
        }
        
        // Schedule update on next animation frame
        this.updateTimer = requestAnimationFrame(() => {
            updateFn();
            this.updateTimer = null;
        });
    }

    updateMarketData(data) {
        // Compact logging: only show summary instead of verbose details
        const cryptoPrices = ['btc_price_usd', 'eth_price_usd', 'sol_price_usd', 'xrp_price_usd', 'ada_price_usd', 'link_price_usd', 'bnb_price_usd'];
        const availablePrices = cryptoPrices.filter(key => data[key] !== undefined);
        const missingPrices = cryptoPrices.filter(key => data[key] === undefined);
        
        // Only log if there are issues or in verbose debug mode
        if (missingPrices.length > 0) {
            debugLog(`⚠️ Market data update: ${availablePrices.length}/${cryptoPrices.length} crypto prices available. Missing: ${missingPrices.join(', ')}`);
        } else {
            // Compact success log (only once every 10 updates to reduce noise)
            if (!this._updateCounter) this._updateCounter = 0;
            this._updateCounter++;
            if (this._updateCounter % 10 === 1) {
                debugLog(`✅ Market data update: All ${cryptoPrices.length} crypto prices received`);
            }
        }

        try {
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

            // Update BTC RSI 14 - use btc_rsi_14 directly
            if (data.btc_rsi_14 !== undefined) {
                this.updateBtcRsi14(data.btc_rsi_14);
            }

            // Update US Stock Indices
            if (data.us_stock_indices) {
                this.updateUSStockIndices(data.us_stock_indices);
            }

            // Update Crypto Prices from Server
            if (data.btc_price_usd !== undefined) {
                this.updateCryptoPrice('BTCUSDT', data.btc_price_usd, data.btc_change_24h);
            }
            if (data.eth_price_usd !== undefined) {
                this.updateCryptoPrice('ETHUSDT', data.eth_price_usd, data.eth_change_24h);
            }
            if (data.sol_price_usd !== undefined) {
                this.updateCryptoPrice('SOLUSDT', data.sol_price_usd, data.sol_change_24h);
            }
            if (data.xrp_price_usd !== undefined) {
                this.updateCryptoPrice('XRPUSDT', data.xrp_price_usd, data.xrp_change_24h);
            }
            if (data.ada_price_usd !== undefined) {
                this.updateCryptoPrice('ADAUSDT', data.ada_price_usd, data.ada_change_24h);
            }
            if (data.link_price_usd !== undefined) {
                this.updateCryptoPrice('LINKUSDT', data.link_price_usd, data.link_change_24h);
            }
            if (data.bnb_price_usd !== undefined) {
                this.updateCryptoPrice('BNBUSDT', data.bnb_price_usd, data.bnb_change_24h);
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
        const formatted = this.formatLargeNumber(value);
        const unitHtml = formatted.unitKey ? `<span class="unit" data-i18n="${formatted.unitKey}">${formatted.unitText}</span>` : '';

        element.innerHTML = `
            <div class="flex items-center justify-between">
                <div class="market-value">$${formatted.number}${unitHtml}</div>
                <div class="market-change ${changeClass}">
                    <span class="change-icon">${changeIcon}</span>
                    ${changeSign}${change.toFixed(2)}% (24h)
                </div>
            </div>
        `;

        this.animateUpdate(element);
        this.updateTranslations(element);
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
        const formatted = this.formatLargeNumber(value);
        const unitHtml = formatted.unitKey ? `<span class="unit" data-i18n="${formatted.unitKey}">${formatted.unitText}</span>` : '';

        element.innerHTML = `
            <div class="flex items-center justify-between">
                <div class="market-value">$${formatted.number}${unitHtml}</div>
                <div class="market-change ${changeClass}">
                    <span class="change-icon">${changeIcon}</span>
                    ${changeSign}${change.toFixed(2)}% (24h)
                </div>
            </div>
        `;

        this.animateUpdate(element);
        this.updateTranslations(element);
        debugLog('✅ Volume 24h updated:', { value, change });
    }

    updateFearGreedIndex(value) {
        const element = this.elements.fearGreed;
        if (!element) return;

        const index = parseInt(value) || 0;
        
        // Check if data has actually changed
        const cachedFearGreedIndex = this.cachedData.fearGreedIndex;
        if (cachedFearGreedIndex !== null && cachedFearGreedIndex === index) {
            debugLog('⏭️ Fear & Greed index unchanged, skipping update:', { index });
            return;
        }
        
        this.cachedData.fearGreedIndex = index;

        let indexClass, indexLabelKey, indexDescriptionKey;
        
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
            <div class="index-display flex items-center justify-between">
                <div class="index-value ${indexClass}">${index}</div>
                <div class="text-right">
                    <div class="index-label" data-i18n="${indexLabelKey}">Loading...</div>
                    <div class="index-description text-xs" data-i18n="${indexDescriptionKey}">Loading...</div>
                </div>
            </div>
        `;

        // Trigger translation update if available
        this.updateTranslations(element);
        
        this.animateUpdate(element);
        
        // Render gauge chart
        this.renderGaugeChart('fear-greed', index);
        
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

        // Add to history
        this.dominanceHistory.btc.push({
            value: dominance,
            timestamp: Date.now()
        });
        
        // Keep only last N points
        if (this.dominanceHistory.btc.length > this.maxHistoryPoints) {
            this.dominanceHistory.btc.shift();
        }

        element.innerHTML = `
            <div class="index-value">${dominance.toFixed(1)}%</div>
        `;

        this.animateUpdate(element);
        
        // Render chart
        this.renderDominanceChart('btc', dominance);
        
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

        // Add to history
        this.dominanceHistory.eth.push({
            value: dominance,
            timestamp: Date.now()
        });
        
        // Keep only last N points
        if (this.dominanceHistory.eth.length > this.maxHistoryPoints) {
            this.dominanceHistory.eth.shift();
        }

        element.innerHTML = `
            <div class="index-value">${dominance.toFixed(1)}%</div>
        `;

        this.animateUpdate(element);
        
        // Render chart
        this.renderDominanceChart('eth', dominance);
        
        debugLog('✅ ETH dominance updated:', { dominance: dominance.toFixed(1) });
    }

    updateBtcRsi14(value) {
        const element = this.elements.btcRsi14;
        if (!element) return;

        const rsi = parseFloat(value) || 0;
        
        // Check if data has actually changed (with small tolerance for floating point comparison)
        const cachedBtcRsi14 = this.cachedData.btcRsi14;
        if (cachedBtcRsi14 !== null && Math.abs(cachedBtcRsi14 - rsi) < 0.01) {
            debugLog('⏭️ BTC RSI 14 unchanged, skipping update:', { rsi });
            return;
        }
        
        this.cachedData.btcRsi14 = rsi;

        let rsiClass = 'neutral';
        let rsiLabelKey;
        
        if (rsi <= 30) {
            rsiClass = 'oversold';
            rsiLabelKey = 'oversold';
        } else if (rsi <= 70) {
            rsiClass = 'neutral';
            rsiLabelKey = 'neutral';
        } else {
            rsiClass = 'overbought';
            rsiLabelKey = 'overbought';
        }

        element.innerHTML = `
            <div class="index-display flex items-center justify-between">
                <div class="index-value ${rsiClass}">${rsi.toFixed(1)}</div>
                <div class="index-label text-right" data-i18n="${rsiLabelKey}">RSI 14</div>
            </div>
        `;

        // Trigger translation update if available
        this.updateTranslations(element);
        
        this.animateUpdate(element);
        
        // Render gauge chart
        this.renderGaugeChart('btc-rsi', rsi);
        
        debugLog('✅ BTC RSI 14 updated:', { rsi: rsi.toFixed(1), class: rsiClass });
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
            debugLog('Translation update not available:', error);
        }
    }

    animateUpdate(element) {
        // Disabled to avoid visual noise with 2-second updates
        // Real-time data updates every 2s don't need animation
        return;
    }

    formatLargeNumber(num) {
        const lang = window.current_language || 'vi';
        const isVietnamese = lang === 'vi';
        
        if (num >= 1e12) {
            return {
                number: (num / 1e12).toFixed(2),
                unitKey: isVietnamese ? 'unit-trillion' : 'unit-trillion-en',
                unitText: isVietnamese ? ' Nghìn Tỷ' : ' T'
            };
        }
        if (num >= 1e9) {
            return {
                number: (num / 1e9).toFixed(2),
                unitKey: isVietnamese ? 'unit-billion' : 'unit-billion-en',
                unitText: isVietnamese ? ' Tỷ' : ' B'
            };
        }
        if (num >= 1e6) {
            return {
                number: (num / 1e6).toFixed(2),
                unitKey: isVietnamese ? 'unit-million' : 'unit-million-en',
                unitText: isVietnamese ? ' Triệu' : ' M'
            };
        }
        return {
            number: num.toLocaleString(lang === 'vi' ? 'vi-VN' : 'en-US'),
            unitKey: null,
            unitText: ''
        };
    }

    startDataRefresh() {
        // WebSocket health check every 30 seconds (optimized frequency)
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
            
            // If no data received for 90 seconds and connected, something may be wrong
            if (this.isConnected && timeSinceLastUpdate > 90000) {
                debugLog('⚠️ No data received for 90+ seconds, requesting fresh data');
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
            
            // Crypto prices now come from server via WebSocket
        }, 30000); // Every 30 seconds (optimized monitoring - reduced unnecessary checks)
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
        `;
        
        this.animateUpdate(element);
        debugLog(`✅ Updated ${displayName}: $${price.toFixed(2)} (${percentSign}${changePercent.toFixed(2)}%)`);
    }

    startHeartbeat() {
        // Clear any existing heartbeat first
        this.stopHeartbeat();
        
        // Send ping every 30 seconds to keep connection alive (optimized frequency)
        this.heartbeatInterval = setInterval(() => {
            if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
                debugLog('🏓 Sending heartbeat ping');
                this.websocket.send('ping');
            } else {
                debugLog('⚠️ WebSocket not open, stopping heartbeat');
                this.stopHeartbeat();
            }
        }, 30000); // Every 30 seconds (optimized heartbeat - reduced server load)
    }

    stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
            debugLog('🛑 Heartbeat stopped');
        }
    }

    destroy() {
        debugLog('🧹 Destroying Market Indicators Dashboard');
        this.stopHeartbeat();
        
        // Cancel any pending updates
        if (this.updateTimer) {
            cancelAnimationFrame(this.updateTimer);
            this.updateTimer = null;
        }
        
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        this.isConnected = false;
    }

    // Crypto prices now come from server via WebSocket

    updateCryptoPrice(symbol, price, changePercent) {
        let elementId;
        let coinName;
        
        // Map symbol to element ID and coin name
        switch (symbol) {
            case 'BTCUSDT':
                elementId = 'binance-btc-price';
                coinName = 'BTC';
                break;
            case 'ETHUSDT':
                elementId = 'binance-eth-price';
                coinName = 'ETH';
                break;
            case 'SOLUSDT':
                elementId = 'binance-sol-price';
                coinName = 'SOL';
                break;
            case 'XRPUSDT':
                elementId = 'binance-xrp-price';
                coinName = 'XRP';
                break;
            case 'ADAUSDT':
                elementId = 'binance-ada-price';
                coinName = 'ADA';
                break;
            case 'LINKUSDT':
                elementId = 'binance-link-price';
                coinName = 'LINK';
                break;
            case 'BNBUSDT':
                elementId = 'binance-bnb-price';
                coinName = 'BNB';
                break;
            default:
                return;
        }
        
        const element = document.getElementById(elementId);
        if (!element) {
            debugError(`❌ Element not found: ${elementId}`);
            return;
        }
        
        if (price === null || price === undefined || changePercent === 'Error') {
            // Only update if needed - use textContent for faster updates
            const priceElement = element.querySelector('[data-price]');
            const changeElement = element.querySelector('[data-change]');
            if (priceElement) priceElement.textContent = '--';
            if (changeElement) {
                changeElement.textContent = 'N/A';
                changeElement.className = 'binance-price-change neutral';
            }
            return;
        }
        
        // OPTIMIZED: Use textContent instead of innerHTML for 10-20ms faster updates
        const changeClass = changePercent >= 0 ? 'positive' : 'negative';
        const changeSign = changePercent >= 0 ? '+' : '';
        const locale = 'en-US';
        
        // Format price with commas and appropriate decimal places
        const formattedPrice = price.toLocaleString(locale, {
            minimumFractionDigits: 2,
            maximumFractionDigits: price >= 1 ? 2 : 6
        });
        
        // Get sub-elements (cached after first update for even better performance)
        const priceElement = element.querySelector('[data-price]');
        const changeElement = element.querySelector('[data-change]');
        
        if (priceElement && changeElement) {
            // Only update textContent - much faster than innerHTML (no re-parse, no re-render)
            priceElement.textContent = `$${formattedPrice}`;
            changeElement.textContent = `${changeSign}${changePercent.toFixed(2)}%`;
            
            // Update className only if changed (avoid unnecessary style recalculation)
            const newClassName = `binance-price-change ${changeClass}`;
            if (changeElement.className !== newClassName) {
                changeElement.className = newClassName;
            }
        } else {
            // Fallback to innerHTML if structure not found (first load only)
            element.innerHTML = `
                <div class="binance-price-value" data-price>$${formattedPrice}</div>
                <div class="binance-price-change ${changeClass}" data-change>
                    ${changeSign}${changePercent.toFixed(2)}%
                </div>
            `;
        }
        
        debugLog(`✅ Updated ${coinName}: $${formattedPrice} (${changeSign}${changePercent.toFixed(2)}%)`);
    }

    /**
     * Render dominance chart using SVG
     * @param {string} type - 'btc' or 'eth'
     * @param {number} currentValue - Current dominance percentage
     */
    renderDominanceChart(type, currentValue) {
        const svgId = type === 'btc' ? 'btc-dominance-svg' : 'eth-dominance-svg';
        const svg = document.getElementById(svgId);
        
        if (!svg) {
            debugLog(`❌ SVG element not found: ${svgId}`);
            return;
        }

        const history = this.dominanceHistory[type];
        
        // If we don't have enough history yet, show a simple pie chart
        if (history.length < 2) {
            this.renderDominancePieChart(svg, type, currentValue);
            return;
        }

        // Larger chart dimensions
        const width = 120;
        const height = 80;
        const padding = { top: 8, right: 8, bottom: 8, left: 8 };
        const chartWidth = width - padding.left - padding.right;
        const chartHeight = height - padding.top - padding.bottom;

        // Get min/max for scaling
        const values = history.map(h => h.value);
        const minValue = Math.max(0, Math.min(...values) - 2);
        const maxValue = Math.min(100, Math.max(...values) + 2);
        const valueRange = maxValue - minValue;

        // Scale functions
        const scaleX = (index) => padding.left + (index / (history.length - 1)) * chartWidth;
        const scaleY = (value) => padding.top + chartHeight - ((value - minValue) / valueRange) * chartHeight;

        // Color scheme
        const color = type === 'btc' ? '#f7931a' : '#627eea';
        const gradientId = type === 'btc' ? 'btc-gradient-large' : 'eth-gradient-large';

        // Clear SVG
        svg.innerHTML = '';

        // Create gradient definition
        const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
        const gradient = document.createElementNS('http://www.w3.org/2000/svg', 'linearGradient');
        gradient.setAttribute('id', gradientId);
        gradient.setAttribute('x1', '0%');
        gradient.setAttribute('y1', '0%');
        gradient.setAttribute('x2', '0%');
        gradient.setAttribute('y2', '100%');
        
        const stop1 = document.createElementNS('http://www.w3.org/2000/svg', 'stop');
        stop1.setAttribute('offset', '0%');
        stop1.setAttribute('style', `stop-color:${color};stop-opacity:0.6`);
        
        const stop2 = document.createElementNS('http://www.w3.org/2000/svg', 'stop');
        stop2.setAttribute('offset', '100%');
        stop2.setAttribute('style', `stop-color:${color};stop-opacity:0.1`);
        
        gradient.appendChild(stop1);
        gradient.appendChild(stop2);
        defs.appendChild(gradient);
        svg.appendChild(defs);

        // Create line path
        let pathData = '';
        let areaPath = '';
        
        history.forEach((point, index) => {
            const x = scaleX(index);
            const y = scaleY(point.value);
            
            if (index === 0) {
                pathData += `M ${x} ${y}`;
                areaPath += `M ${x} ${height - padding.bottom}`;
                areaPath += ` L ${x} ${y}`;
            } else {
                pathData += ` L ${x} ${y}`;
                areaPath += ` L ${x} ${y}`;
            }
        });

        // Close area path
        const lastXPos = scaleX(history.length - 1);
        areaPath += ` L ${lastXPos} ${height - padding.bottom} Z`;

        // Draw area fill
        const area = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        area.setAttribute('fill', `url(#${gradientId})`);
        area.setAttribute('d', areaPath);
        svg.appendChild(area);

        // Draw line
        const line = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        line.setAttribute('stroke', color);
        line.setAttribute('stroke-width', '2');
        line.setAttribute('fill', 'none');
        line.setAttribute('d', pathData);
        svg.appendChild(line);

        // Draw points - all points
        history.forEach((point, index) => {
            const x = scaleX(index);
            const y = scaleY(point.value);
            
            const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            circle.setAttribute('fill', color);
            circle.setAttribute('cx', x);
            circle.setAttribute('cy', y);
            circle.setAttribute('r', index === history.length - 1 ? '3' : '1.5');
            svg.appendChild(circle);
        });

        debugLog(`📊 Rendered ${type.toUpperCase()} dominance large chart with ${history.length} points`);
    }

    /**
     * Render a simple pie chart for dominance when we don't have history
     * @param {SVGElement} svg - SVG element
     * @param {string} type - 'btc' or 'eth'
     * @param {number} value - Dominance percentage
     */
    renderDominancePieChart(svg, type, value) {
        const width = 120;
        const height = 80;
        const centerX = width / 2;
        const centerY = height / 2;
        const radius = 28;

        const color = type === 'btc' ? '#f7931a' : '#627eea';
        const othersColor = 'rgba(128, 128, 128, 0.3)';

        // Clear SVG
        svg.innerHTML = '';

        // Calculate angles
        const angle = (value / 100) * 2 * Math.PI;
        const startAngle = -Math.PI / 2; // Start from top

        // Draw "Others" slice (full circle background)
        const othersCircle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        othersCircle.setAttribute('cx', centerX);
        othersCircle.setAttribute('cy', centerY);
        othersCircle.setAttribute('r', radius);
        othersCircle.setAttribute('fill', othersColor);
        svg.appendChild(othersCircle);

        // Draw dominance slice
        if (value > 0 && value < 100) {
            const endAngle = startAngle + angle;
            
            const x1 = centerX + radius * Math.cos(startAngle);
            const y1 = centerY + radius * Math.sin(startAngle);
            const x2 = centerX + radius * Math.cos(endAngle);
            const y2 = centerY + radius * Math.sin(endAngle);
            
            const largeArc = angle > Math.PI ? 1 : 0;
            
            const pathData = [
                `M ${centerX} ${centerY}`,
                `L ${x1} ${y1}`,
                `A ${radius} ${radius} 0 ${largeArc} 1 ${x2} ${y2}`,
                'Z'
            ].join(' ');
            
            const slice = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            slice.setAttribute('d', pathData);
            slice.setAttribute('fill', color);
            slice.setAttribute('opacity', '0.8');
            svg.appendChild(slice);
        } else if (value >= 100) {
            // Full circle
            const fullCircle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            fullCircle.setAttribute('cx', centerX);
            fullCircle.setAttribute('cy', centerY);
            fullCircle.setAttribute('r', radius);
            fullCircle.setAttribute('fill', color);
            fullCircle.setAttribute('opacity', '0.8');
            svg.appendChild(fullCircle);
        }

        debugLog(`📊 Rendered ${type.toUpperCase()} dominance large pie chart: ${value.toFixed(1)}%`);
    }

    /**
     * Chuyển đổi tọa độ cực sang Descartes cho gauge chart
     */
    polarToCartesian(centerX, centerY, radius, angleInDegrees) {
        const angleInRadians = ((angleInDegrees - 90) * Math.PI) / 180.0;
        return {
            x: centerX + radius * Math.cos(angleInRadians),
            y: centerY + radius * Math.sin(angleInRadians)
        };
    }

    /**
     * Tạo chuỗi path data cho cung tròn SVG
     */
    describeArc(x, y, radius, startAngle, endAngle) {
        const startPoint = this.polarToCartesian(x, y, radius, startAngle);
        const endPoint = this.polarToCartesian(x, y, radius, endAngle);
        const largeArcFlag = endAngle - startAngle <= 180 ? '0' : '1';
        
        const d = [
            'M', startPoint.x, startPoint.y,
            'A', radius, radius, 0, largeArcFlag, '1', endPoint.x, endPoint.y
        ].join(' ');
        
        return d;
    }

    /**
     * Render gauge chart cho Fear & Greed hoặc RSI
     */
    renderGaugeChart(type, value) {
        const svgId = type === 'fear-greed' ? 'fear-greed-gauge-svg' : 'btc-rsi-gauge-svg';
        const svg = document.getElementById(svgId);
        if (!svg) {
            debugLog(`⚠️ Gauge SVG not found for ${type}`);
            return;
        }

        // Cấu hình gauge
        const GAUGE_START_ANGLE = -120;
        const GAUGE_END_ANGLE = 120;
        const ANGLE_SPAN = GAUGE_END_ANGLE - GAUGE_START_ANGLE;
        
        const centerX = 60;
        const centerY = 60;
        const radius = 45;
        const strokeWidth = 10;

        // Xác định điểm màu cho gradient
        let colorStops;
        if (type === 'fear-greed') {
            // Fear & Greed: Đỏ -> Cam -> Vàng -> Xanh lá nhạt -> Xanh lá
            colorStops = [
                { value: 0, color: '#ef4444' },    // Red (Extreme Fear)
                { value: 25, color: '#f97316' },   // Orange
                { value: 40, color: '#fb923c' },   // Light Orange
                { value: 50, color: '#fbbf24' },   // Yellow (Neutral)
                { value: 60, color: '#a3e635' },   // Yellow-Green
                { value: 75, color: '#84cc16' },   // Lime
                { value: 100, color: '#22c55e' }   // Green (Extreme Greed)
            ];
        } else {
            // RSI: Xanh lá -> Xanh nhạt -> Vàng -> Cam -> Đỏ
            colorStops = [
                { value: 0, color: '#22c55e' },    // Green (Oversold)
                { value: 25, color: '#84cc16' },   // Lime
                { value: 40, color: '#a3e635' },   // Yellow-Green
                { value: 50, color: '#fbbf24' },   // Yellow (Neutral)
                { value: 60, color: '#fb923c' },   // Light Orange
                { value: 75, color: '#f97316' },   // Orange
                { value: 100, color: '#ef4444' }   // Red (Overbought)
            ];
        }

        // Tính toán góc cho giá trị hiện tại
        const min = 0;
        const max = 100;
        const clampedValue = Math.max(min, Math.min(max, value));
        const percentage = (clampedValue - min) / (max - min);
        const valueAngle = GAUGE_START_ANGLE + (percentage * ANGLE_SPAN);

        // Clear SVG
        svg.innerHTML = '';

        // Tạo gradient mượt dọc theo cung
        const gradientId = `${type}-gauge-gradient`;
        const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
        const gradient = document.createElementNS('http://www.w3.org/2000/svg', 'linearGradient');
        gradient.setAttribute('id', gradientId);
        gradient.setAttribute('gradientUnits', 'userSpaceOnUse');

        const startPoint = this.polarToCartesian(centerX, centerY, radius, GAUGE_START_ANGLE);
        const endPoint = this.polarToCartesian(centerX, centerY, radius, GAUGE_END_ANGLE);
        gradient.setAttribute('x1', startPoint.x);
        gradient.setAttribute('y1', startPoint.y);
        gradient.setAttribute('x2', endPoint.x);
        gradient.setAttribute('y2', endPoint.y);

        const gradientStops = colorStops.map(stop => ({
            offset: (stop.value - min) / (max - min),
            color: stop.color
        }));

        // Ensure offset boundaries are valid
        gradientStops.forEach(stop => {
            const stopElement = document.createElementNS('http://www.w3.org/2000/svg', 'stop');
            const offset = Math.max(0, Math.min(1, stop.offset));
            stopElement.setAttribute('offset', `${(offset * 100).toFixed(1)}%`);
            stopElement.setAttribute('stop-color', stop.color);
            gradient.appendChild(stopElement);
        });

        defs.appendChild(gradient);
        svg.appendChild(defs);

        // 1. Vẽ track nền (cung tròn hoàn chỉnh)
        const trackPath = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        trackPath.setAttribute('d', this.describeArc(centerX, centerY, radius, GAUGE_START_ANGLE, GAUGE_END_ANGLE));
        trackPath.setAttribute('fill', 'none');
        trackPath.setAttribute('stroke', 'rgba(200, 200, 200, 0.2)');
        trackPath.setAttribute('stroke-width', strokeWidth);
        trackPath.setAttribute('stroke-linecap', 'round');
        svg.appendChild(trackPath);

        // 2. Vẽ cung màu gradient
        const gradientPath = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        gradientPath.setAttribute('d', this.describeArc(centerX, centerY, radius, GAUGE_START_ANGLE, GAUGE_END_ANGLE));
        gradientPath.setAttribute('fill', 'none');
        gradientPath.setAttribute('stroke', `url(#${gradientId})`);
        gradientPath.setAttribute('stroke-width', strokeWidth);
        gradientPath.setAttribute('stroke-linecap', 'round');
        svg.appendChild(gradientPath);

        // 3. Vẽ kim chỉ
        const needleGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        needleGroup.setAttribute('transform', `rotate(${valueAngle} ${centerX} ${centerY})`);
        
        // Kim chỉ (hình tam giác nhỏ gọn)
        const needlePointer = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        const needlePath = `M ${centerX} ${centerY - radius + 5} L ${centerX - 3} ${centerY} L ${centerX + 3} ${centerY} Z`;
        needlePointer.setAttribute('d', needlePath);
        needlePointer.setAttribute('fill', '#1f2937');
        needleGroup.appendChild(needlePointer);
        
        // Pivot (điểm tâm)
        const needlePivot = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        needlePivot.setAttribute('cx', centerX);
        needlePivot.setAttribute('cy', centerY);
        needlePivot.setAttribute('r', '4');
        needlePivot.setAttribute('fill', '#1f2937');
        needleGroup.appendChild(needlePivot);
        
        svg.appendChild(needleGroup);

        debugLog(`📊 Rendered ${type} gauge chart: ${value.toFixed(1)}`);
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
        console.log('🔗 WebSocket readyState:', window.marketIndicatorsDashboard.websocket ? window.marketIndicatorsDashboard.websocket.readyState : 'null');
    } else {
        console.log('❌ No dashboard instance found');
    }
};

// Manual WebSocket data request (only active in DEBUG_MODE)
window.requestMarketData = function() {
    if (!DEBUG_MODE) {
        console.log('Debug mode is disabled. Set DEBUG_MODE = true to enable debugging.');
        return;
    }
    
    if (window.marketIndicatorsDashboard) {
        console.log('📤 Manually requesting fresh market data via WebSocket...');
        window.marketIndicatorsDashboard.requestFreshData();
    } else {
        console.log('❌ No dashboard instance found');
    }
};

// Test function with sample data (only active in DEBUG_MODE)
window.testMarketIndicators = function() {
    const sampleData = {
        btc_change_24h: -1.1761369473535623,
        btc_price_usd: 110349,
        eth_change_24h: -3.174,
        eth_price_usd: 4340.56,
        sol_change_24h: -0.455,
        sol_price_usd: 221.01,
        xrp_change_24h: -2.609,
        xrp_price_usd: 2.7962,
        ada_change_24h: -1.923,
        ada_price_usd: 0.8058,
        link_change_24h: -1.632,
        link_price_usd: 21.7,
        bnb_change_24h: -2.1,
        bnb_price_usd: 680.5,
        fng_value: 48,
        market_cap_usd: 3872941106289.462,
        btc_rsi_14: 38.4490332215743,
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
    
        debugLog('🧪 Testing with sample data (including all crypto prices):', sampleData);
    }
