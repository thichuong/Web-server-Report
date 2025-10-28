/**
 * Date Formatter Utility
 * Formats report creation timestamp with timezone support and i18n
 */

(function() {
    'use strict';
    
    /**
     * Format created_at timestamp with localization
     */
    function formatCreatedAt() {
        const createdAtElement = document.getElementById('report-created-at');
        if (!createdAtElement) {
            console.warn('⚠️ Date Formatter: report-created-at element not found');
            return;
        }
        
        const rawDate = createdAtElement.getAttribute('data-created-at');
        if (!rawDate) {
            console.warn('⚠️ Date Formatter: No data-created-at attribute found');
            return;
        }
        
        try {
            // Parse the date
            const date = new Date(rawDate);
            
            if (isNaN(date.getTime())) {
                console.error('❌ Date Formatter: Invalid date format:', rawDate);
                createdAtElement.textContent = rawDate; // Fallback to raw string
                return;
            }
            
            // Get current language
            const currentLang = (window.languageManager && window.languageManager.currentLanguage) || 'vi';
            
            // Format date with timezone (GMT+7 - Asia/Ho_Chi_Minh)
            const formatter = new Intl.DateTimeFormat(currentLang === 'vi' ? 'vi-VN' : 'en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
                timeZone: 'Asia/Ho_Chi_Minh',
                timeZoneName: 'short'
            });
            
            const formattedDate = formatter.format(date);
            createdAtElement.textContent = formattedDate;
            
            console.log('✅ Date Formatter: Formatted date successfully:', formattedDate);
        } catch (error) {
            console.error('❌ Date Formatter: Error formatting date:', error);
            createdAtElement.textContent = rawDate; // Fallback to raw string
        }
    }
    
    /**
     * Initialize date formatting on page load
     */
    function init() {
        // Format on initial load
        formatCreatedAt();
        
        // Re-format when language changes
        window.addEventListener('languageChanged', function(event) {
            console.log('🌐 Date Formatter: Language changed to', event.detail.language);
            formatCreatedAt();
        });
        
        console.log('✅ Date Formatter: Initialized successfully');
    }
    
    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
