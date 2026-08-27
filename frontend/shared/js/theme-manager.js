// theme-manager.js - Quản lý theme switching cho toàn bộ ứng dụng

// Debug mode - set to false for production (reduces Firefox lag)
const THEME_DEBUG = false;

// Global observer reference for cleanup
let themeObserver = null;

/**
 * Hàm thông báo thay đổi theme cho Shadow DOM và ứng dụng
 */
function notifyThemeChange(theme) {
    if (THEME_DEBUG) console.log('🎨 Parent: Broadcasting theme change:', theme);
    
    // ✨ Shadow DOM Support: Direct function call (standard for DSD)
    if (typeof window.applyReportTheme === 'function') {
        try {
            if (THEME_DEBUG) console.log('📞 Parent: Calling window.applyReportTheme():', theme);
            window.applyReportTheme(theme);
        } catch (e) {
            if (THEME_DEBUG) console.warn('⚠️ Parent: applyReportTheme failed:', e);
        }
    }
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
    
    // Apply initial theme
    notifyThemeChange(currentTheme);

    // Setup theme toggle button click handler
    if (themeToggleButton) {
        themeToggleButton.addEventListener('click', () => {
            const currentTheme = htmlElement.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            
            if (THEME_DEBUG) console.log('🎨 Parent: Theme toggle clicked, switching from', currentTheme, 'to', newTheme);
            
            htmlElement.setAttribute('data-theme', newTheme);
            localStorage.setItem('theme', newTheme);
            
            notifyThemeChange(newTheme);
        });
    }

    // Watch for theme changes on the document (for external theme changes)
    if (themeObserver) {
        try { themeObserver.disconnect(); } catch (e) {}
        themeObserver = null;
    }

    themeObserver = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
            if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {
                const newTheme = htmlElement.getAttribute('data-theme');
                if (THEME_DEBUG) console.log('🎨 Parent: Theme changed externally to:', newTheme);
                notifyThemeChange(newTheme);
            }
        });
    });
    
    themeObserver.observe(htmlElement, {
        attributes: true,
        attributeFilter: ['data-theme']
    });
}

/**
 * Clean up theme manager observers on page unload / navigation
 */
function cleanupThemeManager() {
    if (themeObserver) {
        try { themeObserver.disconnect(); } catch (e) {}
        themeObserver = null;
    }
}

window.addEventListener('beforeunload', cleanupThemeManager);
window.addEventListener('pagehide', cleanupThemeManager);

/**
 * Khởi tạo theme manager khi DOM ready
 */
document.addEventListener('DOMContentLoaded', () => {
    setupThemeSwitcher();
});
