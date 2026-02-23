/**
 * utils.js - General utility functions and UI helpers for the dashboard
 */

export const WS_DEBUG = true;

/**
 * Formats a large number into a human-readable string (trillions, billions, millions).
 * @param {number} num - The number to format.
 * @returns {string} - Formatted string.
 */
export function formatNumber(num) {
    if (window.languageManager && window.languageManager.formatNumberLocalized) {
        return window.languageManager.formatNumberLocalized(num);
    }

    if (num === null || num === undefined) return 'N/A';
    if (num >= 1e12) return (num / 1e12).toFixed(2) + ' nghìn tỷ';
    if (num >= 1e9) return (num / 1e9).toFixed(2) + ' tỷ';
    if (num >= 1e6) return (num / 1e6).toFixed(2) + ' triệu';
    return num.toLocaleString('en-US');
}

/**
 * Gets translated text for a given key.
 * @param {string} key - The translation key.
 * @returns {string} - Translated text or key as fallback.
 */
export function getTranslatedText(key) {
    if (window.languageManager && window.languageManager.getTranslatedText) {
        return window.languageManager.getTranslatedText(key);
    }
    return key;
}

/**
 * Selects an element for the dashboard by language-aware id.
 * @param {string} idBase - Base ID.
 * @param {string} lang - Optional language code.
 * @returns {HTMLElement|null}
 */
export function selectDashboardElementByLang(idBase, lang) {
    const language = lang || (window.languageManager && window.languageManager.currentLanguage) || 'vi';
    if (language === 'en') {
        const enEl = document.getElementById(idBase + '-en');
        if (enEl) return enEl;
    }
    return document.getElementById(idBase);
}

/**
 * Displays a friendly error message in a specific container.
 * @param {string} containerId - ID of the container.
 * @param {string} message - Error message.
 */
export function displayError(containerId, message) {
    const container = document.getElementById(containerId);
    if (container) {
        const errorMsg = message || getTranslatedText('error-loading-data');
        container.innerHTML = `<p class="text-sm text-red-600">${errorMsg}</p>`;
    }
}

/**
 * Displays a toast notification for errors.
 * @param {string} message - Error message.
 */
export function showErrorNotification(message) {
    let notification = document.getElementById('error-notification');
    if (!notification) {
        notification = document.createElement('div');
        notification.id = 'error-notification';
        notification.className = 'fixed top-4 right-4 bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded shadow-lg z-50 max-w-sm';
        document.body.appendChild(notification);
    }

    notification.innerHTML = `
        <div class="flex items-center">
            <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"></path>
            </svg>
            <span class="text-sm">${message}</span>
        </div>
    `;

    setTimeout(() => {
        if (notification && notification.parentNode) {
            notification.parentNode.removeChild(notification);
        }
    }, 5000);
}

/**
 * Updates the WebSocket status indicator.
 * @param {string} status - Current status ('connecting', 'connected', etc.).
 * @param {string} message - Message to display.
 */
export function updateWebSocketStatus(status, message) {
    const statusElement = document.getElementById('websocket-status');
    if (!statusElement) return;

    const statusClasses = {
        'connecting': 'bg-yellow-100 text-yellow-800',
        'connected': 'bg-green-100 text-green-800',
        'disconnected': 'bg-red-100 text-red-800',
        'error': 'bg-red-100 text-red-800'
    };

    const statusIcons = {
        'connecting': 'fas fa-sync-alt animate-spin text-yellow-600',
        'connected': 'fas fa-check-circle text-green-600',
        'disconnected': 'fas fa-times-circle text-red-600',
        'error': 'fas fa-exclamation-triangle text-red-600'
    };

    const statusIcon = statusIcons[status] || 'fas fa-circle text-gray-400';
    const statusClass = statusClasses[status] || 'bg-gray-100 text-gray-800';

    statusElement.className = `inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${statusClass}`;
    statusElement.innerHTML = `
        <div class="w-2 h-2 mr-2">
            <i class="${statusIcon}"></i>
        </div>
        <span>${message}</span>
    `;
}

/**
 * Briefly shows the BTC refresh indicator.
 */
export function showBtcRefreshIndicator() {
    const indicator = document.getElementById('btc-refresh-indicator');
    if (indicator) {
        indicator.style.opacity = '1';
        setTimeout(() => {
            indicator.style.opacity = '0';
        }, 1000);
    }
}
