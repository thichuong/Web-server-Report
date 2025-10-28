/**
 * Report View Iframe Manager
 * Handles iframe height adjustment, navigation, scroll tracking, and theme/language synchronization
 */

// Global state for navigation
let currentNavigationData = null;
let currentActiveSection = null;
let navigationSidebar = null;

/**
 * Message Event Handlers
 */

// Listen for height messages from iframe content
window.addEventListener('message', function(event) {
    if (event.data && event.data.type === 'iframe-height-change') {
        handleIframeHeightChange(event);
    } else if (event.data && event.data.type === 'navigation-data') {
        handleNavigationData(event);
    } else if (event.data && event.data.type === 'active-section-change') {
        handleActiveSectionChange(event);
    } else if (event.data && event.data.type === 'scroll-position') {
        handleScrollPositionUpdate(event);
    } else if (event.data && event.data.type === 'scroll-info-response') {
        handleScrollInfoResponse(event);
    } else if (event.data && event.data.type === 'active-section-response') {
        handleActiveSectionResponse(event);
    } else if (event.data && event.data.type === 'section-position-response') {
        handleSectionPositionResponse(event);
    } else if (event.data && event.data.type === 'request-current-scroll') {
        handleCurrentScrollRequest(event);
    } else if (event.data && event.data.type === 'get-current-theme') {
        handleGetCurrentThemeRequest(event);
    }
});

// Handle current scroll position request from iframe
function handleCurrentScrollRequest(event) {
    console.log('📍 Parent: Received request for current scroll position');
    
    const iframe = document.getElementById('report-iframe');
    if (!iframe || !iframe.contentWindow) {
        console.warn('⚠️ Parent: No iframe found for scroll position update');
        return;
    }
    
    // Calculate current scroll data
    const currentScrollTop = window.pageYOffset || document.documentElement.scrollTop;
    const docHeight = document.documentElement.scrollHeight;
    const winHeight = window.innerHeight;
    const scrollPercent = docHeight > winHeight ? (currentScrollTop / (docHeight - winHeight)) * 100 : 0;
    
    // Calculate iframe's position relative to parent window
    const iframeRect = iframe.getBoundingClientRect();
    const iframeOffsetTop = iframeRect.top + currentScrollTop;
    
    console.log('📤 Parent: Sending current scroll position to iframe:', {
        scrollTop: currentScrollTop,
        iframeOffsetTop: iframeOffsetTop,
        scrollPercent: scrollPercent
    });
    
    // Update scroll progress bar immediately
    handleScrollPositionUpdate({
        data: {
            scrollPercent: scrollPercent,
            scrollTop: currentScrollTop
        }
    });
    
    // Send current scroll position to iframe
    iframe.contentWindow.postMessage({
        type: 'parent-scroll-update',
        scrollTop: currentScrollTop,
        iframeOffsetTop: iframeOffsetTop,
        iframeRect: {
            top: iframeRect.top,
            height: iframeRect.height,
            bottom: iframeRect.bottom
        },
        viewportHeight: winHeight
    }, '*');
}

// Handle get current theme request from iframe
function handleGetCurrentThemeRequest(event) {
    console.log('🎨 Parent: Received request for current theme');
    
    const iframe = document.getElementById('report-iframe');
    if (!iframe || !iframe.contentWindow) {
        console.warn('⚠️ Parent: No iframe found for theme request');
        return;
    }
    
    // Get current theme from document element
    const currentTheme = document.documentElement.getAttribute('data-theme') || 'light';
    
    console.log('📤 Parent: Sending current theme to iframe:', currentTheme);
    
    // Send current theme to iframe
    iframe.contentWindow.postMessage({
        type: 'current-theme-response',
        theme: currentTheme
    }, '*');
}

// Handle section position response from iframe
function handleSectionPositionResponse(event) {
    const { sectionId, offsetTop, success, error } = event.data;
    
    if (!success) {
        console.error('❌ Parent: Could not get section position:', error);
        return;
    }
    
    console.log('📍 Parent: Section position received - sectionId:', sectionId, 'offsetTop:', offsetTop);
    
    // Calculate scroll position relative to iframe position
    const iframe = document.getElementById('report-iframe');
    if (!iframe) return;
    
    const iframeRect = iframe.getBoundingClientRect();
    const parentScrollTop = window.pageYOffset || document.documentElement.scrollTop;
    
    // Calculate target scroll position
    const targetScrollTop = parentScrollTop + iframeRect.top + offsetTop - 100; // 100px offset for better visibility
    
    console.log('🚀 Parent: Scrolling to position:', targetScrollTop);
    
    // Smooth scroll to target position
    window.scrollTo({
        top: targetScrollTop,
        behavior: 'smooth'
    });
    
    // Update navigation highlighting
    updateSidebarNavigationActive(sectionId);
    currentActiveSection = sectionId;
    
    console.log('✅ Parent: Navigation completed for section:', sectionId);
}

