/**
 * FearGreedUpdater - Updates Fear & Greed Index
 */

import { BaseUpdater } from './BaseUpdater.js';

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class FearGreedUpdater extends BaseUpdater {
    constructor(chartRenderer) {
        super('fear-greed-indicator');
        this.chartRenderer = chartRenderer;
    }
    
    /**
     * Get index classification
     * @param {number} index - Fear & Greed index value (0-100)
     * @returns {Object} { class: string, labelKey: string, descriptionKey: string }
     */
    getIndexClass(index) {
        if (index <= 24) {
            return {
                class: 'fear',
                labelKey: 'extreme-fear',
                descriptionKey: 'extreme-fear-desc'
            };
        } else if (index <= 44) {
            return {
                class: 'fear',
                labelKey: 'fear',
                descriptionKey: 'fear-desc'
            };
        } else if (index <= 55) {
            return {
                class: 'neutral',
                labelKey: 'neutral',
                descriptionKey: 'neutral-desc'
            };
        } else if (index <= 74) {
            return {
                class: 'greed',
                labelKey: 'greed',
                descriptionKey: 'greed-desc'
            };
        } else {
            return {
                class: 'greed',
                labelKey: 'extreme-greed',
                descriptionKey: 'extreme-greed-desc'
            };
        }
    }
    
    /**
     * Update fear & greed index display
     * @param {number} value - Index value (0-100)
     */
    update(value) {
        if (!this.exists()) return;
        
        const index = parseInt(value) || 0;
        const indexInfo = this.getIndexClass(index);
        
        const html = `
            <div class="index-display flex items-center justify-between">
                <div class="index-value ${indexInfo.class}">${index}</div>
                <div class="text-right">
                    <div class="index-label" data-i18n="${indexInfo.labelKey}">Loading...</div>
                    <div class="index-description text-xs" data-i18n="${indexInfo.descriptionKey}">Loading...</div>
                </div>
            </div>
        `;
        
        this.setHTML(html);
        
        // Render gauge chart
        if (this.chartRenderer) {
            this.chartRenderer.renderGaugeChart('fear-greed', index);
        }
        
        debugLog('✅ Fear & Greed Index updated:', { index, class: indexInfo.class });
    }
}
