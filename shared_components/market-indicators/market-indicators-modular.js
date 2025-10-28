/**
 * Market Indicators Dashboard - Main Orchestrator
 * 
 * Refactored version with modular architecture
 * - WebSocketManager: Handles WebSocket connections
 * - DataProcessor: Processes market data
 * - StateManager: Manages state and caching
 * - Various Updaters: Update UI elements
 * - ChartRenderer: Renders charts
 */

import { WebSocketManager } from './modules/core/WebSocketManager.js';
import { DataProcessor } from './modules/core/DataProcessor.js';
import { StateManager } from './modules/core/StateManager.js';
import { MarketCapUpdater } from './modules/updaters/MarketCapUpdater.js';
import { VolumeUpdater } from './modules/updaters/VolumeUpdater.js';
import { FearGreedUpdater } from './modules/updaters/FearGreedUpdater.js';
import { DominanceUpdater } from './modules/updaters/DominanceUpdater.js';
import { RsiUpdater } from './modules/updaters/RsiUpdater.js';
import { CryptoPriceUpdater } from './modules/updaters/CryptoPriceUpdater.js';
import { StockIndexUpdater } from './modules/updaters/StockIndexUpdater.js';
import { ChartRenderer } from './modules/charts/ChartRenderer.js';

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

class MarketIndicatorsDashboard {
    constructor() {
        debugLog('🚀 Initializing Market Indicators Dashboard (Modular)');
        
        // Core modules
        this.dataProcessor = new DataProcessor();
        this.stateManager = new StateManager();
        this.chartRenderer = new ChartRenderer();
        
        // UI Updaters
        this.updaters = {
            marketCap: new MarketCapUpdater(),
            volume: new VolumeUpdater(),
            fearGreed: new FearGreedUpdater(this.chartRenderer),
            btcDominance: new DominanceUpdater('btc', this.stateManager, this.chartRenderer),
            ethDominance: new DominanceUpdater('eth', this.stateManager, this.chartRenderer),
            btcRsi: new RsiUpdater(this.chartRenderer),
            cryptoPrice: new CryptoPriceUpdater(),
            stockIndex: new StockIndexUpdater()
        };
        
        // WebSocket Manager
        this.wsManager = new WebSocketManager({
            onMessage: this.handleWebSocketMessage.bind(this),
            onConnected: this.handleConnected.bind(this),
            onDisconnected: this.handleDisconnected.bind(this),
            onError: this.handleError.bind(this)
        });
        
        // Connection status element
        this.connectionStatusElement = document.getElementById('connection-status');
        
        this.init();
    }
    
    init() {
        debugLog('🔧 Initializing components');
        
        // Remove skeletons after short delay
        setTimeout(() => this.removeSkeletons(), 100);
        
        // Request initial data
        this.requestInitialData();
        
        // Connect WebSocket
        this.wsManager.connect();
        
        // Start data refresh
        this.startDataRefresh();
    }
    
    removeSkeletons() {
        const skeletons = document.querySelectorAll('.skeleton-loader, .skeleton');
        skeletons.forEach(skeleton => skeleton.remove());
        debugLog('🦴 Removed all skeleton loaders');
    }
    
    handleWebSocketMessage(message) {
        const now = new Date().toLocaleTimeString();
        debugLog(`📨 [${now}] WebSocket message type: ${message.type}`);
        
        try {
            // Process data based on message type
            let data = message.data || message;
            
            if (message.type === 'dashboard_data' || 
                message.type === 'dashboard_update' || 
                message.type === 'market_update') {
                this.updateDashboard(data);
            } else if (message.type === 'connected' || message.type === 'pong') {
                debugLog(`✅ ${message.type} message received`);
            } else {
                // Try to handle as generic market data
                if (data.btc_price_usd || data.market_cap_usd || data.fng_value) {
                    this.updateDashboard(data);
                }
            }
        } catch (error) {
            console.error('❌ Error handling WebSocket message:', error);
        }
    }
    
