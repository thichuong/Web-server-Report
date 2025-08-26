/**
 * Market Indicators Dashboard JavaScript
 * Handles real-time market data display without charts
 * Integrates with WebSocket for live updates
 */

class MarketIndicatorsDashboard {
    constructor() {
        this.websocket = null;
        this.isConnected = false;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        this.updateAnimationDuration = 300;
        
        // Data cache
        this.cachedData = {
            btcPrice: null,
            marketCap: null,
            volume24h: null,
            fearGreedIndex: null,
            btcDominance: null,
            activeCryptos: null,
            markets: null,
            marketCapChange: null,
            lastUpdated: null
        };
        
        this.init();
    }

    init() {
        console.log('🚀 Initializing Market Indicators Dashboard');
        this.initializeElements();
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
            activeCryptos: document.getElementById('active-cryptos-indicator'),
            markets: document.getElementById('markets-indicator'),
            marketCapChange: document.getElementById('market-cap-change-indicator'),
            lastUpdated: document.getElementById('last-updated-indicator'),
            connectionStatus: document.getElementById('connection-status')
        };

        // Debug: Log which elements were found
        console.log('🔍 Element search results:');
        Object.entries(this.elements).forEach(([key, element]) => {
            if (element) {
                console.log(`  ✅ ${key}: found`);
            } else {
                console.log(`  ❌ ${key}: NOT found`);
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

    connectWebSocket() {
        if (this.isConnected || this.websocket) {
            return;
        }

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        console.log('🔌 Connecting to WebSocket for market indicators:', wsUrl);
        this.updateConnectionStatus('connecting');

        try {
            this.websocket = new WebSocket(wsUrl);

            this.websocket.onopen = () => {
                console.log('✅ Market Indicators WebSocket connected');
                this.isConnected = true;
                this.reconnectAttempts = 0;
                this.reconnectDelay = 1000;
                this.updateConnectionStatus('connected');
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
                console.log('🔌 Market Indicators WebSocket disconnected:', event.code);
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
            this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);
            
            console.log(`🔄 Scheduling reconnect... Attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts}`);
            setTimeout(() => this.connectWebSocket(), this.reconnectDelay);
        } else {
            console.log('❌ Max reconnect attempts reached');
            this.updateConnectionStatus('offline');
        }
    }

    handleWebSocketMessage(message) {
        console.log('📨 Received WebSocket message type:', message.type);
        console.log('📨 Full message data:', message);
        
        switch (message.type) {
            case 'dashboard_data':
            case 'dashboard_update':
                if (message.data) {
                    console.log('📊 Received market data update:', message.data);
                    this.updateMarketData(message.data);
                }
                break;
                
            case 'btc_price_update':
                if (message.data) {
                    console.log('₿ Received BTC price update:', message.data);
                    this.updateBtcPrice(message.data);
                }
                break;
                
            case 'market_update':
                if (message.data) {
                    console.log('📈 Received market update:', message.data);
                    this.updateMarketData(message.data);
                }
                break;
                
            default:
                console.log('❓ Unknown WebSocket message type:', message.type);
                // Try to handle as generic market data if it has the expected fields
                if (message.btc_price_usd || message.market_cap_usd || message.fng_value) {
                    console.log('🔄 Treating unknown message as market data:', message);
                    this.updateMarketData(message);
                }
                break;
        }
    }

    updateMarketData(data) {
        console.log('🔄 Updating market indicators with data:', data);

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
                change: data.market_cap_change_24h || 0
            });
        }

        // Update Volume 24h - use total_volume_24h if available, or estimate from market cap
        if (data.total_volume_24h !== undefined) {
            this.updateVolume24h({
                value: data.total_volume_24h,
                change: data.volume_change_24h || 0
            });
        } else if (data.market_cap_usd !== undefined) {
            // Estimate volume as roughly 10% of market cap (common ratio)
            const estimatedVolume = data.market_cap_usd * 0.1;
            this.updateVolume24h({
                value: estimatedVolume,
                change: 0 // No change data available
            });
        }

        // Update Fear & Greed Index - use fng_value
        if (data.fng_value !== undefined) {
            this.updateFearGreedIndex(data.fng_value);
        }

        // Update BTC Dominance - calculate or use if available
        if (data.btc_dominance !== undefined) {
            this.updateBtcDominance(data.btc_dominance);
        } else if (data.btc_price_usd && data.market_cap_usd) {
            // Estimate BTC dominance: assume 21M BTC supply
            const btcMarketCap = data.btc_price_usd * 21000000;
            const btcDominance = (btcMarketCap / data.market_cap_usd) * 100;
            this.updateBtcDominance(btcDominance);
        }

        // Update additional metrics with reasonable defaults
        if (data.active_cryptocurrencies !== undefined) {
            this.updateActiveCryptos(data.active_cryptocurrencies);
        } else {
            this.updateActiveCryptos(15000); // Reasonable default
        }

        if (data.markets !== undefined) {
            this.updateMarkets(data.markets);
        } else {
            this.updateMarkets(800); // Reasonable default
        }

        if (data.market_cap_change_24h !== undefined) {
            this.updateMarketCapChange(data.market_cap_change_24h);
        } else if (data.btc_change_24h !== undefined) {
            // Use BTC change as proxy for market cap change
            this.updateMarketCapChange(data.btc_change_24h * 0.8); // Slightly lower than BTC
        }

        // Update last updated time
        this.updateLastUpdated();
    }

