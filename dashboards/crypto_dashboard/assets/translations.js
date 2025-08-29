function get_translations_data()
{
    const translations_data = {
    // Homepage
    'homepage-title': { vi: 'Trang chủ - Crypto Dashboard', en: 'Homepage - Crypto Dashboard' },
    'welcome-message': { vi: 'Chào mừng đến Crypto Dashboard', en: 'Welcome to Crypto Dashboard' },
    'homepage-description': { vi: 'Theo dõi và phân tích thị trường tiền mã hóa', en: 'Track and analyze cryptocurrency markets' },
    'view-dashboard': { vi: 'Xem Bài phân tích thị trường Crypto mới nhất', en: 'View Latest Crypto Market Analysis' },
    
    // Dashboard card titles
    'btc-price': { vi: 'Giá BTC', en: 'BTC Price' },
    'market-cap': { vi: 'Tổng Vốn Hóa', en: 'Market Capitalization' },
    'volume-24h': { vi: 'Khối Lượng Giao Dịch 24h', en: '24h Trading Volume' },
    'fear-greed-title': { vi: 'Chỉ số Sợ hãi & Tham lam', en: 'Fear & Greed Index' },
    'rsi-btc-title': { vi: 'Chỉ số Sức mạnh Tương đối (RSI 14) - BTC', en: 'Relative Strength Index (RSI 14) - BTC' },
    'create-report': { vi: 'Tạo báo cáo Mới', en: 'Create New Report' },
    'view-report-history': { vi: 'Lịch Sử Báo Cáo', en: 'View Report History' },
    'home': { vi: 'Trang chủ', en: 'Home' },
    'view-report': { vi: 'Xem báo cáo mới nhất', en: 'View the latest report' },
    'print-report': { vi: 'In Báo cáo', en: 'Print Report' },
    'report-display': { vi: 'Hiển thị báo cáo', en: 'Displaying report' },
    'site-title': { vi: 'Toàn Cảnh Thị Trường Tiền MÃ Hóa', en: 'Crypto Market Overview' },
    'created-at': { vi: 'Tạo lúc', en: 'Created at' },
    'analysis-summary': { vi: 'Bài phân tích và tổng hợp', en: 'Analysis and summary' },
    'close': { vi: 'Đóng', en: 'Close' },
    'no-report-created': { vi: 'Chưa có báo cáo nào được tạo.', en: 'No reports have been created yet.' },
    'please-upload': { vi: 'Vui lòng sử dụng trang tải lên để tạo báo cáo đầu tiên của bạn.', en: 'Please use the upload page to create your first report.' },
    'whole-market': { vi: 'Toàn bộ thị trường', en: 'Whole market' },
    'loading': { vi: 'Đang tải...', en: 'Loading...' },
    'connection-issue': { vi: 'Lỗi kết nối', en: 'Connection issue' },
    
    // Report List Page
    'report-history-desc': { vi: 'Xem lại các báo cáo đã được tạo trước đây.', en: 'Review previously created reports.' },
    'created-date': { vi: 'Ngày Tạo', en: 'Created Date' },
    'actions': { vi: 'Hành Động', en: 'Actions' },
    'view-details': { vi: 'Xem Chi Tiết', en: 'View Details' },
    'no-reports': { vi: 'Chưa có báo cáo nào', en: 'No reports yet' },
    'create-first-report': { vi: 'Hãy tạo báo cáo đầu tiên của bạn!', en: 'Create your first report!' },
    'showing': { vi: 'Hiển thị', en: 'Showing' },
    'of-total': { vi: 'trong tổng số', en: 'of' },
    'reports': { vi: 'báo cáo', en: 'reports' },
    'total-reports': { vi: 'Tổng Báo Cáo', en: 'Total Reports' },
    'latest-report': { vi: 'Báo Cáo Mới Nhất', en: 'Latest Report' },
    'current-page': { vi: 'Trang Hiện Tại', en: 'Current Page' },
    
    // Disclaimer
    'disclaimer-title': { vi: 'Tuyên bố miễn trừ trách nhiệm:', en: 'Disclaimer:' },
    'disclaimer-body': { vi: 'Nội dung và phân tích trên trang này chỉ mang tính chất tham khảo và không cấu thành lời khuyên đầu tư. Mọi quyết định đầu tư là trách nhiệm của người đọc.', en: 'The content and analysis on this site are for informational purposes only and do not constitute investment advice. All investment decisions are the responsibility of the reader.' },
    
    // Navigation
    'report-table-of-contents': { vi: '📋 Mục lục Báo cáo', en: '📋 Report Table of Contents' },
    
    // Fear & Greed / RSI labels
    'extreme-fear': { vi: 'Sợ hãi Tột độ', en: 'Extreme Fear' },
    'fear': { vi: 'Sợ hãi', en: 'Fear' },
    'neutral': { vi: 'Trung lập', en: 'Neutral' },
    'greed': { vi: 'Tham lam', en: 'Greed' },
    'extreme-greed': { vi: 'Tham lam Tột độ', en: 'Extreme Greed' },
    'oversold': { vi: 'Quá bán', en: 'Oversold' },
    'overbought': { vi: 'Quá mua', en: 'Overbought' },
    'bitcoin': { vi: 'Bitcoin', en: 'Bitcoin' },
    'altcoins': { vi: 'Altcoins', en: 'Altcoins' },
    
    // Dashboard status and controls
    'refresh-data': { vi: 'Cập nhật dữ liệu', en: 'Refresh Data' },
    'refreshing': { vi: 'Đang cập nhật...', en: 'Refreshing...' },
    'connecting': { vi: 'Đang kết nối...', en: 'Connecting...' },
    'reconnecting': { vi: 'Đang kết nối lại...', en: 'Reconnecting...' },
    'real-time-connected': { vi: 'Kết nối thời gian thực', en: 'Real-time connected' },
    'connection-lost': { vi: 'Mất kết nối', en: 'Connection lost' },
    'connection-error': { vi: 'Lỗi kết nối', en: 'Connection error' },
    'data-updated': { vi: 'Dữ liệu đã được cập nhật', en: 'Data updated successfully' },
    'refresh-failed': { vi: 'Lỗi cập nhật dữ liệu', en: 'Failed to refresh data' },
    'last-update': { vi: 'Cập nhật lần cuối', en: 'Last updated' },
    'error-loading-data': { vi: 'Lỗi tải dữ liệu', en: 'Error loading data' },
    
    // Market Indicators Component
    'market-indicators-title': { vi: 'Chỉ Số Thị Trường', en: 'Market Indicators' },
    'live-data': { vi: 'Thời gian thực', en: 'Live data' },
    'fear-greed-index': { vi: 'Chỉ Số Sợ Hãi & Tham Lam của thị trường crypto', en: 'Fear & Greed Index of Crypto Market' },
    'btc-dominance': { vi: 'Độ Thống Trị BTC', en: 'BTC Dominance' },
    'btc-market-share': { vi: 'Thị phần BTC', en: 'BTC Market Share' },
    // Market indicators
    'market-indicators-title': { vi: 'Chỉ Số Thị Trường', en: 'Market Indicators' },
    'btc-dominance': { vi: 'Độ Thống Trị BTC', en: 'BTC Dominance' },
    'eth-dominance': { vi: 'Độ Thống Trị ETH', en: 'ETH Dominance' },
    'btc-market-share': { vi: 'Thị phần BTC', en: 'BTC Market Share' },
    'eth-market-share': { vi: 'Thị phần ETH', en: 'ETH Market Share' },
    'active-cryptos': { vi: 'Coin Hoạt Động', en: 'Active Coins' },
    'markets': { vi: 'Sàn Giao Dịch', en: 'Markets' },
    'market-cap-change': { vi: 'Thay Đổi Vốn Hóa', en: 'Market Cap Change' },
    'last-updated': { vi: 'Cập Nhật Lần Cuối', en: 'Last Updated' },
    
    // Fear & Greed Index classifications
    'extreme-fear': { vi: 'Sợ hãi tột độ', en: 'Extreme Fear' },
    'fear': { vi: 'Sợ hãi', en: 'Fear' },
    'neutral': { vi: 'Trung tính', en: 'Neutral' },
    'greed': { vi: 'Tham lam', en: 'Greed' },
    'extreme-greed': { vi: 'Tham lam tột độ', en: 'Extreme Greed' },
    
    // Fear & Greed Index descriptions
    'extreme-fear-desc': { vi: 'Thị trường đang trong trạng thái sợ hãi tột độ', en: 'Market is in extreme fear state' },
    'fear-desc': { vi: 'Thị trường có xu hướng giảm mạnh', en: 'Market tends to decline strongly' },
    'neutral-desc': { vi: 'Thị trường ổn định, không có xu hướng rõ ràng', en: 'Market is stable with no clear trend' },
    'greed-desc': { vi: 'Thị trường có xu hướng tăng mạnh', en: 'Market tends to rise strongly' },
    'extreme-greed-desc': { vi: 'Thị trường đang trong trạng thái tham lam tột độ', en: 'Market is in extreme greed state' },
    
    'powered-by': { vi: 'Được cung cấp bởi', en: 'Powered by' }
    };
    return translations_data;
}

// Auto-register translations when this script loads
(function() {
    // Function to try registering translations
    function tryRegisterTranslations() {
        try {
            const translationsData = get_translations_data();
            
            // Method 1: Use setTranslations if available
            if (typeof window.setTranslations === 'function') {
                window.setTranslations(translationsData);
                console.log('✅ Translations registered via setTranslations');
                return true;
            }
            
            // Method 2: Set global variables directly
            window.translations_data = translationsData;
            window.translations = translationsData;
            console.log('✅ Translations set globally');
            
            return false; // setTranslations not available yet
        } catch (e) {
            console.warn('Could not register translations:', e);
            return false;
        }
    }
    
    // Try immediately first
    if (tryRegisterTranslations()) {
        return; // Success, we're done
    }
    
    // If failed, wait for DOM ready and retry
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function() {
            setTimeout(tryRegisterTranslations, 100);
        });
    } else {
        // DOM already ready, retry after a short delay
        setTimeout(tryRegisterTranslations, 100);
    }
    
    // Also listen for a custom event in case language-toggle.js loads later
    window.addEventListener('languageToggleReady', function() {
        tryRegisterTranslations();
    });
})();

