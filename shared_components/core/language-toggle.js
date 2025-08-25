// language-toggle.js
// Simple language toggle between Vietnamese (vi) and English (en)
(function(){
    const LANG_KEY = 'preferred_language';
    const BACKUP_KEY = 'language';
    const DEFAULT_LANG = 'vi';

    // Minimal translation map for elements used in dashboard/report
    // Use helper to centralize how we obtain the translations object from the page.

    /**
     * Centralized function to get current translations data
     * Similar to callInitializeReportVisuals approach - get data when needed
     */
    function getTranslationsData() {
        try {
            // Method 1: Try the function if available
            if (typeof get_translations_data === 'function') {
                return get_translations_data();
            }
            // Method 2: Try global variables that might have been set
            else if (window.translations_data && typeof window.translations_data === 'object') {
                return window.translations_data;
            }
            else if (window.translations && typeof window.translations === 'object') {
                return window.translations;
            }
            else {
                // Return empty object as fallback
                return {};
            }
        } catch (error) {
            console.warn('Could not get translations data:', error);
            return {};
        }
    }

    // Setter to inject translations from external script (e.g. dashboards/.../translations.js)
    function setTranslations(obj) {
        if (!obj || typeof obj !== 'object') return;
        // Store in global variables for getTranslationsData() to access
        try { 
            window.translations_data = obj; 
            window.translations = obj; 
        } catch (e) {}
        // If language already initialized, re-run update to apply translations
        try {
            const lang = (window.languageManager && window.languageManager.currentLanguage) ? window.languageManager.currentLanguage : document.documentElement.lang || DEFAULT_LANG;
            updateUI(lang);
        } catch (e) {}
    }

    // expose setter so the translations file can call it after loading
    try {
        window.setTranslations = setTranslations;
        window.languageManager = window.languageManager || {};
        window.languageManager.setTranslations = setTranslations;
        
        // Dispatch event to notify other scripts that language-toggle is ready
        setTimeout(() => {
            try {
                const event = new CustomEvent('languageToggleReady');
                window.dispatchEvent(event);
            } catch (e) {
                console.warn('Could not dispatch languageToggleReady event:', e);
            }
        }, 50);
    } catch (e) {}
    
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
        // Get current translations data (similar to callInitializeReportVisuals approach)
        const translations_data = getTranslationsData();
        
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
            const map = translations_data[key];
            if (!map || !map[lang]) return;

            // If element has explicit language child spans, toggle them instead
            const enChild = el.querySelector('.i18n-en');
            const viChild = el.querySelector('.i18n-vi');
            if (enChild || viChild) {
                if (enChild) enChild.style.display = (lang === 'en') ? 'inline' : 'none';
                if (viChild) viChild.style.display = (lang === 'vi') ? 'inline' : 'none';
                return;
            }

            // Otherwise replace text content safely (preserve icons outside the element)
            // Use textContent to avoid interpreting HTML from translations
            el.textContent = map[lang];
        });
        // expose a minimal languageManager for other scripts
        try {
            window.languageManager = window.languageManager || {};
            window.languageManager.currentLanguage = lang;
            window.languageManager.getTranslatedText = function(key) {
                const translations_data = getTranslationsData();
                const m = translations_data[key];
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

        // Call CreateNav to rebuild navigation with translated text (if function exists)
        if (typeof CreateNav === 'function') {
            setTimeout(() => {
                try {
                    CreateNav();
                } catch (error) {
                    console.error('Error calling CreateNav after language change:', error);
                }
            }, 50);
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
     * Centralized function to call initializeAllVisuals_report() or initializeAllVisuals_report_en()
     * This is now the single place where these functions are called to avoid duplicates
     */
    function callInitializeReportVisuals() {
        // Check if report container exists (only call on pages with reports)
        const reportContainer = document.getElementById('report-container');
        if (!reportContainer) {
            return; // Not a report page
        }

        // Get current language
        const currentLang = (window.languageManager && window.languageManager.currentLanguage) ? 
            window.languageManager.currentLanguage : 
            (document.documentElement.lang || DEFAULT_LANG);

        // Determine which function to call based on language
        let initFunction, functionName;
        if (currentLang === 'en') {
            initFunction = window.initializeAllVisuals_report_en;
            functionName = 'initializeAllVisuals_report_en';
        } else {
            initFunction = window.initializeAllVisuals_report;
            functionName = 'initializeAllVisuals_report';
        }

        // Check if the function exists
        if (typeof initFunction !== 'function') {
            console.warn(`⚠️ ${functionName} function not found`);
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
            console.log(`🎨 Calling ${functionName}() from language-toggle.js`);
            initFunction();
        } catch (error) {
            console.error(`❌ Error calling ${functionName}:`, error);
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

            // Get current language
            const currentLang = (window.languageManager && window.languageManager.currentLanguage) ? 
                window.languageManager.currentLanguage : 
                (document.documentElement.lang || DEFAULT_LANG);

            // Determine which function to check based on language
            let initFunction, functionName;
            if (currentLang === 'en') {
                initFunction = window.initializeAllVisuals_report_en;
                functionName = 'initializeAllVisuals_report_en';
            } else {
                initFunction = window.initializeAllVisuals_report;
                functionName = 'initializeAllVisuals_report';
            }

            const hasFunction = typeof initFunction === 'function';
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
                console.warn(`⚠️ Failed to initialize report visuals after max retries. Missing: ${!hasFunction ? functionName + ' function' : 'chart libraries'}`);
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