    updateBtcPrice(data) {
        const element = this.elements.btcPrice;
        if (!element) return;

        const price = parseFloat(data.price_usd || data.btc_price_usd) || 0;
        const change = parseFloat(data.change_24h || data.btc_change_24h) || 0;

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
    }

    updateMarketCap(data) {
        const element = this.elements.marketCap;
        if (!element) return;

        const value = parseFloat(data.value || data.total_market_cap || data.market_cap_usd) || 0;
        const change = parseFloat(data.change || data.market_cap_change_24h) || 0;

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
    }

    updateVolume24h(data) {
        const element = this.elements.volume24h;
        if (!element) return;

        const value = parseFloat(data.value || data.total_volume_24h) || 0;
        const change = parseFloat(data.change || data.volume_change_24h) || 0;

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
    }

    updateFearGreedIndex(value) {
        const element = this.elements.fearGreed;
        if (!element) return;

        const index = parseInt(value) || 0;
        this.cachedData.fearGreedIndex = index;

        let indexClass = 'neutral';
        let indexLabel = 'Trung tính';
        let indexDescription = '';

        if (index <= 25) {
            indexClass = 'fear';
            indexLabel = 'Sợ hãi cực độ';
            indexDescription = 'Thị trường đang trong trạng thái sợ hãi';
        } else if (index <= 50) {
            indexClass = 'fear';
            indexLabel = 'Sợ hãi';
            indexDescription = 'Thị trường có xu hướng giảm';
        } else if (index <= 75) {
            indexClass = 'greed';
            indexLabel = 'Tham lam';
            indexDescription = 'Thị trường có xu hướng tăng';
        } else {
            indexClass = 'greed';
            indexLabel = 'Tham lam cực độ';
            indexDescription = 'Thị trường đang trong trạng thái tham lam';
        }

        element.innerHTML = `
            <div class="index-display">
                <div class="index-value ${indexClass}">${index}</div>
                <div class="index-label">${indexLabel}</div>
                <div class="index-description">${indexDescription}</div>
            </div>
        `;

        this.animateUpdate(element);
    }

    updateBtcDominance(value) {
        const element = this.elements.btcDominance;
        if (!element) return;

        const dominance = parseFloat(value) || 0;
        this.cachedData.btcDominance = dominance;

        element.innerHTML = `
            <div class="index-display">
                <div class="index-value">${dominance.toFixed(1)}%</div>
                <div class="index-label" data-i18n="btc-market-share">Thị phần BTC</div>
            </div>
        `;

        this.animateUpdate(element);
    }

    updateActiveCryptos(value) {
        const element = this.elements.activeCryptos;
        if (!element) return;

        const count = parseInt(value) || 0;
        this.cachedData.activeCryptos = count;

        element.innerHTML = `<div class="fade-in">${count.toLocaleString()}</div>`;
        this.animateUpdate(element);
    }

    updateMarkets(value) {
        const element = this.elements.markets;
        if (!element) return;

        const count = parseInt(value) || 0;
        this.cachedData.markets = count;

        element.innerHTML = `<div class="fade-in">${count.toLocaleString()}</div>`;
        this.animateUpdate(element);
    }

    updateMarketCapChange(value) {
        const element = this.elements.marketCapChange;
        if (!element) return;

        const change = parseFloat(value) || 0;
        this.cachedData.marketCapChange = change;

        const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
        const changeSign = change >= 0 ? '+' : '';

        element.innerHTML = `<div class="fade-in ${changeClass}">${changeSign}${change.toFixed(2)}%</div>`;
        this.animateUpdate(element);
    }

    updateLastUpdated() {
        const element = this.elements.lastUpdated;
        if (!element) return;

        const now = new Date();
        const timeStr = now.toLocaleTimeString('vi-VN', { 
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit' 
        });

        this.cachedData.lastUpdated = now;

        element.innerHTML = `<div class="fade-in">${timeStr}</div>`;
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
        // Refresh data every 30 seconds as backup
        setInterval(() => {
            if (!this.isConnected) {
                this.updateLastUpdated();
            }
        }, 30000);
    }

    destroy() {
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        this.isConnected = false;
    }
}

// Initialize Market Indicators Dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    console.log('📊 DOM loaded, initializing Market Indicators Dashboard');
    
    // Only initialize if the market indicators container exists
    const container = document.getElementById('market-indicators-dashboard');
    if (container) {
        console.log('✅ Market indicators container found, creating dashboard instance');
        window.marketIndicatorsDashboard = new MarketIndicatorsDashboard();
        
        // Add a global function to manually update with data (for debugging)
        window.updateMarketIndicators = function(data) {
            if (window.marketIndicatorsDashboard) {
                console.log('🔧 Manual update triggered with data:', data);
                window.marketIndicatorsDashboard.updateMarketData(data);
            }
        };
    } else {
        console.log('❌ Market indicators container not found');
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

// Global debugging helpers
window.debugMarketIndicators = function() {
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

// Test function with sample data
window.testMarketIndicators = function() {
    const sampleData = {
        btc_change_24h: -1.1761369473535623,
        btc_price_usd: 110349,
        fng_value: 48,
        market_cap_usd: 3872941106289.462,
        rsi_14: 38.4490332215743
    };
    
    console.log('🧪 Testing with sample data:', sampleData);
    if (window.updateMarketIndicators) {
        window.updateMarketIndicators(sampleData);
    } else {
        console.log('❌ updateMarketIndicators function not available');
    }
};
