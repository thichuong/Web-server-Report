/**
 * DataProcessor - Processes and transforms market data
 * 
 * Responsibilities:
 * - Extract and normalize market data from WebSocket messages
 * - Validate data integrity
 * - Transform data into UI-friendly format
 */

const DEBUG_MODE = true;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class DataProcessor {
    constructor() {
        this._updateCounter = 0;
    }
    
    /**
     * Process incoming WebSocket message
     * @param {Object} message - WebSocket message
     * @returns {Object} Processed data categorized by type
     */
    process(message) {
        const result = {
            marketCap: null,
            volume24h: null,
            fearGreed: null,
            btcDominance: null,
            ethDominance: null,
            btcRsi14: null,
            usStockIndices: null,
            cryptoPrices: []
        };
        
        // Extract data based on message type
        let data = message.data || message;
        
        // Market Cap
        if (data.market_cap_usd !== undefined) {
            result.marketCap = {
                value: data.market_cap_usd,
                change: data.market_cap_change_percentage_24h_usd || 0
            };
        }
        
        // Volume 24h
        if (data.volume_24h_usd !== undefined) {
            result.volume24h = {
                value: data.volume_24h_usd,
                change: 0 // Volume change not available from current API
            };
        }
        
        // Fear & Greed Index
        if (data.fng_value !== undefined) {
            result.fearGreed = data.fng_value;
        }
        
        // BTC Dominance
        if (data.btc_market_cap_percentage !== undefined) {
            result.btcDominance = data.btc_market_cap_percentage;
        }
        
        // ETH Dominance
        if (data.eth_market_cap_percentage !== undefined) {
            result.ethDominance = data.eth_market_cap_percentage;
        }
        
        // BTC RSI 14
        if (data.btc_rsi_14 !== undefined) {
            result.btcRsi14 = data.btc_rsi_14;
        }
        
        // US Stock Indices
        if (data.us_stock_indices) {
            result.usStockIndices = data.us_stock_indices;
        }
        
        // Crypto Prices
        const cryptoSymbols = [
            { key: 'btc_price_usd', symbol: 'BTCUSDT', changeKey: 'btc_change_24h' },
            { key: 'eth_price_usd', symbol: 'ETHUSDT', changeKey: 'eth_change_24h' },
            { key: 'sol_price_usd', symbol: 'SOLUSDT', changeKey: 'sol_change_24h' },
            { key: 'xrp_price_usd', symbol: 'XRPUSDT', changeKey: 'xrp_change_24h' },
            { key: 'ada_price_usd', symbol: 'ADAUSDT', changeKey: 'ada_change_24h' },
            { key: 'link_price_usd', symbol: 'LINKUSDT', changeKey: 'link_change_24h' },
            { key: 'bnb_price_usd', symbol: 'BNBUSDT', changeKey: 'bnb_change_24h' }
        ];
        
        cryptoSymbols.forEach(({ key, symbol, changeKey }) => {
            if (data[key] !== undefined) {
                result.cryptoPrices.push({
                    symbol,
                    price: data[key],
                    changePercent: data[changeKey] || 0
                });
            }
        });
        
        // Log summary (throttled)
        this._updateCounter++;
        if (this._updateCounter % 10 === 1) {
            const availableData = Object.entries(result)
                .filter(([key, value]) => value !== null && (Array.isArray(value) ? value.length > 0 : true))
                .map(([key]) => key);
            debugLog(`✅ DataProcessor: Processed data - ${availableData.join(', ')}`);
        }
        
        return result;
    }
    
    /**
     * Validate market data
     * @param {Object} data - Data to validate
     * @returns {Object} Validation result with errors
     */
    validate(data) {
        const errors = [];
        
        // Validate market cap
        if (data.marketCap && (data.marketCap.value < 0 || isNaN(data.marketCap.value))) {
            errors.push('Invalid market cap value');
        }
        
        // Validate volume
        if (data.volume24h && (data.volume24h.value < 0 || isNaN(data.volume24h.value))) {
            errors.push('Invalid volume value');
        }
        
        // Validate fear & greed index (0-100)
        if (data.fearGreed !== null && (data.fearGreed < 0 || data.fearGreed > 100)) {
            errors.push('Invalid fear & greed index (must be 0-100)');
        }
        
        // Validate dominance percentages (0-100)
        if (data.btcDominance !== null && (data.btcDominance < 0 || data.btcDominance > 100)) {
            errors.push('Invalid BTC dominance percentage');
        }
        
        if (data.ethDominance !== null && (data.ethDominance < 0 || data.ethDominance > 100)) {
            errors.push('Invalid ETH dominance percentage');
        }
        
        // Validate RSI (0-100)
        if (data.btcRsi14 !== null && (data.btcRsi14 < 0 || data.btcRsi14 > 100)) {
            errors.push('Invalid RSI value (must be 0-100)');
        }
        
        // Validate crypto prices
        data.cryptoPrices.forEach(crypto => {
            if (crypto.price < 0 || isNaN(crypto.price)) {
                errors.push(`Invalid price for ${crypto.symbol}`);
            }
        });
        
        return {
            isValid: errors.length === 0,
            errors
        };
    }
}
