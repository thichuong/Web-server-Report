/**
 * Report View Shadow DOM Controller
 *
 * Handles Shadow DOM report rendering, navigation, and theme/language synchronization
 * Replaces the iframe-based architecture with modern Declarative Shadow DOM
 */

// Global state
let currentNavigationData = null;
let currentActiveSection = null;
let navigationSidebar = null;
let reportShadowRoot = null;

/**
 * Get preferred language from localStorage
 * @returns {string} Language code ('vi' or 'en')
 */
function getPreferredLanguageFromStorage() {
    try {
        // Try both keys for compatibility
        return localStorage.getItem('preferred_language') ||
               localStorage.getItem('language') ||
               'vi'; // Default to Vietnamese
    } catch (e) {
        console.warn('⚠️ Failed to read language from localStorage:', e);
        return 'vi';
    }
}

/**
 * Initialize Shadow DOM Report (SSR)
 * Content is already rendered by backend, we just need to access it
 */
async function initializeShadowDOMReport() {
    console.log('🚀 Parent: Initializing Shadow DOM report (SSR mode)...');

    const shadowHost = document.getElementById('report-shadow-host');
    if (!shadowHost) {
        console.error('❌ Parent: Shadow host not found');
        return;
    }

    // Check if shadow DOM is already attached (DSD SSR case)
    if (shadowHost.shadowRoot) {
        console.log('✅ Parent: Shadow DOM already attached via SSR (Declarative)');
        reportShadowRoot = shadowHost.shadowRoot;
        onShadowDOMReady();
        return;
    }

    // This should not happen with SSR, but log if it does
    console.warn('⚠️ Parent: Shadow DOM not found - this should not happen with SSR');
}

/**
 * Called when Shadow DOM is ready
 */
function onShadowDOMReady() {
    console.log('🎯 Parent: Shadow DOM is ready');

    // Synchronize initial language state from localStorage
    const preferredLanguage = getPreferredLanguageFromStorage();
    console.log('🌐 Parent: Syncing initial language to Shadow DOM:', preferredLanguage);

    // Call shadow DOM language switch immediately with preferred language
    if (window.switchReportLanguage) {
        try {
            window.switchReportLanguage(preferredLanguage);
            console.log('✅ Parent: Initial language synced to Shadow DOM:', preferredLanguage);
        } catch (e) {
            console.warn('⚠️ Parent: Failed to sync initial language:', e);
        }
    } else {
        console.warn('⚠️ Parent: window.switchReportLanguage not available yet');
        // Retry after short delay
        setTimeout(() => {
            if (window.switchReportLanguage) {
                try {
                    window.switchReportLanguage(preferredLanguage);
                    console.log('✅ Parent: Initial language synced (retry):', preferredLanguage);
                } catch (e) {
                    console.warn('⚠️ Parent: Retry failed to sync language:', e);
                }
            }
        }, 100);
    }

    // Apply current theme
    const currentTheme = document.documentElement.getAttribute('data-theme') || 'light';
    if (window.applyReportTheme) {
        window.applyReportTheme(currentTheme);
    }

    // Extract and create navigation
    extractNavigationFromShadowDOM();

    // Set up scroll tracking
    setupScrollTracking();

    console.log('✅ Parent: Shadow DOM initialization complete');
}

/**
 * Extract navigation sections from shadow DOM content
 */
function extractNavigationFromShadowDOM() {
    if (!reportShadowRoot) {
        console.warn('⚠️ Parent: No shadow root available for navigation extraction');
        return;
    }

    console.log('🧭 Parent: Extracting navigation from shadow DOM...');

    // Find active content based on current language
    const currentLang = (window.languageManager && window.languageManager.currentLanguage) || 'vi';
    const activeContent = reportShadowRoot.querySelector(`#content-${currentLang}`) ||
                          reportShadowRoot.querySelector('.lang-content.active') ||
                          reportShadowRoot.querySelector('#content-vi');

    if (!activeContent) {
        console.warn('⚠️ Parent: No active content found in shadow DOM');
        return;
    }

    // Find all sections with IDs
    const sections = activeContent.querySelectorAll('section[id]');
    console.log(`🧭 Parent: Found ${sections.length} sections in shadow DOM`);

    if (sections.length === 0) {
        return;
    }

    // Build navigation data
    const navigationData = [];
    const maxSections = 12; // Limit to prevent overflow

    sections.forEach((section, index) => {
        if (index >= maxSections) return;

        const h2 = section.querySelector('h2');
        if (h2 && section.id) {
            // Clean h2 text (remove icons)
            const h2Clone = h2.cloneNode(true);
            const icon = h2Clone.querySelector('i');
            if (icon && icon.parentNode) {
                icon.parentNode.removeChild(icon);
            }

            const title = h2Clone.textContent.trim();

            navigationData.push({
                id: section.id,
                title: title,
                fullTitle: title,
                index: index
            });
        }
    });

    currentNavigationData = {
        sections: navigationData,
        totalSections: sections.length,
        visibleSections: navigationData.length,
        language: currentLang,
        hasMore: sections.length > maxSections
    };

    console.log('🧭 Parent: Navigation data extracted:', currentNavigationData);

    // Create navigation sidebar
    createSidebarNavigation();
}

