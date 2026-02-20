// dashboard-websocket.js - Combined dashboard logic and WebSocket functionality

// Reduce console noise for Firefox performance
const WS_DEBUG = true; // Enable debug logging to troubleshoot connection issues

// ===== WEBSOCKET MANAGER =====

/**
 * WebSocket connection manager for real-time dashboard updates
 */
class DashboardWebSocket {
    constructor() {
        this.socket = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000; // Start with 1 second
        this.isConnecting = false;
        this.heartbeatInterval = null;
        this.pendingDOMUpdates = []; // Batch DOM updates
        this.updateScheduled = false;
    }

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

            wsUrl = wsUrl + '/ws';
            if (WS_DEBUG) console.log('🔗 Using injected WebSocket URL:', wsUrl);
        } else {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            wsUrl = `${protocol}//${window.location.host}/ws`;
            if (WS_DEBUG) console.log('⚠️ No injected WebSocket URL, using same-host fallback');
        }

        if (WS_DEBUG) {
            console.log('🔍 [DEBUG] WebSocket connection details:');
            console.log('  🔗 Full URL:', wsUrl);
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
                    if (typeof event.data === 'string' && event.data.startsWith('Connected')) {
                        if (WS_DEBUG) console.log('✅ Server connection check:', event.data);
                        return;
                    }
                    const message = JSON.parse(event.data);
                    this.handleMessage(message);
                } catch (error) {
                    console.error('❌ Error parsing WebSocket message:', error);
                }
            };

            this.socket.onclose = (event) => {
                if (WS_DEBUG) console.log('🔌 WebSocket disconnected:', event.code, event.reason);
                this.isConnecting = false;
                this.stopHeartbeat();
                
                if (event.code !== 1000) { // Not a normal closure
                    updateWebSocketStatus('disconnected', getTranslatedText('connection-lost') || 'Mất kết nối');
                }
                
                // Attempt to reconnect if not manually closed
                if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
                    setTimeout(() => this.reconnect(), this.reconnectDelay);
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
        }
    }

    reconnect() {
        this.reconnectAttempts++;
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000); // Max 30 seconds
        
        console.log(`🔄 Reconnecting... Attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts}`);
        this.connect();
    }

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
            if (WS_DEBUG) console.log('⚠️ Message has no data field:', message);
        }
    }

    // Batch DOM updates using requestAnimationFrame for Firefox performance
    scheduleDOMUpdate(updateFn) {
        this.pendingDOMUpdates.push(updateFn);
        
        if (!this.updateScheduled) {
            this.updateScheduled = true;
            requestAnimationFrame(() => {
                // Execute all pending updates in one batch
                this.pendingDOMUpdates.forEach(fn => {
                    try {
                        fn();
                    } catch (error) {
                        console.error('❌ Error in batched DOM update:', error);
                    }
                });
                this.pendingDOMUpdates = [];
                this.updateScheduled = false;
            });
        }
    }

    startHeartbeat() {
        // Clear any existing heartbeat first
        this.stopHeartbeat();
        
        this.heartbeatInterval = setInterval(() => {
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                this.socket.send('ping');
            } else {
                if (WS_DEBUG) console.log('⚠️ Socket not open, stopping heartbeat');
                this.stopHeartbeat();
            }
        }, 30000); // Ping every 30 seconds (optimal for Cloudflare - keeps connection alive within 100s timeout)
    }

    stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }
    }

    disconnect() {
        this.stopHeartbeat();
        
        // Clear pending updates
        this.pendingDOMUpdates = [];
        this.updateScheduled = false;
        
        if (this.socket) {
            this.socket.close(1000, 'Manual disconnect');
            this.socket = null;
        }
    }
}

// ===== WEBSOCKET DATA HANDLERS =====

