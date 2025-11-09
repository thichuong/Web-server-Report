/**
 * StateManager - Manages application state and caching
 * 
 * Responsibilities:
 * - Cache market data to avoid unnecessary updates
 * - Manage dominance history for charts
 * - Provide state comparison for change detection
 */

const DEBUG_MODE = true;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class StateManager {
    constructor(config = {}) {
        this.maxHistoryPoints = config.maxHistoryPoints || 20;
        
        // Data cache
        this.cache = {
            marketCap: null,
            volume24h: null,
            fearGreedIndex: null,
            btcDominance: null,
            ethDominance: null,
            btcRsi14: null,
            usStockIndices: null,
            cryptoPrices: {}
        };
        
        // Dominance history for charts
        this.dominanceHistory = {
            btc: [],
            eth: []
        };
    }
    
    /**
     * Check if data has changed since last update
     * @param {string} key - Cache key
     * @param {*} newValue - New value to compare
     * @param {number} tolerance - Tolerance for floating point comparison
     * @returns {boolean} True if data has changed
     */
    hasChanged(key, newValue, tolerance = 0.01) {
        const cached = this.cache[key];
        
        if (cached === null || cached === undefined) {
            return true; // No cached value, consider it changed
        }
        
        // For objects (like marketCap, volume24h)
        if (typeof newValue === 'object' && newValue !== null) {
            return JSON.stringify(cached) !== JSON.stringify(newValue);
        }
        
        // For numbers with tolerance
        if (typeof newValue === 'number') {
            return Math.abs(cached - newValue) >= tolerance;
        }
        
        // For other types (string, boolean, etc.)
        return cached !== newValue;
    }
    
    /**
     * Update cache with new value
     * @param {string} key - Cache key
     * @param {*} value - Value to cache
     */
    set(key, value) {
        this.cache[key] = value;
        debugLog(`📦 StateManager: Cached ${key}:`, value);
    }
    
    /**
     * Get cached value
     * @param {string} key - Cache key
     * @returns {*} Cached value or null
     */
    get(key) {
        return this.cache[key];
    }
    
    /**
     * Add dominance value to history
     * @param {string} crypto - 'btc' or 'eth'
     * @param {number} value - Dominance value
     */
    addDominanceHistory(crypto, value) {
        if (!this.dominanceHistory[crypto]) {
            console.warn(`Unknown crypto for dominance history: ${crypto}`);
            return;
        }
        
        this.dominanceHistory[crypto].push({
            value,
            timestamp: Date.now()
        });
        
        // Keep only last N points
        if (this.dominanceHistory[crypto].length > this.maxHistoryPoints) {
            this.dominanceHistory[crypto].shift();
        }
        
        debugLog(`📈 StateManager: Added ${crypto.toUpperCase()} dominance to history (${this.dominanceHistory[crypto].length}/${this.maxHistoryPoints})`);
    }
    
    /**
     * Get dominance history for charting
     * @param {string} crypto - 'btc' or 'eth'
     * @returns {Array} Array of {value, timestamp} objects
     */
    getDominanceHistory(crypto) {
        return this.dominanceHistory[crypto] || [];
    }
    
    /**
     * Clear all cached data
     */
    clear() {
        this.cache = {
            marketCap: null,
            volume24h: null,
            fearGreedIndex: null,
            btcDominance: null,
            ethDominance: null,
            btcRsi14: null,
            usStockIndices: null,
            cryptoPrices: {}
        };
        
        this.dominanceHistory = {
            btc: [],
            eth: []
        };
        
        debugLog('🧹 StateManager: Cache cleared');
    }
    
    /**
     * Get cache statistics
     * @returns {Object} Cache statistics
     */
    getStats() {
        const cachedKeys = Object.entries(this.cache)
            .filter(([key, value]) => value !== null && value !== undefined)
            .map(([key]) => key);
        
        return {
            cachedItems: cachedKeys.length,
            totalKeys: Object.keys(this.cache).length,
            cachedKeys,
            historyPoints: {
                btc: this.dominanceHistory.btc.length,
                eth: this.dominanceHistory.eth.length
            }
        };
    }
}
