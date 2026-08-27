/**
 * api-service.js - Handles HTTP API calls for dashboard data
 */

import {
    WS_DEBUG,
    showErrorNotification,
    getTranslatedText
} from './utils.js';

import {
    updateDashboardFromData,
    displayFallbackData
} from './ui-updaters.js';

/**
 * Fetches the latest dashboard summary from the API.
 * @returns {Promise<void>}
 */
export async function fetchDashboardSummary() {
    if (WS_DEBUG) console.log('🌐 Fetching dashboard summary from API...');

    try {
        const response = await fetch('/api/crypto/dashboard-summary');
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();

        // Cache for language switching
        window.dashboardSummaryCache = data;

        updateDashboardFromData(data);
        if (WS_DEBUG) console.log('✅ Dashboard summary loaded from API');
    } catch (error) {
        console.error('❌ Failed to fetch dashboard summary:', error);

        // Use cached data if available
        if (window.dashboardSummaryCache) {
            console.log('📦 Using cached data as fallback');
            updateDashboardFromData(window.dashboardSummaryCache);
        } else {
            showErrorNotification(getTranslatedText('error-fetching-data') || 'Lỗi tải dữ liệu');
            displayFallbackData();
        }
    }
}

/**
 * Manually refreshes the dashboard data via WebSocket.
 * @returns {Promise<void>}
 */
export async function manualRefreshDashboard() {
    const refreshBtn = document.getElementById('refresh-dashboard');
    if (refreshBtn) {
        refreshBtn.disabled = true;
        const icon = refreshBtn.querySelector('i');
        if (icon) icon.classList.add('animate-spin');
    }

    if (window.dashboardWS && typeof window.dashboardWS.requestFreshData === 'function') {
        const sent = window.dashboardWS.requestFreshData();
        if (WS_DEBUG) console.log('🔄 Requested fresh dashboard data via WebSocket:', sent);
    }

    setTimeout(() => {
        if (refreshBtn) {
            refreshBtn.disabled = false;
            const icon = refreshBtn.querySelector('i');
            if (icon) icon.classList.remove('animate-spin');
        }
    }, 600);
}