/**
 * Create navigation sidebar
 */
function createSidebarNavigation() {
    if (!currentNavigationData || !currentNavigationData.sections) {
        console.log('🧭 Parent: No navigation data available');
        return;
    }

    navigationSidebar = document.getElementById('navigation-sidebar');
    if (!navigationSidebar) {
        console.warn('⚠️ Parent: Navigation sidebar element not found');
        return;
    }

    console.log('🧭 Parent: Creating navigation with', currentNavigationData.sections.length, 'sections');

    // Show navigation sidebar
    navigationSidebar.classList.remove('hidden');

    // Escape HTML helper
    const escapeHtml = (text) => {
        const div = document.createElement('div');
        div.textContent = text || '';
        return div.innerHTML;
    };

    // Determine navigation title based on current language
    const isVietnamese = currentNavigationData && currentNavigationData.language === 'vi';
    const navTitle = isVietnamese ? '📋 Mục lục Báo cáo' : '📋 Report Contents';

    const navHTML = `
        <div class="nav-header">
            <h3 class="nav-title">
                <span class="nav-title">${navTitle}</span>
            </h3>
            <div class="scroll-progress-container mb-3">
                <div class="scroll-progress-bar bg-gray-200 h-2 rounded-full relative overflow-hidden">
                    <div class="scroll-indicator bg-blue-500 h-full rounded-full transition-all duration-300" style="width: 0%;"></div>
                </div>
                <span class="scroll-progress-text text-xs text-gray-500 mt-1 block">Đọc: 0%</span>
            </div>
        </div>
        <div class="nav-content">
            <ul class="nav-links">
                ${currentNavigationData.sections.map((section) => {
                    const safeSectionId = escapeHtml(section.id);
                    const safeSectionTitle = escapeHtml(section.title);
                    return `
                    <li>
                        <a href="#${safeSectionId}"
                           data-section-id="${safeSectionId}"
                           title="${safeSectionTitle}">
                            ${safeSectionTitle}
                        </a>
                    </li>`;
                }).join('')}
            </ul>
        </div>
    `;

    navigationSidebar.innerHTML = navHTML;

    // Add click handler
    navigationSidebar.addEventListener('click', handleSidebarNavClick);

    console.log('✅ Parent: Navigation created successfully');
}

/**
 * Handle sidebar navigation clicks
 */
function handleSidebarNavClick(event) {
    const link = event.target.closest('a[data-section-id]');
    if (!link) return;

    event.preventDefault();
    event.stopPropagation();

    const sectionId = link.getAttribute('data-section-id');
    if (!sectionId || !reportShadowRoot) return;

    console.log('🎯 Parent: Navigating to section:', sectionId);

    // Find section in shadow DOM
    const targetSection = reportShadowRoot.querySelector(`#${sectionId}`);
    if (!targetSection) {
        console.error('❌ Parent: Section not found in shadow DOM:', sectionId);
        return;
    }

    // Get shadow host position
    const shadowHost = document.getElementById('report-shadow-host');
    if (!shadowHost) return;

    const hostRect = shadowHost.getBoundingClientRect();
    const sectionRect = targetSection.getBoundingClientRect();

    // Calculate scroll position
    // Section rect is relative to viewport, we need to account for current scroll
    const currentScroll = window.pageYOffset || document.documentElement.scrollTop;
    const targetScroll = currentScroll + sectionRect.top - 100; // 100px offset for better visibility

    console.log('🚀 Parent: Scrolling to position:', targetScroll);

    // Smooth scroll
    window.scrollTo({
        top: targetScroll,
        behavior: 'smooth'
    });

    // Update active navigation
    updateSidebarNavigationActive(sectionId);
    currentActiveSection = sectionId;
}

/**
 * Update active navigation item
 */
