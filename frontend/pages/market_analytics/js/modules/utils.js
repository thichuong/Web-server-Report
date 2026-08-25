/**
 * Utility formatters and helpers for Market Analytics
 */

// Helper to format currency and numbers
const formatNumber = (num, decimals = 2) => {
    if (num === null || num === undefined || isNaN(num)) return '--';
    if (Math.abs(num) >= 1e9) return (num / 1e9).toFixed(decimals) + 'B';
    if (Math.abs(num) >= 1e6) return (num / 1e6).toFixed(decimals) + 'M';
    if (Math.abs(num) >= 1e3) return (num / 1e3).toFixed(decimals) + 'K';
    return Number(num).toLocaleString('en-US', { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
};

const formatPrice = (price) => {
    if (price === null || price === undefined || isNaN(price)) return '--';
    const num = Number(price);
    if (num >= 1000) return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    if (num >= 1) return num.toFixed(4);
    return num.toFixed(6);
};

const formatDate = (timestamp, timeframe = '1h') => {
    const d = new Date(timestamp);
    if (timeframe === '1d') {
        return `${d.getMonth() + 1}/${d.getDate()}`;
    }
    return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')} (${d.getDate()}/${d.getMonth() + 1})`;
};

const getI18nText = (key, defaultText) => {
    const lang = (document.documentElement.lang || 'vi') === 'en' ? 'en' : 'vi';
    const dict = window.translations_data || window.translations;
    if (dict && dict[key] && dict[key][lang]) {
        return dict[key][lang];
    }
    return defaultText;
};


export { formatNumber, formatPrice, formatDate, getI18nText };
