function get_translations_data()
{
    const translations_data = {
    // Homepage Navigation & Hero
    'homepage-title': { vi: 'Trang chủ - Crypto Dashboard', en: 'Homepage - Crypto Dashboard' },
    'welcome-message': { vi: 'Chào mừng đến Crypto Dashboard', en: 'Welcome to Crypto Dashboard' },
    'homepage-description': { vi: 'Theo dõi và phân tích thị trường tiền mã hóa với các công cụ phân tích chuyên nghiệp và dữ liệu thời gian thực', en: 'Track and analyze cryptocurrency markets with professional analysis tools and real-time data' },
    'view-dashboard': { vi: 'Xem Bài phân tích thị trường Crypto mới nhất', en: 'View Latest Crypto Market Analysis' },
    'analytics-nav-btn': { vi: 'Phân Tích Volume & OI', en: 'Volume & OI Analytics' },
    'home': { vi: 'Trang chủ', en: 'Home' },
    'view-report-history': { vi: 'Lịch Sử Báo Cáo', en: 'View Report History' },
    'create-report': { vi: 'Tạo báo cáo Mới', en: 'Create New Report' },

    // Market Indicators Component
    'market-indicators-title': { vi: 'Chỉ Số Thị Trường', en: 'Market Indicators' },
    'crypto-market-stats': { vi: 'Thống Kê Thị Trường Crypto', en: 'Crypto Market Statistics' },
    'live-data': { vi: 'Thời gian thực', en: 'Live data' },
    'market-cap': { vi: 'Tổng Vốn Hóa', en: 'Market Capitalization' },
    'volume-24h': { vi: 'Khối Lượng Giao Dịch 24h', en: '24h Trading Volume' },
    'fear-greed-index': { vi: 'Chỉ Số Sợ Hãi & Tham Lam của thị trường crypto', en: 'Fear & Greed Index of Crypto Market' },
    'fear-greed-title': { vi: 'Chỉ số Sợ hãi & Tham lam', en: 'Fear & Greed Index' },
    'btc-rsi-14': { vi: 'BTC RSI 14', en: 'BTC RSI 14' },
    'rsi-btc-title': { vi: 'Chỉ số Sức mạnh Tương đối (RSI 14) - BTC', en: 'Relative Strength Index (RSI 14) - BTC' },
    'btc-dominance': { vi: 'Độ Thống Trị BTC', en: 'BTC Dominance' },
    'eth-dominance': { vi: 'Độ Thống Trị ETH', en: 'ETH Dominance' },
    'btc-market-share': { vi: 'Thị phần BTC', en: 'BTC Market Share' },
    'eth-market-share': { vi: 'Thị phần ETH', en: 'ETH Market Share' },
    'active-cryptos': { vi: 'Coin Hoạt Động', en: 'Active Coins' },
    'markets': { vi: 'Sàn Giao Dịch', en: 'Markets' },
    'market-cap-change': { vi: 'Thay Đổi Vốn Hóa', en: 'Market Cap Change' },
    'last-updated': { vi: 'Cập Nhật Lần Cuối', en: 'Last Updated' },
    'last-update': { vi: 'Cập nhật lần cuối', en: 'Last updated' },
    'binance-prices-title': { vi: 'Giá Crypto từ Binance', en: 'Crypto Prices from Binance' },
    'powered-by': { vi: 'Được cung cấp bởi', en: 'Powered by' },
    'websocket-api': { vi: 'API WebSocket', en: 'WebSocket API' },

    // Fear & Greed / RSI Levels & Descriptions
    'extreme-fear': { vi: 'Sợ hãi tột độ', en: 'Extreme Fear' },
    'fear': { vi: 'Sợ hãi', en: 'Fear' },
    'neutral': { vi: 'Trung tính', en: 'Neutral' },
    'greed': { vi: 'Tham lam', en: 'Greed' },
    'extreme-greed': { vi: 'Tham lam tột độ', en: 'Extreme Greed' },
    'extreme-fear-desc': { vi: 'Thị trường đang trong trạng thái sợ hãi tột độ', en: 'Market is in extreme fear state' },
    'fear-desc': { vi: 'Thị trường có xu hướng giảm mạnh', en: 'Market tends to decline strongly' },
    'neutral-desc': { vi: 'Thị trường ổn định, không có xu hướng rõ ràng', en: 'Market is stable with no clear trend' },
    'greed-desc': { vi: 'Thị trường có xu hướng tăng mạnh', en: 'Market tends to rise strongly' },
    'extreme-greed-desc': { vi: 'Thị trường đang trong trạng thái tham lam tột độ', en: 'Market is in extreme greed state' },
    'oversold': { vi: 'Quá bán', en: 'Oversold' },
    'overbought': { vi: 'Quá mua', en: 'Overbought' },

    // Connection & Real-time Status
    'connecting': { vi: 'Đang kết nối...', en: 'Connecting...' },
    'reconnecting': { vi: 'Đang kết nối lại...', en: 'Reconnecting...' },
    'real-time-connected': { vi: 'Kết nối thời gian thực', en: 'Real-time connected' },
    'connection-lost': { vi: 'Mất kết nối', en: 'Connection lost' },
    'connection-error': { vi: 'Lỗi kết nối', en: 'Connection error' },
    'connection-issue': { vi: 'Lỗi kết nối', en: 'Connection issue' },
    'connection-connecting': { vi: 'Đang kết nối...', en: 'Connecting...' },
    'connection-connected': { vi: 'Đã kết nối', en: 'Connected' },
    'connection-disconnected': { vi: 'Mất kết nối', en: 'Disconnected' },
    'connection-offline': { vi: 'Ngoại tuyến', en: 'Offline' },
    'data-updated': { vi: 'Dữ liệu đã được cập nhật', en: 'Data updated successfully' },
    'refresh-data': { vi: 'Cập nhật dữ liệu', en: 'Refresh Data' },
    'refreshing': { vi: 'Đang cập nhật...', en: 'Refreshing...' },
    'refresh-failed': { vi: 'Lỗi cập nhật dữ liệu', en: 'Failed to refresh data' },
    'error-loading-data': { vi: 'Lỗi tải dữ liệu', en: 'Error loading data' },
    'loading': { vi: 'Đang tải...', en: 'Loading...' },

    // Unit translations for large numbers
    'unit-trillion': { vi: ' Nghìn Tỷ', en: ' T' },
    'unit-billion': { vi: ' Tỷ', en: ' B' },
    'unit-million': { vi: ' Triệu', en: ' M' },
    'unit-trillion-en': { vi: ' Nghìn Tỷ', en: ' T' },
    'unit-billion-en': { vi: ' Tỷ', en: ' B' },
    'unit-million-en': { vi: ' Triệu', en: ' M' }
    };
    return translations_data;
}

// Auto-register translations when this script loads
(function() {
    function tryRegisterTranslations() {
        try {
            const translationsData = get_translations_data();
            
            // Method 1: Use setTranslations if available
            if (typeof window.setTranslations === 'function') {
                window.setTranslations(translationsData);
                console.log('✅ Home Translations registered via setTranslations');
                return true;
            }
            
            // Method 2: Set global variables directly
            window.translations_data = translationsData;
            window.translations = translationsData;
            console.log('✅ Home Translations set globally');
            
            return false;
        } catch (e) {
            console.warn('Could not register translations:', e);
            return false;
        }
    }
    
    // Try immediately first
    if (tryRegisterTranslations()) {
        return;
    }
    
    // If failed, wait for DOM ready and retry
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function() {
            setTimeout(tryRegisterTranslations, 100);
        });
    } else {
        setTimeout(tryRegisterTranslations, 100);
    }
    
    // Also listen for a custom event in case language-toggle.js loads later
    window.addEventListener('languageToggleReady', function() {
        tryRegisterTranslations();
    });
})();
