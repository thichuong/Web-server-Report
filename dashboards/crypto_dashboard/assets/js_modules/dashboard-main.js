/**
 * dashboard-main.js - Main entry point for the dashboard ES Modules
 */

import { WS_DEBUG } from './utils.js';
import { fetchDashboardSummary, manualRefreshDashboard } from './api-service.js';
import { renderDashboardFromCache } from './ui-updaters.js';
import { DashboardWebSocket } from './websocket-manager.js';

// Global instances for browser access if needed (backward compatibility)
let dashboardWS = null;

/**
 * Initializes the dashboard application.
 */
function initDashboard() {
    console.log('🚀 Initializing Dashboard (ESM)...');

    // 1. Fetch initial data via HTTP
    fetchDashboardSummary();

    // 2. Initialize WebSocket connection
    dashboardWS = new DashboardWebSocket();
    window.dashboardWS = dashboardWS; // Expose for debugging
    dashboardWS.connect();

    // 3. Register global event listeners
    setupEventListeners();

    console.log('✅ Dashboard initialization complete');
}

/**
 * Sets up global event listeners for theme, language, etc.
 */
function setupEventListeners() {
    // Language change listener
    window.addEventListener('languageChanged', (event) => {
        const newLang = event.detail.language;
        if (WS_DEBUG) console.log(`🌐 Language changed to: ${newLang}, re-rendering dashboard...`);

        // Re-render visuals using cached data
        renderDashboardFromCache(newLang);

        // If we want to force refresh from server on language change:
        // fetchDashboardSummary();
    });

    // Theme change listener (for potential chart adjustments)
    window.addEventListener('themeChanged', (event) => {
        const newTheme = event.detail.theme;
        if (WS_DEBUG) console.log(`🎨 Theme changed to: ${newTheme}`);

        // Re-render gauges to update colors if needed
        if (window.dashboardSummaryCache) {
            renderDashboardFromCache();
        }
    });

    // Manual refresh button
    const refreshBtn = document.getElementById('refresh-dashboard');
    if (refreshBtn) {
        refreshBtn.addEventListener('click', manualRefreshDashboard);
    }
}

// Start initialization when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initDashboard);
} else {
    initDashboard();
}

// Export for potential use in other scripts
export { dashboardWS, initDashboard };