    updateDashboard(rawData) {
        // Process raw data
        const data = this.dataProcessor.process(rawData);
        
        // Validate data
        const validation = this.dataProcessor.validate(data);
        if (!validation.isValid) {
            console.warn('⚠️ Data validation failed:', validation.errors);
        }
        
        // Update each component only if data changed
        if (data.marketCap && this.stateManager.hasChanged('marketCap', data.marketCap)) {
            this.updaters.marketCap.update(data.marketCap);
            this.stateManager.set('marketCap', data.marketCap);
        }
        
        if (data.volume24h && this.stateManager.hasChanged('volume24h', data.volume24h)) {
            this.updaters.volume.update(data.volume24h);
            this.stateManager.set('volume24h', data.volume24h);
        }
        
        if (data.fearGreed !== null && this.stateManager.hasChanged('fearGreedIndex', data.fearGreed)) {
            this.updaters.fearGreed.update(data.fearGreed);
            this.stateManager.set('fearGreedIndex', data.fearGreed);
        }
        
        if (data.btcDominance !== null && this.stateManager.hasChanged('btcDominance', data.btcDominance)) {
            this.updaters.btcDominance.update(data.btcDominance);
            this.stateManager.set('btcDominance', data.btcDominance);
        }
        
        if (data.ethDominance !== null && this.stateManager.hasChanged('ethDominance', data.ethDominance)) {
            this.updaters.ethDominance.update(data.ethDominance);
            this.stateManager.set('ethDominance', data.ethDominance);
        }
        
        if (data.btcRsi14 !== null && this.stateManager.hasChanged('btcRsi14', data.btcRsi14)) {
            this.updaters.btcRsi.update(data.btcRsi14);
            this.stateManager.set('btcRsi14', data.btcRsi14);
        }
        
        if (data.usStockIndices) {
            if (this.stateManager.hasChanged('usStockIndices', data.usStockIndices)) {
                this.updaters.stockIndex.update(data.usStockIndices);
                this.stateManager.set('usStockIndices', data.usStockIndices);
            }
        }
        
        if (data.cryptoPrices && data.cryptoPrices.length > 0) {
            this.updaters.cryptoPrice.updateBatch(data.cryptoPrices);
        }
        
        debugLog('✅ Dashboard update completed');
    }
    
    handleConnected() {
        debugLog('✅ WebSocket connected');
        this.updateConnectionStatus('connected');
        this.requestFreshData();
    }
    
    handleDisconnected() {
        debugLog('🔌 WebSocket disconnected');
        this.updateConnectionStatus('disconnected');
    }
    
    handleError(error) {
        console.error('❌ WebSocket error:', error);
        this.updateConnectionStatus('offline');
    }
    
    updateConnectionStatus(status) {
        if (!this.connectionStatusElement) return;
        
        const statusConfig = {
            'connected': { icon: '🟢', text: 'Đang kết nối', class: 'connected' },
            'disconnected': { icon: '🟡', text: 'Mất kết nối', class: 'warning' },
            'offline': { icon: '🔴', text: 'Ngoại tuyến', class: 'error' }
        };
        
        const config = statusConfig[status] || statusConfig.offline;
        this.connectionStatusElement.innerHTML = `${config.icon} <span data-i18n="connection-${status}">${config.text}</span>`;
        this.connectionStatusElement.className = `connection-status ${config.class}`;
    }
    
    requestInitialData() {
        debugLog('📡 Requesting initial market data');
        fetch('/api/market-summary')
            .then(res => res.json())
            .then(data => {
                debugLog('✅ Initial data received via HTTP');
                this.updateDashboard(data);
            })
            .catch(err => {
                console.warn('⚠️ Failed to fetch initial data:', err);
            });
    }
    
    requestFreshData() {
        if (this.wsManager.connected) {
            this.wsManager.send(JSON.stringify({ type: 'request_market_data' }));
            debugLog('📡 Requested fresh data via WebSocket');
        }
    }
    
    startDataRefresh() {
        // Refresh data every 2 minutes
        setInterval(() => {
            this.requestFreshData();
        }, 120000);
        
        debugLog('⏰ Data refresh scheduled every 2 minutes');
    }
    
    destroy() {
        debugLog('🧹 Destroying Market Indicators Dashboard');
        this.wsManager.close();
        this.stateManager.clear();
    }
}

// Initialize dashboard when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.marketIndicatorsDashboard = new MarketIndicatorsDashboard();
    });
} else {
    window.marketIndicatorsDashboard = new MarketIndicatorsDashboard();
}