// Handle iframe height change with enhanced logging
function handleIframeHeightChange(event) {
    const iframe = document.getElementById('report-iframe');
    if (iframe && iframe.contentWindow === event.source) {
        const newHeight = event.data.height;
        const reason = event.data.reason || 'Unknown';
        
        console.log(`📏 Parent: Setting iframe height to ${newHeight}px (${reason})`);
        iframe.style.height = newHeight + 'px';
        
        // Remove any fixed height and scrolling from iframe
        iframe.style.overflow = 'hidden';
        iframe.scrolling = 'no';
    }
}

// Handle navigation data from iframe
function handleNavigationData(event) {
    console.log('🧭 Parent: Received navigation data:', event.data);
    
    // Validate navigation data
    if (!event.data || !event.data.sections || event.data.sections.length === 0) {
        console.warn('⚠️ Parent: Invalid or empty navigation data received');
        return;
    }
    
    currentNavigationData = event.data;
    console.log('📊 Parent: Processing', currentNavigationData.sections.length, 'navigation sections');
    createSidebarNavigation();
}

// Handle active section change from iframe
function handleActiveSectionChange(event) {
    currentActiveSection = event.data.activeSection;
    updateSidebarNavigationActive(event.data.activeSection);
}

// Handle scroll position updates from iframe
function handleScrollPositionUpdate(event) {
    const scrollData = event.data;
    const sidebar = document.getElementById('navigation-sidebar');
    
    if (sidebar) {
        // Update scroll progress bar
        const scrollIndicator = sidebar.querySelector('.scroll-indicator');
        const scrollProgressText = sidebar.querySelector('.scroll-progress-text');
        
        if (scrollIndicator) {
            scrollIndicator.style.width = scrollData.scrollPercent + '%';
        }
        
        if (scrollProgressText) {
            const isVietnamese = currentNavigationData && currentNavigationData.language === 'vi';
            const progressText = isVietnamese ? 'Đọc' : 'Read';
            scrollProgressText.textContent = `${progressText}: ${Math.round(scrollData.scrollPercent)}%`;
        }
        
        // Optionally log scroll updates (can be disabled for performance)
        if (window.debugScrollTracking) {
            console.log('📜 Parent: Iframe scroll - Top:', scrollData.scrollTop, 
                       'Percent:', scrollData.scrollPercent + '%');
        }
    }
}

// Handle scroll info response from iframe
function handleScrollInfoResponse(event) {
    const scrollData = event.data;
    
    // Update progress bar
    handleScrollPositionUpdate(event);
    
    // Update active section if provided
    if (scrollData.activeSection) {
        updateSidebarNavigationActive(scrollData.activeSection);
    }
}

// Handle active section response (simplified)
function handleActiveSectionResponse(event) {
    const data = event.data;
    
    if (data.activeSection) {
        currentActiveSection = data.activeSection;
        updateSidebarNavigationActive(data.activeSection);
        
        const reason = data.reason || 'scroll';
        console.log(`📍 Parent: Updated active section to "${data.activeSection}" (${reason})`);
        
        // Log debug info if available
        if (data.debug) {
            console.log('🔍 Debug info:', data.debug);
        }
    }
}

/**
 * Navigation Management
 */

