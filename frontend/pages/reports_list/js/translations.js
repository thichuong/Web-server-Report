function get_translations_data()
{
    const translations_data = {
    // Navigation & Page Header
    'home': { vi: 'Trang chủ', en: 'Home' },
    'analytics-nav-btn': { vi: 'Phân Tích Volume & OI', en: 'Volume & OI Analytics' },
    'view-report-history': { vi: 'Lịch Sử Báo Cáo', en: 'View Report History' },
    'report-history-desc': { vi: 'Xem lại các báo cáo đã được tạo trước đây.', en: 'Review previously created reports.' },

    // Table Headers & Actions
    'created-date': { vi: 'Ngày Tạo', en: 'Created Date' },
    'actions': { vi: 'Hành Động', en: 'Actions' },
    'view-details': { vi: 'Xem Chi Tiết', en: 'View Details' },
    'no-reports': { vi: 'Chưa có báo cáo nào', en: 'No reports yet' },
    'create-first-report': { vi: 'Hãy tạo báo cáo đầu tiên của bạn!', en: 'Create your first report!' },
    'create-report': { vi: 'Tạo Báo Cáo Mới', en: 'Create New Report' },

    // Pagination & Metric Cards
    'showing': { vi: 'Hiển thị', en: 'Showing' },
    'of-total': { vi: 'trong tổng số', en: 'of' },
    'reports': { vi: 'báo cáo', en: 'reports' },
    'total-reports': { vi: 'Tổng Báo Cáo', en: 'Total Reports' },
    'latest-report': { vi: 'Báo Cáo Mới Nhất', en: 'Latest Report' },
    'current-page': { vi: 'Trang Hiện Tại', en: 'Current Page' }
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
                console.log('✅ Reports List Translations registered via setTranslations');
                return true;
            }
            
            // Method 2: Set global variables directly
            window.translations_data = translationsData;
            window.translations = translationsData;
            console.log('✅ Reports List Translations set globally');
            
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
