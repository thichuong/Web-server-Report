/**
 * DominanceUpdater - Updates BTC/ETH Dominance indicators
 */

import { BaseUpdater } from './BaseUpdater.js';

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class DominanceUpdater extends BaseUpdater {
    constructor(crypto, stateManager, chartRenderer) {
        super(`${crypto}-dominance-indicator`);
        this.crypto = crypto; // 'btc' or 'eth'
        this.stateManager = stateManager;
        this.chartRenderer = chartRenderer;
    }
    
    /**
     * Update dominance display
     * @param {number} value - Dominance percentage
     */
    update(value) {
        if (!this.exists()) return;
        
        const dominance = parseFloat(value) || 0;
        
        // Add to history via state manager
        if (this.stateManager) {
            this.stateManager.addDominanceHistory(this.crypto, dominance);
        }
        
        const html = `
            <div class="index-value">${dominance.toFixed(1)}%</div>
        `;
        
        this.setHTML(html);
        
        // Render dominance chart
        if (this.chartRenderer) {
            this.chartRenderer.renderDominanceChart(this.crypto, dominance);
        }
        
        debugLog(`✅ ${this.crypto.toUpperCase()} dominance updated:`, { dominance: dominance.toFixed(1) });
    }
}