// Helper function để update BTC price từ WebSocket
function updateBtcPriceFromWebSocket(btcData) {
    if (WS_DEBUG) console.log('🔄 Updating BTC price from WebSocket:', btcData);
    
    const btcContainer = selectDashboardElementByLang('btc-price-container');
    
    // Handle both formats: direct BTC data or full dashboard data
    const priceValue = btcData.btc_price_usd || btcData.price_usd || 0;
    const changeValue = btcData.btc_change_24h || btcData.change_24h || 0;
    
    if (btcContainer && priceValue) {
        showBtcRefreshIndicator();
        
        const price = parseFloat(priceValue) || 0;
        const change = parseFloat(changeValue) || 0;
        const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
        const changeIcon = change >= 0 ? '📈' : '📉';
        
        // OPTIMIZED: Use textContent instead of innerHTML for faster updates
        const priceElement = btcContainer.querySelector('[data-btc-price]');
        const changeElement = btcContainer.querySelector('[data-btc-change]');
        
        if (priceElement && changeElement) {
            // Only update text - 90% faster than innerHTML
            priceElement.textContent = `$${price.toLocaleString('en-US')}`;
            changeElement.textContent = `${changeIcon} ${change.toFixed(2)}% (24h)`;
            
            // Update className only if changed
            const newClassName = `text-sm font-semibold ${changeClass}`;
            if (changeElement.className !== newClassName) {
                changeElement.className = newClassName;
            }
        } else {
            // Fallback to innerHTML if structure not found (first load only)
            btcContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900" data-btc-price>$${price.toLocaleString('en-US')}</p>
                <p class="text-sm font-semibold ${changeClass}" data-btc-change>${changeIcon} ${change.toFixed(2)}% (24h)</p>`;
        }
            
        try { 
            btcContainer.dataset.btcPriceUsd = String(price); 
            btcContainer.dataset.btcChange24h = String(change); 
        } catch(e){}
        
        if (WS_DEBUG) console.log('✅ BTC price container updated');
    }
}

// Helper function để update market data từ WebSocket
function updateMarketDataFromWebSocket(marketData) {
    if (WS_DEBUG) console.log('🔄 Updating market data from WebSocket:', marketData);
    
    // Update Market Cap
    if (marketData.market_cap_usd) {
        const marketCapContainer = selectDashboardElementByLang('market-cap-container');
        if (marketCapContainer) {
            const marketCap = parseFloat(marketData.market_cap_usd) || 0;
            marketCapContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">$${(marketCap / 1e12).toFixed(2)}T</p>
                <p class="text-sm text-gray-600">Market Cap</p>`;
        }
    }
    
    // Update Volume
    if (marketData.volume_24h_usd) {
        const volumeContainer = selectDashboardElementByLang('volume-container');
        if (volumeContainer) {
            const volume = parseFloat(marketData.volume_24h_usd) || 0;
            volumeContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">$${(volume / 1e9).toFixed(1)}B</p>
                <p class="text-sm text-gray-600">24h Volume</p>`;
        }
    }
    
    // Update Fear & Greed if available
    if (marketData.fng_value) {
        const fngContainer = selectDashboardElementByLang('fear-greed-container');
        const fngValue = parseInt(marketData.fng_value, 10);
        if (!isNaN(fngValue) && fngContainer) {
            updateFearGreedIndex(fngContainer, fngValue);
        }
    }
    
    if (WS_DEBUG) console.log('✅ Market data updated from WebSocket');
}

// ===== DASHBOARD UTILITIES =====

/**
 * Định dạng số lớn thành dạng ngắn gọn (nghìn tỷ, tỷ, triệu).
 * @param {number} num - Số cần định dạng.
 * @returns {string} - Chuỗi đã được định dạng.
 */
function formatNumber(num) {
    // Sử dụng formatNumberLocalized nếu có sẵn, nếu không dùng format cũ
    if (window.languageManager && window.languageManager.formatNumberLocalized) {
        return window.languageManager.formatNumberLocalized(num);
    }
    
    // Fallback to old format
    if (num === null || num === undefined) return 'N/A';
    if (num >= 1e12) return (num / 1e12).toFixed(2) + ' nghìn tỷ';
    if (num >= 1e9) return (num / 1e9).toFixed(2) + ' tỷ';
    if (num >= 1e6) return (num / 1e6).toFixed(2) + ' triệu';
    return num.toLocaleString('en-US');
}

/**
 * Lấy text đã dịch
 */
function getTranslatedText(key) {
    if (window.languageManager && window.languageManager.getTranslatedText) {
        return window.languageManager.getTranslatedText(key);
    }
    return key; // fallback
}

/**
 * Select an element for dashboard by language-aware id.
 * If `lang` is 'en' it will try id + '-en' first, then fallback to base id.
 * If no language specified, prefer window.languageManager.currentLanguage when available.
 */
function selectDashboardElementByLang(idBase, lang) {
    const language = lang || (window.languageManager && window.languageManager.currentLanguage) || 'vi';
    if (language === 'en') {
        const enEl = document.getElementById(idBase + '-en');
        if (enEl) return enEl;
    }
    return document.getElementById(idBase);
}

/**
 * Hiển thị thông báo lỗi thân thiện trên một card cụ thể.
 * @param {string} containerId - ID của container cần hiển thị lỗi.
 * @param {string} message - Thông báo lỗi.
 */
function displayError(containerId, message) {
    const container = document.getElementById(containerId);
    if (container) {
        const errorMsg = message || getTranslatedText('error-loading-data');
        container.innerHTML = `<p class="text-sm text-red-600">${errorMsg}</p>`;
    }
}

// ===== DASHBOARD DATA MANAGEMENT =====

/**
 * Update dashboard UI from data object (used by both HTTP API and WebSocket)
 */
function updateDashboardFromData(data) {
    // 🔍 DEBUG: Log the received data structure (only in debug mode)
    if (WS_DEBUG) {
        console.log('🔍 [DEBUG] updateDashboardFromData received:', data);
        console.log('🔍 [DEBUG] Data types:', {
            market_cap_usd: typeof data.market_cap_usd,
            volume_24h_usd: typeof data.volume_24h_usd,
            btc_price_usd: typeof data.btc_price_usd,
            btc_change_24h: typeof data.btc_change_24h,
            fng_value: typeof data.fng_value,
            btc_rsi_14: typeof data.btc_rsi_14
        });
    }
    
    // Cập nhật Vốn hóa thị trường
    const marketCapContainer = selectDashboardElementByLang('market-cap-container');
    if (marketCapContainer) {
        const marketCapValue = parseFloat(data.market_cap_usd || data.market_cap) || 0;
        const marketCapChange = parseFloat(data.market_cap_change_percentage_24h_usd) || 0;
        if (WS_DEBUG) console.log('🔍 [DEBUG] Market Cap parsed:', marketCapValue, 'Change:', marketCapChange);
        
        const changeClass = marketCapChange >= 0 ? 'text-green-600' : 'text-red-600';
        const changeSign = marketCapChange >= 0 ? '+' : '';
        const changeIcon = marketCapChange >= 0 ? '📈' : '📉';
        
        marketCapContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(marketCapValue)}</p>
            <p class="text-sm ${changeClass}">${changeIcon} ${changeSign}${marketCapChange.toFixed(2)}% (24h)</p>
            <p class="text-xs text-gray-600">Market Cap</p>`;
        // cache numeric value so we can re-render visuals without re-fetch
        try { marketCapContainer.dataset.marketCap = String(marketCapValue); } catch(e){}
    }

    // Cập nhật Khối lượng giao dịch
    const volumeContainer = selectDashboardElementByLang('volume-24h-container');
    if (volumeContainer) {
        const volumeValue = parseFloat(data.volume_24h_usd || data.volume_24h) || 0;
        if (WS_DEBUG) console.log('🔍 [DEBUG] Volume parsed:', volumeValue);
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(volumeValue)}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
        try { volumeContainer.dataset.volume24h = String(volumeValue); } catch(e){}
    }

    // Cập nhật giá BTC với visual feedback
    const btcContainer = selectDashboardElementByLang('btc-price-container');
    if (btcContainer) {
        // Show refresh indicator briefly
        showBtcRefreshIndicator();
        
        const btcPrice = parseFloat(data.btc_price_usd) || 0;
        const change = parseFloat(data.btc_change_24h) || 0;
        if (WS_DEBUG) console.log('🔍 [DEBUG] BTC Price parsed:', btcPrice, 'Change:', change);
        
        // Safely handle change value - could be undefined, null, or 0
        const safeChange = (change !== undefined && change !== null) ? change : 0;
        const changeClass = safeChange >= 0 ? 'text-green-600' : 'text-red-600';
        const changeIcon = safeChange >= 0 ? '📈' : '📉';
        
        // OPTIMIZED: Use textContent instead of innerHTML for faster updates
        const priceElement = btcContainer.querySelector('[data-btc-price]');
        const changeElement = btcContainer.querySelector('[data-btc-change]');
        
        if (priceElement && changeElement) {
            // Only update text - 90% faster than innerHTML
            priceElement.textContent = btcPrice > 0 ? '$' + btcPrice.toLocaleString('en-US') : '$N/A';
            changeElement.textContent = `${changeIcon} ${safeChange.toFixed(2)}% (24h)`;
            
            // Update className only if changed
            const newClassName = `text-sm font-semibold ${changeClass}`;
            if (changeElement.className !== newClassName) {
                changeElement.className = newClassName;
            }
        } else {
            // Fallback to innerHTML if structure not found
            btcContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900" data-btc-price>${btcPrice > 0 ? '$' + btcPrice.toLocaleString('en-US') : '$N/A'}</p>
                <p class="text-sm font-semibold ${changeClass}" data-btc-change>${changeIcon} ${safeChange.toFixed(2)}% (24h)</p>`;
        }
        
        try { btcContainer.dataset.btcPriceUsd = String(btcPrice); btcContainer.dataset.btcChange24h = String(change); } catch(e){}
    }

    // Cập nhật chỉ số Sợ hãi & Tham lam
    const fngContainer = selectDashboardElementByLang('fear-greed-container');
    const fngValue = parseInt(data.fng_value, 10);
    if (WS_DEBUG) console.log('🔍 [DEBUG] F&G Value parsed:', fngValue, 'from:', data.fng_value);
    if (!isNaN(fngValue) && fngContainer) {
        const fngConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                { limit: 45, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
            ]
        };
        createGauge(fngContainer, fngValue, fngConfig);
        try { fngContainer.dataset.value = String(fngValue); } catch(e){}
    } else {
        if (WS_DEBUG) console.log('❌ [DEBUG] F&G Value invalid or container not found');
        displayError('fear-greed-container', 'Giá trị F&G không hợp lệ.');
    }

    // Cập nhật chỉ số RSI
    const rsiContainer = selectDashboardElementByLang('rsi-container');
    const rsiValue = parseFloat(data.btc_rsi_14);
    if (WS_DEBUG) console.log('🔍 [DEBUG] RSI Value parsed:', rsiValue, 'from:', data.btc_rsi_14);
    if (rsiValue !== null && rsiValue !== undefined && !isNaN(rsiValue) && rsiContainer) {
        const rsiConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
            ]
        };
        createGauge(rsiContainer, rsiValue, rsiConfig);
        try { rsiContainer.dataset.value = String(rsiValue); } catch(e){}
    } else {
        if (WS_DEBUG) console.log('❌ [DEBUG] RSI Value invalid or container not found');
        displayError('rsi-container', 'Không nhận được giá trị RSI.');
    }

    if (WS_DEBUG) console.log('✅ Dashboard UI updated successfully');
    
    // Update dominance info if available
    if (data.btc_market_cap_percentage && WS_DEBUG) {
        console.log('🔍 [DEBUG] BTC Dominance:', data.btc_market_cap_percentage);
        // You can add dominance display logic here if needed
    }
    
    if (data.eth_market_cap_percentage && WS_DEBUG) {
        console.log('🔍 [DEBUG] ETH Dominance:', data.eth_market_cap_percentage);
        // You can add dominance display logic here if needed
    }
    
    // Update last updated time
    updateLastUpdatedTime();
}

// ===== UI ENHANCEMENT FUNCTIONS =====

/**
 * Show BTC refresh indicator briefly
 */
function showBtcRefreshIndicator() {
    const indicator = document.getElementById('btc-refresh-indicator');
    if (indicator) {
        indicator.style.opacity = '1';
        setTimeout(() => {
            indicator.style.opacity = '0';
        }, 1000);
    }
}

/**
 * Update WebSocket status indicator
 */
function updateWebSocketStatus(status, message) {
    const statusElement = document.getElementById('websocket-status');
    if (!statusElement) return;
    
    const statusClasses = {
        'connecting': 'bg-yellow-100 text-yellow-800',
        'connected': 'bg-green-100 text-green-800',
        'disconnected': 'bg-red-100 text-red-800',
        'error': 'bg-red-100 text-red-800'
    };
    
    const statusIcons = {
        'connecting': 'fas fa-sync-alt animate-spin text-yellow-600',
        'connected': 'fas fa-check-circle text-green-600',
        'disconnected': 'fas fa-times-circle text-red-600',
        'error': 'fas fa-exclamation-triangle text-red-600'
    };
    
    const statusIcon = statusIcons[status] || 'fas fa-circle text-gray-400';
    const statusClass = statusClasses[status] || 'bg-gray-100 text-gray-800';
    
    statusElement.className = `inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${statusClass}`;
    statusElement.innerHTML = `
        <div class="w-2 h-2 mr-2">
            <i class="${statusIcon}"></i>
        </div>
        <span>${message}</span>
    `;
}

/**
 * Update last updated time
 */
function updateLastUpdatedTime() {
    const timeElement = document.getElementById('last-update-time');
    if (timeElement) {
        const now = new Date();
        const lang = (window.languageManager && window.languageManager.currentLanguage) || 'vi';
        
        try {
            const timeOptions = { 
                hour: '2-digit', 
                minute: '2-digit', 
                second: '2-digit',
                timeZone: 'Asia/Ho_Chi_Minh', 
                hour12: false 
            };
            const timeFormatter = new Intl.DateTimeFormat((lang === 'en') ? 'en-US' : 'vi-VN', timeOptions);
            timeElement.textContent = timeFormatter.format(now) + ' (GMT+7)';
        } catch (e) {
            timeElement.textContent = now.toLocaleTimeString();
        }
    }
}

/**
 * Manual refresh function
 */
async function manualRefreshDashboard() {
    const refreshBtn = document.getElementById('refresh-dashboard');
    if (refreshBtn) {
        const originalText = refreshBtn.innerHTML;
        refreshBtn.innerHTML = '<i class="fas fa-sync-alt animate-spin mr-2"></i> <span data-i18n="refreshing">Đang cập nhật...</span>';
        refreshBtn.disabled = true;
    }
    
    try {
        await fetchDashboardSummary();
        
        // Show success feedback
        updateWebSocketStatus('connected', getTranslatedText('data-updated') || 'Dữ liệu đã được cập nhật');
        
        setTimeout(() => {
            if (dashboardWebSocket && dashboardWebSocket.socket && dashboardWebSocket.socket.readyState === WebSocket.OPEN) {
                updateWebSocketStatus('connected', getTranslatedText('real-time-connected') || 'Kết nối thời gian thực');
            }
        }, 2000);
        
    } catch (error) {
        console.error('Manual refresh failed:', error);
        updateWebSocketStatus('error', getTranslatedText('refresh-failed') || 'Lỗi cập nhật dữ liệu');
    } finally {
        // Reset button
        setTimeout(() => {
            if (refreshBtn) {
                refreshBtn.innerHTML = '<i class="fas fa-sync-alt mr-2"></i> <span data-i18n="refresh-data">Cập nhật dữ liệu</span>';
                refreshBtn.disabled = false;
            }
        }, 1000);
    }
}

/**
 * Re-render dashboard UI from previously cached summary data without re-fetching.
 * Useful when only language changed.
 */
function renderDashboardFromCache(lang) {
    const data = window.dashboardSummaryCache;
    if (!data) return;

    // market cap
    const marketCapContainer = selectDashboardElementByLang('market-cap-container', lang);
    if (marketCapContainer) {
        marketCapContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(Number(marketCapContainer.dataset.marketCap || data.market_cap))}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    // volume
    const volumeContainer = selectDashboardElementByLang('volume-24h-container', lang);
    if (volumeContainer) {
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(Number(volumeContainer.dataset.volume24h || data.volume_24h))}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    // btc
    const btcContainer = selectDashboardElementByLang('btc-price-container', lang);
    if (btcContainer) {
        const price = btcContainer.dataset.btcPriceUsd || data.btc_price_usd;
        const change = Number(btcContainer.dataset.btcChange24h || data.btc_change_24h || 0);
        const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
        btcContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + (price ? Number(price).toLocaleString('en-US') : 'N/A')}</p>
            <p class="text-sm font-semibold ${changeClass}">${!isNaN(change) ? change.toFixed(2) : 'N/A'}% (24h)</p>`;
    }

    // Fear & Greed gauge
    const fngContainer = selectDashboardElementByLang('fear-greed-container', lang);
    const fngVal = fngContainer ? Number(fngContainer.dataset.value || data.fng_value) : null;
    if (fngContainer && !isNaN(fngVal)) {
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
        try { createGauge(fngContainer, fngVal, fngConfig); } catch(e) { console.error('createGauge lỗi khi render từ cache', e); }
    }

    // RSI
    const rsiContainer = selectDashboardElementByLang('rsi-container', lang);
    const rsiVal = rsiContainer ? Number(rsiContainer.dataset.value || data.btc_rsi_14) : null;
    if (rsiContainer && rsiVal !== null && !isNaN(rsiVal)) {
        const rsiConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
            ]
        };
        try { createGauge(rsiContainer, rsiVal, rsiConfig); } catch(e) { console.error('createGauge lỗi khi render RSI từ cache', e); }
    }
}