// Create navigation in left sidebar
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
    
    // Create navigation HTML with proper escaping
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
                ${currentNavigationData.sections.map((section, index) => {
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
    
    // Remove any existing event listeners to prevent duplicates
    navigationSidebar.removeEventListener('click', handleSidebarNavClick);
    
    // Add single click handler to sidebar
    navigationSidebar.addEventListener('click', handleSidebarNavClick);
    
    console.log('✅ Parent: Navigation created successfully');
    console.log('🔗 Parent: Event listener attached to navigation sidebar');
}

// Handle sidebar navigation clicks
function handleSidebarNavClick(event) {
    console.log('🔴 Parent: Click detected on navigation sidebar:', event.target);
    
    // Find the clicked navigation link
    const link = event.target.closest('a[data-section-id]');
    if (!link) {
        console.log('❌ Parent: No navigation link found in click target');
        return;
    }
    
    // Prevent default link behavior only after we confirm it's a valid navigation link
    event.preventDefault();
    event.stopPropagation();
    
    const sectionId = link.getAttribute('data-section-id');
    if (!sectionId) {
        console.log('❌ Parent: No section ID found on clicked link');
        return;
    }
    
    console.log('🎯 Parent: Navigating to section:', sectionId);
    
    // Request section position from iframe
    const iframe = document.getElementById('report-iframe');
    if (iframe && iframe.contentWindow) {
        console.log('📤 Parent: Sending get-section-position request for:', sectionId);
        iframe.contentWindow.postMessage({
            type: 'get-section-position',
            sectionId: sectionId
        }, '*');
    } else {
        console.error('❌ Parent: Iframe not found or contentWindow unavailable');
    }
}

// Update active navigation item
function updateSidebarNavigationActive(sectionId) {
    const sidebar = document.getElementById('navigation-sidebar');
    if (!sidebar) return;
    
    const links = sidebar.querySelectorAll('a[data-section-id]');
    let foundMatch = false;
    
    links.forEach(link => {
        const isActive = link.getAttribute('data-section-id') === sectionId;
        link.classList.toggle('active', isActive);
        if (isActive) foundMatch = true;
    });
    
    if (foundMatch) {
        console.log('✅ Parent: Successfully highlighted navigation for section:', sectionId);
    } else {
        console.warn('⚠️ Parent: No navigation link found for section:', sectionId);
        // List all available section IDs for debugging
        const availableSections = Array.from(links).map(link => link.getAttribute('data-section-id'));
        console.log('Available sections:', availableSections);
    }
}

/**
 * Iframe Management
 */

// Auto-resize iframe based on content (fallback method)
function autoResizeIframe(iframe) {
    try {
        const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
        if (!iframeDoc) {
            console.warn('Cannot access iframe document - using fallback height');
            iframe.style.height = '800px';
            return;
        }
        
        const body = iframeDoc.body;
        const html = iframeDoc.documentElement;
        
        const height = Math.max(
            body.scrollHeight,
            body.offsetHeight,
            html.clientHeight,
            html.scrollHeight,
            html.offsetHeight
        );
        
        // Set iframe height and remove scrolling
        iframe.style.height = Math.max(height + 20, 400) + 'px'; // Add small buffer
        iframe.style.overflow = 'hidden';
        iframe.scrolling = 'no';
        
        console.log(`📏 Parent: Auto-resized iframe to ${height}px`);
    } catch (e) {
        console.warn('Cannot resize iframe due to cross-origin restrictions:', e);
        // Set a reasonable default height and remove scrolling
        iframe.style.height = '800px';
        iframe.style.overflow = 'hidden';
        iframe.scrolling = 'no';
    }
}

// Start scroll tracking
function startScrollTracking() {
    const iframe = document.getElementById('report-iframe');
    if (!iframe) return;
    
    let lastScrollTop = 0;
    
    function handleParentScroll() {
        const currentScrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const docHeight = document.documentElement.scrollHeight;
        const winHeight = window.innerHeight;
        const scrollPercent = docHeight > winHeight ? (currentScrollTop / (docHeight - winHeight)) * 100 : 0;
        
        // Update progress bar immediately
        const sidebar = document.getElementById('navigation-sidebar');
        if (sidebar) {
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
        
        // Send scroll update to iframe for better active section detection
        if (iframe && iframe.contentWindow) {
            // Calculate iframe's position relative to parent window
            const iframeRect = iframe.getBoundingClientRect();
            const iframeOffsetTop = iframeRect.top + currentScrollTop;
            
            iframe.contentWindow.postMessage({
                type: 'parent-scroll-update',
                scrollTop: currentScrollTop,
                iframeOffsetTop: iframeOffsetTop,
                iframeRect: {
                    top: iframeRect.top,
                    height: iframeRect.height,
                    bottom: iframeRect.bottom
                },
                viewportHeight: winHeight
            }, '*');
        }
        
        lastScrollTop = currentScrollTop;
    }
    
    // Listen to parent window scroll with throttling
    let scrollTimeout;
    window.addEventListener('scroll', () => {
        clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(handleParentScroll, 16); // ~60fps
    }, { passive: true });
    
    console.log('📜 Parent: Started enhanced scroll tracking with iframe communication');
}

// Get report ID and sandbox token from template
function getReportConfig() {
    // These will be set by the server-side template
    return {
        reportId: window.REPORT_ID || null,
        sandboxToken: window.SANDBOX_TOKEN || null
    };
}

// Initialize iframe functionality
function initializeIframe() {
    const iframe = document.getElementById('report-iframe');
    
    if (iframe) {
        console.log('🚀 Parent: Initializing iframe for report loading...');
        
        // Set iframe src dynamically based on current language
        function setIframeSrc() {
            const config = getReportConfig();
            if (!config.reportId || !config.sandboxToken) {
                console.error('❌ Parent: Missing report configuration');
                return;
            }
            
            const currentLang = (window.languageManager && window.languageManager.currentLanguage) || 'vi';
            const baseUrl = `/api/crypto_reports/${config.reportId}/sandboxed`;
            const params = new URLSearchParams({
                token: config.sandboxToken,
                lang: currentLang,
                chart_modules: 'true'
            });
            iframe.src = `${baseUrl}?${params.toString()}`;
            console.log('🔗 Parent: Set iframe src with language:', currentLang);
        }
        
        // Set initial src
        setIframeSrc();
        
        // Configure iframe for auto-resize
        iframe.style.overflow = 'hidden';
        iframe.scrolling = 'no';
        
        // Add error handling for iframe
        iframe.addEventListener('error', function() {
            console.error('❌ Parent: Iframe failed to load');
        });
        
        iframe.addEventListener('load', function() {
            console.log('✅ Parent: Iframe loaded successfully');
            
            // The iframe will send height via postMessage
            // Also try direct access as fallback
            setTimeout(() => {
                autoResizeIframe(iframe);
            }, 200);
            
            // Send initial theme to iframe
            setTimeout(() => {
                const currentTheme = document.documentElement.getAttribute('data-theme') || 'light';
                if (iframe.contentWindow) {
                    iframe.contentWindow.postMessage({
                        type: 'theme-change',
                        theme: currentTheme
                    }, '*');
                }
            }, 200);
            
            // Start scroll tracking after iframe loads
            setTimeout(() => {
                startScrollTracking();
                console.log('📜 Parent: Started scroll tracking');
            }, 200);
        });
        
        // Force load iframe immediately if it hasn't started loading
        setTimeout(() => {
            if (!iframe.src || iframe.src === 'about:blank') {
                console.warn('⚠️ Parent: Iframe src not set, attempting to reload...');
                iframe.src = iframe.getAttribute('src') || iframe.src;
            }
        }, 100);
    } else {
        console.warn('⚠️ Parent: No iframe found on page - report may be empty');
    }
}

/**
 * Language and Theme Event Handlers
 */

// Language switching for iframe
window.addEventListener('languageChanged', function(event) {
    const iframe = document.getElementById('report-iframe');
    if (iframe && iframe.contentWindow) {
        // Send language change message to iframe
        iframe.contentWindow.postMessage({
            type: 'language-change',
            language: event.detail.language
        }, '*');
    }
    
    // Update iframe src with new language
    if (iframe) {
        const config = getReportConfig();
        if (!config.reportId || !config.sandboxToken) {
            console.error('❌ Parent: Missing report configuration for language change');
            return;
        }
        
        const baseUrl = `/api/crypto_reports/${config.reportId}/sandboxed`;
        const params = new URLSearchParams({
            token: config.sandboxToken,
            lang: event.detail.language,
            chart_modules: 'true'
        });
        iframe.src = `${baseUrl}?${params.toString()}`;
        console.log('🔄 Parent: Updated iframe src for language change:', event.detail.language);
    }
    
    // Update sidebar navigation language
    if (navigationSidebar) {
        const scrollProgressText = navigationSidebar.querySelector('.scroll-progress-text');
        
        // Update scroll progress text language
        if (scrollProgressText) {
            const currentText = scrollProgressText.textContent;
            const percentMatch = currentText.match(/(\d+)%/);
            const percent = percentMatch ? percentMatch[1] : '0';
            const progressText = event.detail.language === 'en' ? 'Read' : 'Đọc';
            scrollProgressText.textContent = `${progressText}: ${percent}%`;
        }
    }
});

// Theme switching for iframe and sidebar navigation
window.addEventListener('themeChanged', function(event) {
    const iframe = document.getElementById('report-iframe');
    if (iframe && iframe.contentWindow) {
        // Send theme change message to iframe
        iframe.contentWindow.postMessage({
            type: 'theme-change',
            theme: event.detail.theme
        }, '*');
    }
    
    // Apply theme to sidebar navigation
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
 * Initialization
 */
document.addEventListener('DOMContentLoaded', initializeIframe);

