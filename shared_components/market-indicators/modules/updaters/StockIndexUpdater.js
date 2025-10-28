/**
 * StockIndexUpdater - Updates US Stock Index indicators
 */

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class StockIndexUpdater {
    constructor() {
        this.indexMap = {
            'DIA': { id: 'dia-indicator', name: 'DJIA' },
            'SPY': { id: 'spy-indicator', name: 'S&P 500' },
            'QQQM': { id: 'qqq-indicator', name: 'Nasdaq 100' }
        };
    }
    
    /**
     * Update single stock index
     * @param {string} indexKey - Index key (DIA, SPY, QQQM)
     * @param {Object} indexData - { price, change, change_percent, status }
     */
    updateIndex(indexKey, indexData) {
        const indexInfo = this.indexMap[indexKey];
        if (!indexInfo) {
            debugLog(`⚠️ Unknown index: ${indexKey}`);
            return;
        }
        
        const element = document.getElementById(indexInfo.id);
        if (!element) {
            debugLog(`❌ Element not found: ${indexInfo.id}`);
            return;
        }
        
        // Handle error or loading state
        if (!indexData || indexData.status !== 'success') {
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
        
        // Simple fade-in animation
        element.classList.add('fade-in');
        setTimeout(() => element.classList.remove('fade-in'), 300);
        
        debugLog(`✅ Updated ${indexInfo.name}: $${price.toFixed(2)} (${percentSign}${changePercent.toFixed(2)}%)`);
    }
    
    /**
     * Update all stock indices
     * @param {Object} indices - { DIA: {...}, SPY: {...}, QQQM: {...} }
     */
    update(indices) {
        if (!indices) return;
        
        Object.entries(this.indexMap).forEach(([key, info]) => {
            if (indices[key]) {
                this.updateIndex(key, indices[key]);
            }
        });
    }
}
