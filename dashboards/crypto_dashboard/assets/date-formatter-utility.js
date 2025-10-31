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

            // Format time (GMT+7 - Asia/Ho_Chi_Minh)
            const timeFormatter = new Intl.DateTimeFormat(currentLang === 'vi' ? 'vi-VN' : 'en-US', {
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
                hour12: false,
                timeZone: 'Asia/Ho_Chi_Minh'
            });

            // Format date without timezone
            const dateFormatter = new Intl.DateTimeFormat(currentLang === 'vi' ? 'vi-VN' : 'en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
                timeZone: 'Asia/Ho_Chi_Minh'
            });

            // Get timezone name
            const timezoneFormatter = new Intl.DateTimeFormat(currentLang === 'vi' ? 'vi-VN' : 'en-US', {
                timeZone: 'Asia/Ho_Chi_Minh',
                timeZoneName: 'short'
            });
            const timezoneParts = timezoneFormatter.formatToParts(date);
            const timezone = timezoneParts.find(part => part.type === 'timeZoneName')?.value || 'GMT+7';

            // Combine: time first, then date, then timezone
            const formattedTime = timeFormatter.format(date);
            const formattedDate = dateFormatter.format(date);
            const combinedFormat = `${formattedTime}, ${formattedDate} ${timezone}`;

            createdAtElement.textContent = combinedFormat;
            
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
