/**
 * MarketCapUpdater - Updates Market Cap indicator
 */

import { BaseUpdater } from './BaseUpdater.js';

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class MarketCapUpdater extends BaseUpdater {
    constructor() {
        super('market-cap-indicator');
    }
    
    /**
     * Update market cap display
     * @param {Object} data - { value: number, change: number }
     */
    update(data) {
        if (!this.exists()) return;
        
        const value = parseFloat(data.value || data.total_market_cap || data.market_cap_usd) || 0;
        const change = parseFloat(data.change || data.market_cap_change_percentage_24h_usd) || 0;
        
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
        debugLog('✅ Market cap updated:', { value, change });
    }
}
