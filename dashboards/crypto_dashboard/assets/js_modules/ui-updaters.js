/**
 * ui-updaters.js - Functions for updating dashboard UI elements from data
 */

import {
    WS_DEBUG,
    formatNumber,
    getTranslatedText,
    selectDashboardElementByLang,
    displayError,
    showBtcRefreshIndicator
} from './utils.js';

/**
 * Updates BTC price from WebSocket or API data.
 * @param {Object} btcData - BTC data object.
 */
export function updateBtcPriceFromWebSocket(btcData) {
    if (WS_DEBUG) console.log('🔄 Updating BTC price from WebSocket:', btcData);

    const btcContainer = selectDashboardElementByLang('btc-price-container');

    const priceValue = btcData.btc_price_usd || btcData.price_usd || 0;
    const changeValue = btcData.btc_change_24h || btcData.change_24h || 0;

    if (btcContainer && priceValue) {
        showBtcRefreshIndicator();

        const price = parseFloat(priceValue) || 0;
        const change = parseFloat(changeValue) || 0;
        const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
        const changeIcon = change >= 0 ? '📈' : '📉';

        const priceElement = btcContainer.querySelector('[data-btc-price]');
        const changeElement = btcContainer.querySelector('[data-btc-change]');

        if (priceElement && changeElement) {
            priceElement.textContent = `$${price.toLocaleString('en-US')}`;
            changeElement.textContent = `${changeIcon} ${change.toFixed(2)}% (24h)`;

            const newClassName = `text-sm font-semibold ${changeClass}`;
            if (changeElement.className !== newClassName) {
                changeElement.className = newClassName;
            }
        } else {
            btcContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900" data-btc-price>$${price.toLocaleString('en-US')}</p>
                <p class="text-sm font-semibold ${changeClass}" data-btc-change>${changeIcon} ${change.toFixed(2)}% (24h)</p>`;
        }

        try {
            btcContainer.dataset.btcPriceUsd = String(price);
            btcContainer.dataset.btcChange24h = String(change);
        } catch (e) { }
    }
}

/**
 * Updates market data (cap, volume, F&G) from WebSocket or API data.
 * @param {Object} marketData - Market data object.
 */
export function updateMarketDataFromWebSocket(marketData) {
    if (WS_DEBUG) console.log('🔄 Updating market data from WebSocket:', marketData);

    if (marketData.market_cap_usd) {
        const marketCapContainer = selectDashboardElementByLang('market-cap-container');
        if (marketCapContainer) {
            const marketCap = parseFloat(marketData.market_cap_usd) || 0;
            marketCapContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">$${(marketCap / 1e12).toFixed(2)}T</p>
                <p class="text-sm text-gray-600">Market Cap</p>`;
        }
    }

    if (marketData.volume_24h_usd) {
        const volumeContainer = selectDashboardElementByLang('volume-container');
        if (volumeContainer) {
            const volume = parseFloat(marketData.volume_24h_usd) || 0;
            volumeContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">$${(volume / 1e9).toFixed(1)}B</p>
                <p class="text-sm text-gray-600">24h Volume</p>`;
        }
    }

    if (marketData.fng_value) {
        const fngContainer = selectDashboardElementByLang('fear-greed-container');
        const fngValue = parseInt(marketData.fng_value, 10);
        if (!isNaN(fngValue) && fngContainer && typeof window.createGauge === 'function') {
            const fngConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                    { limit: 45, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                    { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                    { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                    { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
                ]
            };
            window.createGauge(fngContainer, fngValue, fngConfig);
        }
    }
}

/**
 * Updates the dashboard from a data object.
 * @param {Object} data - Full dashboard summary data.
 */
export function updateDashboardFromData(data) {
    if (WS_DEBUG) console.log('🔍 [DEBUG] updateDashboardFromData received:', data);

    // 1. Market Cap
    const marketCapContainer = selectDashboardElementByLang('market-cap-container');
    if (marketCapContainer) {
        const marketCapValue = parseFloat(data.market_cap_usd || data.market_cap) || 0;
        const marketCapChange = parseFloat(data.market_cap_change_percentage_24h_usd) || 0;

        const changeClass = marketCapChange >= 0 ? 'text-green-600' : 'text-red-600';
        const changeSign = marketCapChange >= 0 ? '+' : '';
        const changeIcon = marketCapChange >= 0 ? '📈' : '📉';

        marketCapContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(marketCapValue)}</p>
            <p class="text-sm ${changeClass}">${changeIcon} ${changeSign}${marketCapChange.toFixed(2)}% (24h)</p>
            <p class="text-xs text-gray-600">Market Cap</p>`;
        try { marketCapContainer.dataset.marketCap = String(marketCapValue); } catch (e) { }
    }

    // 2. Volume
    const volumeContainer = selectDashboardElementByLang('volume-24h-container');
    if (volumeContainer) {
        const volumeValue = parseFloat(data.volume_24h_usd || data.volume_24h) || 0;
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(volumeValue)}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
        try { volumeContainer.dataset.volume24h = String(volumeValue); } catch (e) { }
    }

    // 3. BTC Price
    const btcContainer = selectDashboardElementByLang('btc-price-container');
    if (btcContainer) {
        showBtcRefreshIndicator();
        const btcPrice = parseFloat(data.btc_price_usd) || 0;
        const change = parseFloat(data.btc_change_24h) || 0;
        const safeChange = (change !== undefined && change !== null) ? change : 0;
        const changeClass = safeChange >= 0 ? 'text-green-600' : 'text-red-600';
        const changeIcon = safeChange >= 0 ? '📈' : '📉';

        const priceElement = btcContainer.querySelector('[data-btc-price]');
        const changeElement = btcContainer.querySelector('[data-btc-change]');

        if (priceElement && changeElement) {
            priceElement.textContent = btcPrice > 0 ? '$' + btcPrice.toLocaleString('en-US') : '$N/A';
            changeElement.textContent = `${changeIcon} ${safeChange.toFixed(2)}% (24h)`;

            const newClassName = `text-sm font-semibold ${changeClass}`;
            if (changeElement.className !== newClassName) {
                changeElement.className = newClassName;
            }
        } else {
            btcContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900" data-btc-price>${btcPrice > 0 ? '$' + btcPrice.toLocaleString('en-US') : '$N/A'}</p>
                <p class="text-sm font-semibold ${changeClass}" data-btc-change>${changeIcon} ${safeChange.toFixed(2)}% (24h)</p>`;
        }
        try { btcContainer.dataset.btcPriceUsd = String(btcPrice); btcContainer.dataset.btcChange24h = String(change); } catch (e) { }
    }

    // 4. Fear & Greed
    const fngContainer = selectDashboardElementByLang('fear-greed-container');
    const fngValue = parseInt(data.fng_value, 10);
    if (!isNaN(fngValue) && fngContainer && typeof window.createGauge === 'function') {
        const fngConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                { limit: 45, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
            ]
        };
        window.createGauge(fngContainer, fngValue, fngConfig);
        try { fngContainer.dataset.value = String(fngValue); } catch (e) { }
    }

    // 5. RSI
    const rsiContainer = selectDashboardElementByLang('rsi-container');
    const rsiValue = parseFloat(data.btc_rsi_14);
    if (rsiValue !== null && !isNaN(rsiValue) && rsiContainer && typeof window.createGauge === 'function') {
        const rsiConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
            ]
        };
        window.createGauge(rsiContainer, rsiValue, rsiConfig);
        try { rsiContainer.dataset.value = String(rsiValue); } catch (e) { }
    }

    updateLastUpdatedTime();
}

/**
 * Updates the last successful update timestamp display.
 */
export function updateLastUpdatedTime() {
    const timeElement = document.getElementById('last-update-time');
    if (timeElement) {
        const now = new Date();
        const lang = (window.languageManager && window.languageManager.currentLanguage) || 'vi';

        try {
            const timeOptions = {
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
                timeZone: 'Asia/Ho_Chi_Minh',
                hour12: false
            };
            const timeFormatter = new Intl.DateTimeFormat((lang === 'en') ? 'en-US' : 'vi-VN', timeOptions);
            timeElement.textContent = timeFormatter.format(now) + ' (GMT+7)';
        } catch (e) {
            timeElement.textContent = now.toLocaleTimeString();
        }
    }
}

/**
 * Renders the dashboard from cached data (useful for language changes).
 * @param {string} lang - Language code.
 */
export function renderDashboardFromCache(lang) {
    const data = window.dashboardSummaryCache;
    if (!data) return;

    const marketCapContainer = selectDashboardElementByLang('market-cap-container', lang);
    if (marketCapContainer) {
        marketCapContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(Number(marketCapContainer.dataset.marketCap || data.market_cap))}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    const volumeContainer = selectDashboardElementByLang('volume-24h-container', lang);
    if (volumeContainer) {
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(Number(volumeContainer.dataset.volume24h || data.volume_24h))}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    const btcContainer = selectDashboardElementByLang('btc-price-container', lang);
    if (btcContainer) {
        const price = btcContainer.dataset.btcPriceUsd || data.btc_price_usd;
        const change = Number(btcContainer.dataset.btcChange24h || data.btc_change_24h || 0);
        const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
        btcContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + (price ? Number(price).toLocaleString('en-US') : 'N/A')}</p>
            <p class="text-sm font-semibold ${changeClass}">${!isNaN(change) ? change.toFixed(2) : 'N/A'}% (24h)</p>`;
    }

    if (typeof window.createGauge === 'function') {
        const fngContainer = selectDashboardElementByLang('fear-greed-container', lang);
        const fngVal = fngContainer ? Number(fngContainer.dataset.value || data.fng_value) : null;
        if (fngContainer && !isNaN(fngVal)) {
            const fngConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                    { limit: 49, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                    { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                    { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                    { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
                ]
            };
            window.createGauge(fngContainer, fngVal, fngConfig);
        }

        const rsiContainer = selectDashboardElementByLang('rsi-container', lang);
        const rsiVal = rsiContainer ? Number(rsiContainer.dataset.value || data.btc_rsi_14) : null;
        if (rsiContainer && rsiVal !== null && !isNaN(rsiVal)) {
            const rsiConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                    { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                    { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
                ]
            };
            window.createGauge(rsiContainer, rsiVal, rsiConfig);
        }
    }
}

/**
 * Displays fallback loading states for dashboard cards.
 */
export function displayFallbackData() {
    const marketCapContainer = document.getElementById('market-cap-container');
    if (marketCapContainer) {
        marketCapContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    const volumeContainer = document.getElementById('volume-24h-container');
    if (volumeContainer) {
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    const btcContainer = document.getElementById('btc-price-container');
    if (btcContainer) {
        btcContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">Bitcoin</p>`;
    }

    if (typeof window.createGauge === 'function') {
        const fngContainer = document.getElementById('fear-greed-container');
        if (fngContainer) {
            const fngConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                    { limit: 49, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                    { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                    { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                    { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
                ]
            };
            window.createGauge(fngContainer, 50, fngConfig);
        }

        const rsiContainer = document.getElementById('rsi-container');
        if (rsiContainer) {
            const rsiConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                    { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                    { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
                ]
            };
            window.createGauge(rsiContainer, 50, rsiConfig);
        }
    }
}
