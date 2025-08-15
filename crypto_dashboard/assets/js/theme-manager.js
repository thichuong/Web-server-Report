// theme-manager.js - Quản lý theme switching cho toàn bộ ứng dụng

/**
 * Hàm thông báo tất cả iframe về việc thay đổi theme
 */
function notifyIframesThemeChange(theme) {
    const iframes = document.querySelectorAll('iframe');
    iframes.forEach(iframe => {
        try {
            iframe.contentWindow.postMessage({
                type: 'themeChange',
                theme: theme
            }, '*');
        } catch (e) {
            console.log('Could not send theme message to iframe:', e);
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
    
    // Wait a bit for iframes to load, then send initial theme
    setTimeout(() => {
        notifyIframesThemeChange(currentTheme === 'light' ? null : currentTheme);
    }, 1000);

    // Setup theme toggle button click handler
    if (themeToggleButton) {
        themeToggleButton.addEventListener('click', () => {
            const currentTheme = htmlElement.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            
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
