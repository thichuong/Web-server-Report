// language-toggle.js
// Simple language toggle between Vietnamese (vi) and English (en)
(function(){
    const LANG_KEY = 'preferred_language';
    const BACKUP_KEY = 'language';
    const DEFAULT_LANG = 'vi';

    // Minimal translation map for elements used in dashboard/report
    const translations = {
        'create-report': { vi: 'Tạo Báo cáo Mới', en: 'Create New Report' },
        'view-report-history': { vi: 'Xem Lịch Sử Báo Cáo', en: 'View Report History' },
        'print-report': { vi: 'In Báo cáo', en: 'Print Report' },
    'report-display': { vi: 'Hiển thị báo cáo', en: 'Displaying report' },
    'site-title': { vi: 'Toàn Cảnh Thị Trường Tiền Mã Hóa', en: 'Crypto Market Overview' },
        'created-at': { vi: 'tạo lúc', en: 'created at' },
        'analysis-summary': { vi: 'Bài phân tích và tổng hợp', en: 'Analysis and summary' },
        'close': { vi: 'Đóng', en: 'Close' },
        'no-report-created': { vi: 'Chưa có báo cáo nào được tạo.', en: 'No reports have been created yet.' },
        'please-upload': { vi: 'Vui lòng sử dụng trang tải lên để tạo báo cáo đầu tiên của bạn.', en: 'Please use the upload page to create your first report.' },
        'whole-market': { vi: 'Toàn bộ thị trường', en: 'Whole market' },
        'loading': { vi: 'Đang tải...', en: 'Loading...' },
        'connection-issue': { vi: 'Lỗi kết nối', en: 'Connection issue' },
    // Disclaimer
    'disclaimer-title': { vi: 'Tuyên bố miễn trừ trách nhiệm:', en: 'Disclaimer:' },
    'disclaimer-body': { vi: 'Nội dung và phân tích trên trang này chỉ mang tính chất tham khảo và không cấu thành lời khuyên đầu tư. Mọi quyết định đầu tư là trách nhiệm của người đọc.', en: 'The content and analysis on this site are for informational purposes only and do not constitute investment advice. All investment decisions are the responsibility of the reader.' },
        // Fear & Greed / RSI labels
        'extreme-fear': { vi: 'Sợ hãi Tột độ', en: 'Extreme Fear' },
        'fear': { vi: 'Sợ hãi', en: 'Fear' },
        'neutral': { vi: 'Trung lập', en: 'Neutral' },
        'greed': { vi: 'Tham lam', en: 'Greed' },
        'extreme-greed': { vi: 'Tham lam Tột độ', en: 'Extreme Greed' },
        'oversold': { vi: 'Quá bán', en: 'Oversold' },
    'overbought': { vi: 'Quá mua', en: 'Overbought' },
    'bitcoin': { vi: 'Bitcoin', en: 'Bitcoin' },
    'altcoins': { vi: 'Altcoins', en: 'Altcoins' }
    };

    function getPreferredLanguage(){
        try { return localStorage.getItem(LANG_KEY) || DEFAULT_LANG; } catch(e){ return DEFAULT_LANG; }
    }

    function setPreferredLanguage(lang){
        try {
            localStorage.setItem(LANG_KEY, lang);
            localStorage.setItem(BACKUP_KEY, lang);
        } catch(e){}
        updateUI(lang);
        // Notify other scripts that language changed
        try {
            const ev = new CustomEvent('languageChanged', { detail: { language: lang } });
            window.dispatchEvent(ev);
        } catch(e) {
            // ignore
        }
    }

    function updateUI(lang){
        // update html lang attribute
        document.documentElement.lang = (lang === 'en') ? 'en' : 'vi';

        // update button text
        const btnText = document.querySelector('#language-toggle .lang-text');
        if (btnText) btnText.textContent = (lang === 'en') ? 'EN' : 'VI';

        // swap report content if both versions exist
        const viContainer = document.getElementById('report-content-vi');
        const enContainer = document.getElementById('report-content-en');
        if (viContainer) viContainer.style.display = (lang === 'vi') ? 'block' : 'none';
        if (enContainer) enContainer.style.display = (lang === 'en') ? 'block' : 'none';

        // swap title and subtitle elements for PDF template
        const titleVi = document.getElementById('title-vi');
        const titleEn = document.getElementById('title-en');
        const subtitleVi = document.getElementById('subtitle-vi');
        const subtitleEn = document.getElementById('subtitle-en');
        const descriptionVi = document.getElementById('description-vi');
        const descriptionEn = document.getElementById('description-en');
        
        if (titleVi) titleVi.style.display = (lang === 'vi') ? 'block' : 'none';
        if (titleEn) titleEn.style.display = (lang === 'en') ? 'block' : 'none';
        if (subtitleVi) subtitleVi.style.display = (lang === 'vi') ? 'block' : 'none';
        if (subtitleEn) subtitleEn.style.display = (lang === 'en') ? 'block' : 'none';
        if (descriptionVi) descriptionVi.style.display = (lang === 'vi') ? 'block' : 'none';
        if (descriptionEn) descriptionEn.style.display = (lang === 'en') ? 'block' : 'none';

        // translate elements that use data-i18n
        document.querySelectorAll('[data-i18n]').forEach(el => {
            const key = el.getAttribute('data-i18n');
            if (!key) return;
            const map = translations[key];
            if (map && map[lang]) {
                el.textContent = map[lang];
            }
        });
        // expose a minimal languageManager for other scripts
        try {
            window.languageManager = window.languageManager || {};
            window.languageManager.currentLanguage = lang;
            window.languageManager.getTranslatedText = function(key) {
                const m = translations[key];
                if (m && m[lang]) return m[lang];
                return key;
            };
            window.languageManager.formatNumberLocalized = function(num) {
                if (num === null || num === undefined) return 'N/A';
                // simple large-number formatting with locale
                if (lang === 'vi') {
                    if (num >= 1e12) return (num / 1e12).toFixed(2) + ' nghìn tỷ';
                    if (num >= 1e9) return (num / 1e9).toFixed(2) + ' tỷ';
                    if (num >= 1e6) return (num / 1e6).toFixed(2) + ' triệu';
                    return new Intl.NumberFormat('vi-VN').format(num);
                } else {
                    if (num >= 1e12) return (num / 1e12).toFixed(2) + 'T';
                    if (num >= 1e9) return (num / 1e9).toFixed(2) + 'B';
                    if (num >= 1e6) return (num / 1e6).toFixed(2) + 'M';
                    return new Intl.NumberFormat('en-US').format(num);
                }
            };
        } catch (e) {
            // ignore
        }

        // Re-initialize report visuals after language change (if on a report page)
        if (document.getElementById('report-container')) {
            setTimeout(() => {
                callInitializeReportVisuals();
            }, 100);
        }
    }

    // =====================================
    // REPORT VISUALS INITIALIZATION LOGIC
    // =====================================
    
    /**
     * Centralized function to call initializeAllVisuals_report()
     * This is now the single place where this function is called to avoid duplicates
     */
    function callInitializeReportVisuals() {
        // Check if report container exists (only call on pages with reports)
        const reportContainer = document.getElementById('report-container');
        if (!reportContainer) {
            return; // Not a report page
        }

        // Check if the function exists
        if (typeof initializeAllVisuals_report !== 'function') {
            console.warn("⚠️ initializeAllVisuals_report function not found");
            return;
        }

        // Check if chart libraries are loaded
        if (typeof createGauge !== 'function' || 
            typeof createDoughnutChart !== 'function' || 
            typeof createBarChart !== 'function') {
            console.warn("⚠️ Chart libraries not loaded yet. Retrying...");
            setTimeout(callInitializeReportVisuals, 500);
            return;
        }

        try {
            console.log("🎨 Calling initializeAllVisuals_report() from language-toggle.js");
            initializeAllVisuals_report();
        } catch (error) {
            console.error("❌ Error calling initializeAllVisuals_report:", error);
        }
    }

    /**
     * Initialize report visuals with retry mechanism
     */
    function initializeReportVisualsWithRetry(maxRetries = 3, delay = 1000) {
        let attempts = 0;
        
        function attempt() {
            attempts++;
            console.log(`🔄 Attempting to initialize report visuals (${attempts}/${maxRetries})`);
            
            const reportContainer = document.getElementById('report-container');
            if (!reportContainer) {
                return; // Not a report page
            }

            const hasFunction = typeof initializeAllVisuals_report === 'function';
            const hasChartLibs = typeof createGauge === 'function' && 
                                typeof createDoughnutChart === 'function' && 
                                typeof createBarChart === 'function';

            if (hasFunction && hasChartLibs) {
                callInitializeReportVisuals();
                return;
            }

            if (attempts < maxRetries) {
                setTimeout(attempt, delay);
            } else {
                console.warn("⚠️ Failed to initialize report visuals after max retries");
            }
        }
        
        attempt();
    }

    function toggleLanguage(){
        const current = getPreferredLanguage();
        const next = (current === 'vi') ? 'en' : 'vi';
        setPreferredLanguage(next);
    }

    document.addEventListener('DOMContentLoaded', function(){
        const initial = getPreferredLanguage();
        // set initial UI
        updateUI(initial);

        // Initialize report visuals after DOM is ready and language is set
        setTimeout(() => {
            initializeReportVisualsWithRetry();
        }, 100);

        // Notify other scripts about initial language so they can initialize correctly
        try {
            const evInit = new CustomEvent('languageChanged', { detail: { language: initial } });
            window.dispatchEvent(evInit);
        } catch(e) {}

        const toggleBtn = document.getElementById('language-toggle');
        if (toggleBtn) {
            toggleBtn.addEventListener('click', function(e){
                e.preventDefault();
                toggleLanguage();
            });
        }
    });
})();