function updateSidebarNavigationActive(sectionId) {
    const sidebar = document.getElementById('navigation-sidebar');
    if (!sidebar) return;

    const links = sidebar.querySelectorAll('a[data-section-id]');
    links.forEach(link => {
        const isActive = link.getAttribute('data-section-id') === sectionId;
        link.classList.toggle('active', isActive);
    });
}

/**
 * Setup scroll tracking
 */
function setupScrollTracking() {
    console.log('📜 Parent: Setting up scroll tracking...');

    let scrollTimeout;

    function handleScroll() {
        clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(() => {
            updateScrollProgress();
            updateActiveSectionFromScroll();
        }, 16); // ~60fps
    }

    window.addEventListener('scroll', handleScroll, { passive: true });

    console.log('✅ Parent: Scroll tracking enabled');
}

/**
 * Update scroll progress bar
 */
function updateScrollProgress() {
    const sidebar = document.getElementById('navigation-sidebar');
    if (!sidebar) return;

    const currentScrollTop = window.pageYOffset || document.documentElement.scrollTop;
    const docHeight = document.documentElement.scrollHeight;
    const winHeight = window.innerHeight;
    const scrollPercent = docHeight > winHeight ? (currentScrollTop / (docHeight - winHeight)) * 100 : 0;

    const scrollIndicator = sidebar.querySelector('.scroll-indicator');
    const scrollProgressText = sidebar.querySelector('.scroll-progress-text');

    if (scrollIndicator) {
        scrollIndicator.style.width = scrollPercent + '%';
    }

    if (scrollProgressText) {
        const isVietnamese = currentNavigationData && currentNavigationData.language === 'vi';
        const progressText = isVietnamese ? 'Đọc' : 'Read';
        scrollProgressText.textContent = `${progressText}: ${Math.round(scrollPercent)}%`;
    }
}

/**
 * Update active section based on scroll position
 */
function updateActiveSectionFromScroll() {
    if (!reportShadowRoot || !currentNavigationData) return;

    const currentLang = (window.languageManager && window.languageManager.currentLanguage) || 'vi';
    const activeContent = reportShadowRoot.querySelector(`#content-${currentLang}`) ||
                          reportShadowRoot.querySelector('.lang-content.active');

    if (!activeContent) return;

    const sections = activeContent.querySelectorAll('section[id]');
    if (sections.length === 0) return;

    // Find which section is currently in viewport
    const viewportCenter = window.innerHeight / 2;
    let activeSection = null;
    let minDistance = Infinity;

    sections.forEach(section => {
        const rect = section.getBoundingClientRect();
        const sectionCenter = rect.top + (rect.height / 2);
        const distance = Math.abs(sectionCenter - viewportCenter);

        if (distance < minDistance && rect.top < viewportCenter && rect.bottom > 0) {
            minDistance = distance;
            activeSection = section;
        }
    });

    if (activeSection && activeSection.id) {
        if (currentActiveSection !== activeSection.id) {
            currentActiveSection = activeSection.id;
            updateSidebarNavigationActive(activeSection.id);
        }
    }
}

/**
 * Get report configuration
 */
function getReportConfig() {
    return {
        reportId: window.REPORT_ID || null,
        shadowDomToken: window.SHADOW_DOM_TOKEN || null
    };
}

/**
 * Language change handler
 */
window.addEventListener('languageChanged', function(event) {
    console.log('🌐 Parent: Language changed to:', event.detail.language);

    // Call shadow DOM language switch function
    if (window.switchReportLanguage) {
        window.switchReportLanguage(event.detail.language);
    }

    // Re-extract navigation for new language
    setTimeout(() => {
        extractNavigationFromShadowDOM();
        updateScrollProgress();
        updateActiveSectionFromScroll();
    }, 200);
});

/**
 * Theme change handler
 */
window.addEventListener('themeChanged', function(event) {
    console.log('🎨 Parent: Theme changed to:', event.detail.theme);

    // Call shadow DOM theme function
    if (window.applyReportTheme) {
        window.applyReportTheme(event.detail.theme);
    }

    // Apply theme to navigation sidebar
    if (navigationSidebar) {
        if (event.detail.theme === 'dark') {
            navigationSidebar.classList.add('dark-theme');
            navigationSidebar.classList.remove('light-theme');
        } else {
            navigationSidebar.classList.add('light-theme');
            navigationSidebar.classList.remove('dark-theme');
        }
    }
});

/**
 * Initialize on DOM ready
 */
document.addEventListener('DOMContentLoaded', initializeShadowDOMReport);
