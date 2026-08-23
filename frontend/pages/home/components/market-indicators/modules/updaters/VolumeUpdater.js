/**
 * VolumeUpdater - Updates Volume 24h indicator
 */

import { BaseUpdater } from './BaseUpdater.js';

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class VolumeUpdater extends BaseUpdater {
    constructor() {
        super('volume-24h-indicator');
    }
    
    /**
     * Update volume 24h display
     * @param {Object} data - { value: number, change: number }
     */
    update(data) {
        if (!this.exists()) return;
        
        const value = parseFloat(data.value || data.total_volume_24h) || 0;
        const change = parseFloat(data.change || data.volume_change_24h) || 0;
        
        const changeInfo = this.getChangeInfo(change);
        const formatted = this.formatLargeNumber(value);
        const unitHtml = formatted.unitKey ? 
            `<span class="unit" data-i18n="${formatted.unitKey}">${formatted.unitText}</span>` : '';
        
        const html = `
            <div class="flex items-center justify-between">
                <div class="market-value">$${formatted.number}${unitHtml}</div>
                <div class="market-change ${changeInfo.class}">
                    <span class="change-icon">${changeInfo.icon}</span>
                    ${changeInfo.sign}${change.toFixed(2)}% (24h)
                </div>
            </div>
        `;
        
        this.setHTML(html);
        debugLog('✅ Volume 24h updated:', { value, change });
    }
}