/**
 * Fetch toàn bộ dữ liệu cho dashboard từ endpoint tổng hợp.
 * Ưu tiên WebSocket, fallback sang HTTP API nếu cần.
 */
async function fetchDashboardSummary() {
    // Chỉ chạy nếu có các element dashboard
    if (!document.getElementById('market-cap-container') && 
        !document.getElementById('volume-24h-container') && 
        !document.getElementById('btc-price-container')) {
        return; // Không phải trang dashboard, bỏ qua
    }

    try {
        // Try WebSocket first for real-time data
        if (dashboardWebSocket && dashboardWebSocket.socket && 
            dashboardWebSocket.socket.readyState === WebSocket.OPEN) {
            if (WS_DEBUG) console.log('🔗 Using WebSocket for dashboard data');
            return; // WebSocket will handle updates
        }

        // Fallback to HTTP API
        if (WS_DEBUG) console.log('📡 Fetching dashboard data via HTTP API...');
        const response = await fetch('/api/crypto/dashboard-summary', {
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        
        // Kiểm tra nếu response trống
        if (!response.body) {
            throw new Error('Server trả về response trống');
        }
        
        // Kiểm tra content-type để đảm bảo response là JSON
        const contentType = response.headers.get('content-type');
        if (!contentType || !contentType.includes('application/json')) {
            console.error('Response không phải JSON:', contentType);
            const responseText = await response.text();
            console.error('Response text:', responseText);
            throw new Error(`Server trả về định dạng không hợp lệ: ${contentType || 'unknown'}`);
        }
        
        // Đọc response text trước để kiểm tra
        const responseText = await response.text();
        if (!responseText || responseText.trim().length === 0) {
            throw new Error('Server trả về nội dung trống');
        }
        
        if (!response.ok) {
            let errorData;
            try {
                errorData = JSON.parse(responseText);
            } catch (jsonError) {
                console.error('Lỗi parse JSON từ error response:', jsonError);
                console.error('Error response text:', responseText);
                throw new Error(`Lỗi server ${response.status}: Không thể đọc response`);
            }
            const errorMessage = errorData.errors ? JSON.stringify(errorData.errors) : `Lỗi server ${response.status}`;
            throw new Error(errorMessage);
        }
        
        let data;
        try {
            data = JSON.parse(responseText);
        } catch (jsonError) {
            console.error('Lỗi parse JSON từ success response:', jsonError);
            console.error('Success response text:', responseText);
            throw new Error('Server trả về dữ liệu không hợp lệ');
        }

        // Process the data (same as WebSocket handler)
        updateDashboardFromData(data);

        // Cache the last successful summary so we can re-render visuals on language change without re-fetching
        try { window.dashboardSummaryCache = data; } catch(e) {}

    } catch (error) {
        console.error('Lỗi fetchDashboardSummary:', error);
        console.error('Error stack:', error.stack);
        
        // Hiển thị fallback data thay vì chỉ hiển thị lỗi
        displayFallbackData();

        // Nếu đã có cached data hoặc UI đã được cập nhật, không cần hiện toast lỗi
        const hasCached = !!window.dashboardSummaryCache;
        const uiPopulated = document.getElementById('market-cap-container') && document.getElementById('market-cap-container').innerText.trim().length > 0;

        if (!hasCached && !uiPopulated) {
            // Hiển thị thông báo lỗi nhẹ nhàng nếu không có dữ liệu để hiển thị
            showErrorNotification(getTranslatedText('connection-issue'));
        } else {
            if (WS_DEBUG) console.log('ℹ️ Có cached data hoặc UI đã hiển thị, bỏ qua thông báo lỗi');
        }
    }
}

// ===== FALLBACK & ERROR HANDLING =====

/**
 * Hiển thị dữ liệu mặc định khi API không khả dụng
 */
function displayFallbackData() {
    // Hiển thị market cap fallback
    const marketCapContainer = document.getElementById('market-cap-container');
    if (marketCapContainer) {
        marketCapContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    // Hiển thị volume fallback
    const volumeContainer = document.getElementById('volume-24h-container');
    if (volumeContainer) {
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    // Hiển thị BTC price fallback
    const btcContainer = document.getElementById('btc-price-container');
    if (btcContainer) {
        btcContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">Bitcoin</p>`;
    }

    // Hiển thị F&G fallback
    const fngContainer = document.getElementById('fear-greed-container');
    if (fngContainer) {
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
        createGauge(fngContainer, 50, fngConfig); // Default neutral value
    }

    // Hiển thị RSI fallback
    const rsiContainer = document.getElementById('rsi-container');
    if (rsiContainer) {
        const rsiConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
            ]
        };
        createGauge(rsiContainer, 50, rsiConfig); // Default neutral value
    }
}

/**
 * Hiển thị thông báo lỗi dạng toast
 */
function showErrorNotification(message) {
    // Tạo toast notification nếu chưa có
    let notification = document.getElementById('error-notification');
    if (!notification) {
        notification = document.createElement('div');
        notification.id = 'error-notification';
        notification.className = 'fixed top-4 right-4 bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded shadow-lg z-50 max-w-sm';
        document.body.appendChild(notification);
    }
    
    notification.innerHTML = `
        <div class="flex items-center">
            <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"></path>
            </svg>
            <span class="text-sm">${message}</span>
        </div>
    `;
    
    // Tự động ẩn sau 5 giây
    setTimeout(() => {
        if (notification && notification.parentNode) {
            notification.parentNode.removeChild(notification);
        }
    }, 5000);
}

// ===== REPORT NAVIGATION =====

/**
 * Tải nội dung báo cáo từ file tĩnh và tạo mục lục điều hướng.
 */
// async function CreateNav() {
//     try {
//         const reportContainer = document.getElementById('report-container');
//         const navLinksContainer = document.getElementById('report-nav-links');

//         // Thoát sớm nếu các container chính không tồn tại để tránh lỗi
//         if (!reportContainer || !navLinksContainer) {
//             console.error("Không tìm thấy container cho báo cáo (#report-container) hoặc mục lục (#report-nav-links).");
//             return;
//         }

//         // Ngắt observer cũ (nếu có) để tránh quan sát trùng lặp
//         if (reportContainer._navObserver) {
//             try { reportContainer._navObserver.disconnect(); } catch(e){}
//             reportContainer._navObserver = null;
//         }

//         // Xóa nội dung cũ của mục lục trước khi tạo mới để tránh trùng lặp
//         navLinksContainer.innerHTML = '';

//         // Nếu có 2 container nội dung (vi/en), chỉ lấy các section từ phần đang hiển thị
//         const viContainer = document.getElementById('report-content-vi');
//         const enContainer = document.getElementById('report-content-en');
//         let activeContent = reportContainer; // fallback: toàn bộ reportContainer

//         if (viContainer || enContainer) {
//             const viVisible = viContainer && window.getComputedStyle(viContainer).display !== 'none';
//             const enVisible = enContainer && window.getComputedStyle(enContainer).display !== 'none';
//             if (viVisible) activeContent = viContainer;
//             else if (enVisible) activeContent = enContainer;
//             else activeContent = viContainer || enContainer || reportContainer;
//         }

//         const reportSections = activeContent.querySelectorAll('section');

//         // Build navigation links only from the active content's sections
//         reportSections.forEach(section => {
//             const h2 = section.querySelector('h2');
//             if (h2 && section.id) {
//                 const li = document.createElement('li');
//                 const a = document.createElement('a');
//                 a.href = `#${section.id}`;
//                 // remove any icon node inside h2 when constructing the label
//                 const h2Text = h2.cloneNode(true);
//                 const icon = h2Text.querySelector('i');
//                 if (icon && icon.parentNode) icon.parentNode.removeChild(icon);
//                 a.textContent = h2Text.textContent.trim();
//                 a.classList.add('block', 'py-1', 'px-2', 'rounded');
//                 // smooth scroll on click và active ngay lập tức
//                 a.addEventListener('click', (e) => {
//                     e.preventDefault();
                    
//                     const target = activeContent.querySelector(`#${section.id}`);
//                     if (target) {
//                         // Active ngay lập tức khi click
//                         navLinksContainer.querySelectorAll('a').forEach(link => link.classList.remove('active'));
//                         a.classList.add('active');
                        
//                         // Scroll tới target
//                         target.scrollIntoView({ behavior: 'smooth', block: 'start' });
//                     }
//                 });
//                 li.appendChild(a);
//                 navLinksContainer.appendChild(li);
//             }
//         });

//         const navLinks = navLinksContainer.querySelectorAll('a');

//         // Quan sát các section để tự động active nav link khi scroll
//         const observer = new IntersectionObserver(() => {
//             // More deterministic selection:
//             // Choose the section whose top is the closest to the anchor line (20% from top)
//             // Preference: sections with top <= anchor (the one closest below the anchor). If none, pick the nearest section below the anchor.
//             const viewportHeight = window.innerHeight;
//             const anchor = viewportHeight * 0.2; // 20% from top

//             let bestSection = null;
//             let bestTop = -Infinity; // for tops <= anchor we want the maximum (closest to anchor from above)

//             // First pass: find section top <= anchor and still at least partially visible
//             reportSections.forEach(section => {
//                 const rect = section.getBoundingClientRect();
//                 // ignore sections that are completely scrolled past
//                 if (rect.bottom <= 0 || rect.top >= viewportHeight) return;
//                 if (rect.top <= anchor) {
//                     if (rect.top > bestTop) {
//                         bestTop = rect.top;
//                         bestSection = section;
//                     }
//                 }
//             });

//             // Second pass: if none found, pick the section whose top is the smallest positive distance below anchor
//             if (!bestSection) {
//                 let minBelow = Infinity;
//                 reportSections.forEach(section => {
//                     const rect = section.getBoundingClientRect();
//                     if (rect.bottom <= 0 || rect.top >= viewportHeight) return;
//                     if (rect.top > anchor && rect.top < minBelow) {
//                         minBelow = rect.top;
//                         bestSection = section;
//                     }
//                 });
//             }

//             if (bestSection) {
//                 const targetId = bestSection.id;
//                 navLinks.forEach(link => {
//                     const isTarget = link.getAttribute('href').substring(1) === targetId;
//                     link.classList.toggle('active', isTarget);
//                 });
//             }
//         }, {
//             root: null,
//             rootMargin: "0px",
//             threshold: [0, 0.1, 0.25, 0.5, 1.0]
//         });

//         // Quan sát tất cả sections
//         reportSections.forEach(section => {
//             observer.observe(section);
//         });

//         // Thiết lập nav link đầu tiên làm active ban đầu nếu chưa có active nào
//         if (navLinks.length > 0 && !navLinksContainer.querySelector('a.active')) {
//             navLinks[0].classList.add('active');
//         }

//         // Lưu observer vào DOM node để có thể disconnect khi tạo lại nav
//         reportContainer._navObserver = observer;

//     } catch (error) {
//         console.error('Lỗi tải báo cáo:', error);
//         const reportContainer = document.getElementById('report-container');
//         if (reportContainer) {
//             reportContainer.innerHTML = '<p class="text-red-600 font-semibold">Lỗi: Không thể tải nội dung báo cáo chi tiết.</p>';
//         }
//     }
// }

// ===== MAIN INITIALIZATION =====

// Global WebSocket instance
let dashboardWebSocket = null;

/**
 * Hàm khởi tạo dashboard
 */
function initDashboard() {
    // Chỉ chạy nếu đang ở trang dashboard (có các element dashboard)
    if (document.getElementById('market-cap-container') || 
        document.getElementById('volume-24h-container') || 
        document.getElementById('btc-price-container')) {
        
        if (WS_DEBUG) console.log('🚀 Initializing dashboard...');
        
        // Nạp trước dữ liệu dashboard trước khi khởi tạo WebSocket
        fetchDashboardSummary();
        
        // Set up refresh button
        const refreshBtn = document.getElementById('refresh-dashboard');
        if (refreshBtn) {
            refreshBtn.addEventListener('click', manualRefreshDashboard);
        }
        
        // Set initial status sau khi đã nạp dữ liệu
        updateWebSocketStatus('connecting', getTranslatedText('connecting') || 'Đang kết nối...');
        
        // Initialize WebSocket connection for real-time updates
        if (WS_DEBUG) {
            console.log('🔍 [DEBUG] Checking WebSocket initialization...');
            console.log('  🔍 dashboardWebSocket exists:', !!dashboardWebSocket);
            console.log('  🔍 DashboardWebSocket class exists:', typeof DashboardWebSocket !== 'undefined');
        }
        
        if (!dashboardWebSocket && typeof DashboardWebSocket !== 'undefined') {
            if (WS_DEBUG) console.log('🚀 [DEBUG] Creating new WebSocket connection...');
            dashboardWebSocket = new DashboardWebSocket();
            dashboardWebSocket.connect();
            if (WS_DEBUG) console.log('🚀 WebSocket connection initialized');
        } else {
            if (WS_DEBUG) console.log('⚠️ [DEBUG] WebSocket initialization skipped:', {
                dashboardWebSocketExists: !!dashboardWebSocket,
                DashboardWebSocketClassExists: typeof DashboardWebSocket !== 'undefined'
            });
        }
        
        // Đặt lịch gọi lại hàm tổng hợp sau mỗi 10 phút (backup cho WebSocket)
        // WebSocket sẽ update real-time, nhưng giữ interval làm fallback
        setInterval(() => {
            // Chỉ fetch qua HTTP nếu WebSocket không khả dụng
            if (!dashboardWebSocket || 
                !dashboardWebSocket.socket || 
                dashboardWebSocket.socket.readyState !== WebSocket.OPEN) {
                if (WS_DEBUG) console.log('📡 WebSocket unavailable, falling back to HTTP polling');
                updateWebSocketStatus('connecting', getTranslatedText('reconnecting') || 'Đang kết nối lại...');
                fetchDashboardSummary();
            }
        }, 600000); // 10 phút
        
        // Lắng nghe sự kiện thay đổi ngôn ngữ — chỉ cập nhật UI (nav & visuals), không re-fetch dữ liệu
        window.addEventListener('languageChanged', (e) => {
            const lang = e?.detail?.language;
            // Rebuild navigation to match the newly visible report content (VI/EN)
            try { CreateNav(); } catch(err) { console.error('CreateNav lỗi sau khi đổi ngôn ngữ', err); }
            // Re-render dashboard cards & small charts from cached summary if available (no network call)
            try { if (window.dashboardSummaryCache) renderDashboardFromCache(lang); } catch(err) { console.error('renderDashboardFromCache lỗi', err); }
        });

        // Cleanup WebSocket on page unload
        window.addEventListener('beforeunload', () => {
            if (dashboardWebSocket) {
                dashboardWebSocket.disconnect();
                if (WS_DEBUG) console.log('🔌 WebSocket disconnected on page unload');
            }
        });
    }
    
    CreateNav();
}

// Khởi tạo dashboard khi DOM ready
document.addEventListener('DOMContentLoaded', () => {
    initDashboard();
});
