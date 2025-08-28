// theme-manager.js - Quản lý theme switching cho toàn bộ ứng dụng

/**
 * Hàm thông báo tất cả iframe về việc thay đổi theme
 */
function notifyIframesThemeChange(theme) {
    console.log('🎨 Parent: Broadcasting theme change to iframes:', theme);
    
    // Target specifically sandboxed iframes first
    const sandboxedIframes = document.querySelectorAll('iframe[src*="/api/sandboxed"]');
    if (sandboxedIframes.length > 0) {
        sandboxedIframes.forEach(iframe => {
            try {
                console.log('📨 Parent: Sending theme change message to sandboxed iframe:', theme);
                iframe.contentWindow.postMessage({
                    type: 'theme-change',
                    theme: theme
                }, '*');
            } catch (e) {
                console.warn('❌ Parent: Could not send theme message to sandboxed iframe:', e);
            }
        });
    }
    
    // Also send to all other iframes as fallback
    const allIframes = document.querySelectorAll('iframe');
    allIframes.forEach(iframe => {
        // Skip sandboxed iframes as they were already handled above
        if (iframe.src && iframe.src.includes('/api/sandboxed')) {
            return;
        }
        
        try {
            iframe.contentWindow.postMessage({
                type: 'theme-change',
                theme: theme
            }, '*');
        } catch (e) {
            console.log('📭 Parent: Could not send theme message to regular iframe:', e);
        }
    });
    
    if (sandboxedIframes.length === 0 && allIframes.length === 0) {
        console.log('📭 Parent: No iframes found for theme message');
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
    
    console.log('🎨 Parent: Initial theme loaded:', currentTheme);
    
    // Wait a bit for iframes to load, then send initial theme
    setTimeout(() => {
        console.log('🎨 Parent: Sending initial theme to iframes after delay:', currentTheme);
        notifyIframesThemeChange(currentTheme);
    }, 1000);

    // Setup theme toggle button click handler
    if (themeToggleButton) {
        themeToggleButton.addEventListener('click', () => {
            const currentTheme = htmlElement.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            
            console.log('🎨 Parent: Theme toggle clicked, switching from', currentTheme, 'to', newTheme);
            
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
                console.log('🎨 Parent: Theme changed externally to:', newTheme);
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
