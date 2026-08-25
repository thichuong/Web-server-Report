/**
 * Market Analytics Client-side Engine Entry Point
 */

import { MarketAnalyticsApp } from './modules/app.js';

// Auto-bootstrap on DOM ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.marketAnalyticsApp = new MarketAnalyticsApp();
    });
} else {
    window.marketAnalyticsApp = new MarketAnalyticsApp();
}

export { MarketAnalyticsApp };
