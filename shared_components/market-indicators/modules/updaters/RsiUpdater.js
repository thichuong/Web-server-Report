/**
 * RsiUpdater - Updates BTC RSI 14 indicator
 */

import { BaseUpdater } from './BaseUpdater.js';

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class RsiUpdater extends BaseUpdater {
    constructor(chartRenderer) {
        super('btc-rsi-14-indicator');
        this.chartRenderer = chartRenderer;
    }
    
    /**
     * Get RSI classification
     * @param {number} rsi - RSI value (0-100)
     * @returns {Object} { class: string, label: string }
     */
    getRsiClass(rsi) {
        if (rsi < 30) {
            return { class: 'oversold', label: 'Oversold' };
        } else if (rsi > 70) {
            return { class: 'overbought', label: 'Overbought' };
        } else {
            return { class: 'neutral', label: 'Neutral' };
        }
    }
    
    /**
     * Update RSI display
     * @param {number} value - RSI value (0-100)
     */
    update(value) {
        if (!this.exists()) return;
        
        const rsi = parseFloat(value) || 0;
        const rsiInfo = this.getRsiClass(rsi);
        
        const html = `
            <div class="index-display flex items-center justify-between">
                <div class="index-value ${rsiInfo.class}">${rsi.toFixed(1)}</div>
                <div class="text-right">
                    <div class="index-label">${rsiInfo.label}</div>
                </div>
            </div>
        `;
        
        this.setHTML(html);
        
        // Render RSI gauge chart
        if (this.chartRenderer) {
            this.chartRenderer.renderGaugeChart('btc-rsi', rsi);
        }
        
        debugLog('✅ BTC RSI 14 updated:', { rsi: rsi.toFixed(1), class: rsiInfo.class });
    }
}
