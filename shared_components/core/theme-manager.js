// theme-manager.js - Quản lý theme switching cho toàn bộ ứng dụng

// Debug mode - set to false for production (reduces Firefox lag)
const THEME_DEBUG = false;

/**
 * Hàm thông báo tất cả iframe về việc thay đổi theme
 */
function notifyIframesThemeChange(theme) {
    if (THEME_DEBUG) console.log('🎨 Parent: Broadcasting theme change:', theme);
    
    // ✨ Shadow DOM Support: Direct function call (standard for DSD)
    // This is much faster and cleaner than postMessage for Shadow DOM
    if (typeof window.applyReportTheme === 'function') {
        try {
            if (THEME_DEBUG) console.log('📞 Parent: Calling window.applyReportTheme():', theme);
            window.applyReportTheme(theme);
        } catch (e) {
            if (THEME_DEBUG) console.warn('⚠️ Parent: applyReportTheme failed:', e);
        }
    }

    // Legacy Iframe Support: Only run if iframes are actually present
    const allIframes = document.querySelectorAll('iframe');
    if (allIframes.length === 0) {
        if (THEME_DEBUG) console.log('📭 Parent: No iframes found for theme message');
        return;
    }

    if (THEME_DEBUG) console.log(`📨 Parent: Found ${allIframes.length} iframes, sending theme message`);

    allIframes.forEach(iframe => {
        try {
            iframe.contentWindow.postMessage({
                type: 'theme-change',
                theme: theme
            }, '*');
        } catch (e) {
            if (THEME_DEBUG) console.log('📭 Parent: Could not send theme message to iframe:', e);
        }
    });
}

/**
 * Khởi tạo theme switching
 */
function setupThemeSwitcher() {
    const themeToggleButton = document.getElementById('theme-toggle');
    const htmlElement = document.documentElement;

    // Load saved theme from localStorage
    const currentTheme = localStorage.getItem('theme') || 'light';
    htmlElement.setAttribute('data-theme', currentTheme);
    
    if (THEME_DEBUG) console.log('🎨 Parent: Initial theme loaded:', currentTheme);
    
    // Wait a bit for iframes to load, then send initial theme
    setTimeout(() => {
        if (THEME_DEBUG) console.log('🎨 Parent: Sending initial theme to iframes after delay:', currentTheme);
        notifyIframesThemeChange(currentTheme);
    }, 1000);

    // Setup theme toggle button click handler
    if (themeToggleButton) {
        themeToggleButton.addEventListener('click', () => {
            const currentTheme = htmlElement.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            
            if (THEME_DEBUG) console.log('🎨 Parent: Theme toggle clicked, switching from', currentTheme, 'to', newTheme);
            
            htmlElement.setAttribute('data-theme', newTheme);
            localStorage.setItem('theme', newTheme);
            
            // Notify all iframes about the theme change
            notifyIframesThemeChange(newTheme);
        });
    }

    // Watch for theme changes on the document (for external theme changes)
    const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
            if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {
                const newTheme = htmlElement.getAttribute('data-theme');
                if (THEME_DEBUG) console.log('🎨 Parent: Theme changed externally to:', newTheme);
                notifyIframesThemeChange(newTheme);
            }
        });
    });
    
    observer.observe(htmlElement, {
        attributes: true,
        attributeFilter: ['data-theme']
    });
}

/**
 * Khởi tạo theme manager khi DOM ready
 */
document.addEventListener('DOMContentLoaded', () => {
    setupThemeSwitcher();
});
