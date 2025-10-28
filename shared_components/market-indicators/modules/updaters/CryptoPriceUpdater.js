/**
 * CryptoPriceUpdater - Updates cryptocurrency price displays
 */

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class CryptoPriceUpdater {
    constructor() {
        this.symbolMap = {
            'BTCUSDT': { id: 'binance-btc-price', name: 'BTC' },
            'ETHUSDT': { id: 'binance-eth-price', name: 'ETH' },
            'SOLUSDT': { id: 'binance-sol-price', name: 'SOL' },
            'XRPUSDT': { id: 'binance-xrp-price', name: 'XRP' },
            'ADAUSDT': { id: 'binance-ada-price', name: 'ADA' },
            'LINKUSDT': { id: 'binance-link-price', name: 'LINK' },
            'BNBUSDT': { id: 'binance-bnb-price', name: 'BNB' }
        };
    }
    
    /**
     * Update crypto price display
     * @param {string} symbol - Crypto symbol (e.g., 'BTCUSDT')
     * @param {number} price - Current price
     * @param {number} changePercent - 24h change percentage
     */
    update(symbol, price, changePercent) {
        const symbolInfo = this.symbolMap[symbol];
        if (!symbolInfo) {
            debugLog(`⚠️ Unknown symbol: ${symbol}`);
            return;
        }
        
        const element = document.getElementById(symbolInfo.id);
        if (!element) {
            debugLog(`❌ Element not found: ${symbolInfo.id}`);
            return;
        }
        
        // Handle error state
        if (price === null || price === undefined || changePercent === 'Error') {
            const priceElement = element.querySelector('[data-price]');
            const changeElement = element.querySelector('[data-change]');
            if (priceElement) priceElement.textContent = '--';
            if (changeElement) {
                changeElement.textContent = 'N/A';
                changeElement.className = 'binance-price-change neutral';
            }
            return;
        }
        
        // Format price and change
        const changeClass = changePercent >= 0 ? 'positive' : 'negative';
        const changeSign = changePercent >= 0 ? '+' : '';
        
        const formattedPrice = price.toLocaleString('en-US', {
            minimumFractionDigits: 2,
            maximumFractionDigits: price < 1 ? 6 : 2
        });
        
        // Use textContent for faster updates
        const priceElement = element.querySelector('[data-price]');
        const changeElement = element.querySelector('[data-change]');
        
        if (priceElement) {
            priceElement.textContent = `$${formattedPrice}`;
        }
        
        if (changeElement) {
            changeElement.textContent = `${changeSign}${changePercent.toFixed(2)}%`;
            changeElement.className = `binance-price-change ${changeClass}`;
        }
        
        debugLog(`✅ Updated ${symbolInfo.name}: $${formattedPrice} (${changeSign}${changePercent.toFixed(2)}%)`);
    }
    
    /**
     * Update multiple crypto prices at once
     * @param {Array} prices - Array of {symbol, price, changePercent}
     */
    updateBatch(prices) {
        prices.forEach(({ symbol, price, changePercent }) => {
            this.update(symbol, price, changePercent);
        });
    }
}
