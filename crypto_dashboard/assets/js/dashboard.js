// dashboard.js - Logic riêng cho trang dashboard chính

/**
 * Định dạng số lớn thành dạng ngắn gọn (nghìn tỷ, tỷ, triệu).
 * @param {number} num - Số cần định dạng.
 * @returns {string} - Chuỗi đã được định dạng.
 */
function formatNumber(num) {
    // Sử dụng formatNumberLocalized nếu có sẵn, nếu không dùng format cũ
    if (window.languageManager && window.languageManager.formatNumberLocalized) {
        return window.languageManager.formatNumberLocalized(num);
    }
    
    // Fallback to old format
    if (num === null || num === undefined) return 'N/A';
    if (num >= 1e12) return (num / 1e12).toFixed(2) + ' nghìn tỷ';
    if (num >= 1e9) return (num / 1e9).toFixed(2) + ' tỷ';
    if (num >= 1e6) return (num / 1e6).toFixed(2) + ' triệu';
    return num.toLocaleString('en-US');
}

/**
 * Lấy text đã dịch
 */
function getTranslatedText(key) {
    if (window.languageManager && window.languageManager.getTranslatedText) {
        return window.languageManager.getTranslatedText(key);
    }
    return key; // fallback
}

/**
 * Select an element for dashboard by language-aware id.
 * If `lang` is 'en' it will try id + '-en' first, then fallback to base id.
 * If no language specified, prefer window.languageManager.currentLanguage when available.
 */
function selectDashboardElementByLang(idBase, lang) {
    const language = lang || (window.languageManager && window.languageManager.currentLanguage) || 'vi';
    if (language === 'en') {
        const enEl = document.getElementById(idBase + '-en');
        if (enEl) return enEl;
    }
    return document.getElementById(idBase);
}

/**
 * Hiển thị thông báo lỗi thân thiện trên một card cụ thể.
 * @param {string} containerId - ID của container cần hiển thị lỗi.
 * @param {string} message - Thông báo lỗi.
 */
function displayError(containerId, message) {
    const container = document.getElementById(containerId);
    if (container) {
        const errorMsg = message || getTranslatedText('error-loading-data');
        container.innerHTML = `<p class="text-sm text-red-600">${errorMsg}</p>`;
    }
}

/**
 * Re-render dashboard UI from previously cached summary data without re-fetching.
 * Useful when only language changed.
 */
function renderDashboardFromCache(lang) {
    const data = window.dashboardSummaryCache;
    if (!data) return;

    // market cap
    const marketCapContainer = selectDashboardElementByLang('market-cap-container', lang);
    if (marketCapContainer) {
        marketCapContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(Number(marketCapContainer.dataset.marketCap || data.market_cap))}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    // volume
    const volumeContainer = selectDashboardElementByLang('volume-24h-container', lang);
    if (volumeContainer) {
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(Number(volumeContainer.dataset.volume24h || data.volume_24h))}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    // btc
    const btcContainer = selectDashboardElementByLang('btc-price-container', lang);
    if (btcContainer) {
        const price = btcContainer.dataset.btcPriceUsd || data.btc_price_usd;
        const change = Number(btcContainer.dataset.btcChange24h || data.btc_change_24h || 0);
        const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
        btcContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-900">${'$' + (price ? Number(price).toLocaleString('en-US') : 'N/A')}</p>
            <p class="text-sm font-semibold ${changeClass}">${!isNaN(change) ? change.toFixed(2) : 'N/A'}% (24h)</p>`;
    }

    // Fear & Greed gauge
    const fngContainer = selectDashboardElementByLang('fear-greed-container', lang);
    const fngVal = fngContainer ? Number(fngContainer.dataset.value || data.fng_value) : null;
    if (fngContainer && !isNaN(fngVal)) {
        const fngConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                { limit: 49, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
            ]
        };
        try { createGauge(fngContainer, fngVal, fngConfig); } catch(e) { console.error('createGauge lỗi khi render từ cache', e); }
    }

    // RSI
    const rsiContainer = selectDashboardElementByLang('rsi-container', lang);
    const rsiVal = rsiContainer ? Number(rsiContainer.dataset.value || data.rsi_14) : null;
    if (rsiContainer && rsiVal !== null && !isNaN(rsiVal)) {
        const rsiConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
            ]
        };
        try { createGauge(rsiContainer, rsiVal, rsiConfig); } catch(e) { console.error('createGauge lỗi khi render RSI từ cache', e); }
    }
}

/**
 * Fetch toàn bộ dữ liệu cho dashboard từ endpoint tổng hợp.
 */
async function fetchDashboardSummary() {
    // Chỉ chạy nếu có các element dashboard
    if (!document.getElementById('market-cap-container') && 
        !document.getElementById('volume-24h-container') && 
        !document.getElementById('btc-price-container')) {
        return; // Không phải trang dashboard, bỏ qua
    }

    try {
        const response = await fetch('/api/crypto/dashboard-summary', {
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        
        // Kiểm tra nếu response trống
        if (!response.body) {
            throw new Error('Server trả về response trống');
        }
        
        // Kiểm tra content-type để đảm bảo response là JSON
        const contentType = response.headers.get('content-type');
        if (!contentType || !contentType.includes('application/json')) {
            console.error('Response không phải JSON:', contentType);
            const responseText = await response.text();
            console.error('Response text:', responseText);
            throw new Error(`Server trả về định dạng không hợp lệ: ${contentType || 'unknown'}`);
        }
        
        // Đọc response text trước để kiểm tra
        const responseText = await response.text();
        if (!responseText || responseText.trim().length === 0) {
            throw new Error('Server trả về nội dung trống');
        }
        
        if (!response.ok) {
            let errorData;
            try {
                errorData = JSON.parse(responseText);
            } catch (jsonError) {
                console.error('Lỗi parse JSON từ error response:', jsonError);
                console.error('Error response text:', responseText);
                throw new Error(`Lỗi server ${response.status}: Không thể đọc response`);
            }
            const errorMessage = errorData.errors ? JSON.stringify(errorData.errors) : `Lỗi server ${response.status}`;
            throw new Error(errorMessage);
        }
        
        let data;
        try {
            data = JSON.parse(responseText);
        } catch (jsonError) {
            console.error('Lỗi parse JSON từ success response:', jsonError);
            console.error('Success response text:', responseText);
            throw new Error('Server trả về dữ liệu không hợp lệ');
        }

        // Cập nhật Vốn hóa thị trường
        const marketCapContainer = selectDashboardElementByLang('market-cap-container');
        if (marketCapContainer) {
            marketCapContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(data.market_cap)}</p>
                <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
            // cache numeric value so we can re-render visuals without re-fetch
            try { marketCapContainer.dataset.marketCap = String(data.market_cap); } catch(e){}
        }

        // Cập nhật Khối lượng giao dịch
        const volumeContainer = selectDashboardElementByLang('volume-24h-container');
        if (volumeContainer) {
            volumeContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(data.volume_24h)}</p>
                <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
            try { volumeContainer.dataset.volume24h = String(data.volume_24h); } catch(e){}
        }

        // Cập nhật giá BTC
        const btcContainer = selectDashboardElementByLang('btc-price-container');
        if (btcContainer) {
            const change = data.btc_change_24h;
            const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
            btcContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">${'$' + (data.btc_price_usd ? data.btc_price_usd.toLocaleString('en-US') : 'N/A')}</p>
                <p class="text-sm font-semibold ${changeClass}">${change !== null ? change.toFixed(2) : 'N/A'}% (24h)</p>`;
            try { btcContainer.dataset.btcPriceUsd = String(data.btc_price_usd); btcContainer.dataset.btcChange24h = String(data.btc_change_24h); } catch(e){}
        }

        // Cập nhật chỉ số Sợ hãi & Tham lam
        const fngContainer = selectDashboardElementByLang('fear-greed-container');
        const fngValue = parseInt(data.fng_value, 10);
        if (!isNaN(fngValue)) {
            const fngConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                    { limit: 49, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                    { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                    { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                    { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
                ]
            };
            createGauge(fngContainer, fngValue, fngConfig);
            try { fngContainer.dataset.value = String(fngValue); } catch(e){}
        } else {
            displayError('fear-greed-container', 'Giá trị F&G không hợp lệ.');
        }

        // Cập nhật chỉ số RSI
        const rsiContainer = selectDashboardElementByLang('rsi-container');
        const rsiValue = data.rsi_14;
        if (rsiValue !== null && rsiValue !== undefined) {
            const rsiConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                    { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                    { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
                ]
            };
            createGauge(rsiContainer, rsiValue, rsiConfig);
            try { rsiContainer.dataset.value = String(rsiValue); } catch(e){}
        } else {
             displayError('rsi-container', 'Không nhận được giá trị RSI.');
        }

        // Cache the last successful summary so we can re-render visuals on language change without re-fetching
        try { window.dashboardSummaryCache = data; } catch(e) {}

    } catch (error) {
        console.error('Lỗi fetchDashboardSummary:', error);
        console.error('Error stack:', error.stack);
        
        // Hiển thị fallback data thay vì chỉ hiển thị lỗi
        displayFallbackData();
        
        // Hiển thị thông báo lỗi nhẹ nhàng
        showErrorNotification(getTranslatedText('connection-issue'));
    }
}

/**
 * Hiển thị dữ liệu mặc định khi API không khả dụng
 */
function displayFallbackData() {
    // Hiển thị market cap fallback
    const marketCapContainer = document.getElementById('market-cap-container');
    if (marketCapContainer) {
        marketCapContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    // Hiển thị volume fallback
    const volumeContainer = document.getElementById('volume-24h-container');
    if (volumeContainer) {
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">${getTranslatedText('whole-market')}</p>`;
    }

    // Hiển thị BTC price fallback
    const btcContainer = document.getElementById('btc-price-container');
    if (btcContainer) {
        btcContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">${getTranslatedText('loading')}</p>
            <p class="text-sm text-gray-500">Bitcoin</p>`;
    }

    // Hiển thị F&G fallback
    const fngContainer = document.getElementById('fear-greed-container');
    if (fngContainer) {
        const fngConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 24, color: 'var(--fng-extreme-fear-color)', label: getTranslatedText('extreme-fear') },
                { limit: 49, color: 'var(--fng-fear-color)', label: getTranslatedText('fear') },
                { limit: 54, color: 'var(--fng-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 74, color: 'var(--fng-greed-color)', label: getTranslatedText('greed') },
                { limit: 100, color: 'var(--fng-extreme-greed-color)', label: getTranslatedText('extreme-greed') }
            ]
        };
        createGauge(fngContainer, 50, fngConfig); // Default neutral value
    }

    // Hiển thị RSI fallback
    const rsiContainer = document.getElementById('rsi-container');
    if (rsiContainer) {
        const rsiConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 30, color: 'var(--rsi-oversold-color)', label: getTranslatedText('oversold') },
                { limit: 70, color: 'var(--rsi-neutral-color)', label: getTranslatedText('neutral') },
                { limit: 100, color: 'var(--rsi-overbought-color)', label: getTranslatedText('overbought') }
            ]
        };
        createGauge(rsiContainer, 50, rsiConfig); // Default neutral value
    }
}

/**
 * Hiển thị thông báo lỗi dạng toast
 */
function showErrorNotification(message) {
    // Tạo toast notification nếu chưa có
    let notification = document.getElementById('error-notification');
    if (!notification) {
        notification = document.createElement('div');
        notification.id = 'error-notification';
        notification.className = 'fixed top-4 right-4 bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded shadow-lg z-50 max-w-sm';
        document.body.appendChild(notification);
    }
    
    notification.innerHTML = `
        <div class="flex items-center">
            <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"></path>
            </svg>
            <span class="text-sm">${message}</span>
        </div>
    `;
    
    // Tự động ẩn sau 5 giây
    setTimeout(() => {
        if (notification && notification.parentNode) {
            notification.parentNode.removeChild(notification);
        }
    }, 5000);
}

/**
 * Tải nội dung báo cáo từ file tĩnh và tạo mục lục điều hướng.
 */
async function CreateNav() {
    try {
        const reportContainer = document.getElementById('report-container');
        const navLinksContainer = document.getElementById('report-nav-links');

        // Thoát sớm nếu các container chính không tồn tại để tránh lỗi
        if (!reportContainer || !navLinksContainer) {
            console.error("Không tìm thấy container cho báo cáo (#report-container) hoặc mục lục (#report-nav-links).");
            return;
        }

        // Ngắt observer cũ (nếu có) để tránh quan sát trùng lặp
        if (reportContainer._navObserver) {
            try { reportContainer._navObserver.disconnect(); } catch(e){}
            reportContainer._navObserver = null;
        }

        // Xóa nội dung cũ của mục lục trước khi tạo mới để tránh trùng lặp
        navLinksContainer.innerHTML = '';

        // Nếu có 2 container nội dung (vi/en), chỉ lấy các section từ phần đang hiển thị
        const viContainer = document.getElementById('report-content-vi');
        const enContainer = document.getElementById('report-content-en');
        let activeContent = reportContainer; // fallback: toàn bộ reportContainer

        if (viContainer || enContainer) {
            const viVisible = viContainer && window.getComputedStyle(viContainer).display !== 'none';
            const enVisible = enContainer && window.getComputedStyle(enContainer).display !== 'none';
            if (viVisible) activeContent = viContainer;
            else if (enVisible) activeContent = enContainer;
            else activeContent = viContainer || enContainer || reportContainer;
        }

        const reportSections = activeContent.querySelectorAll('section');

        // Build navigation links only from the active content's sections
        reportSections.forEach(section => {
            const h2 = section.querySelector('h2');
            if (h2 && section.id) {
                const li = document.createElement('li');
                const a = document.createElement('a');
                a.href = `#${section.id}`;
                // remove any icon node inside h2 when constructing the label
                const h2Text = h2.cloneNode(true);
                const icon = h2Text.querySelector('i');
                if (icon && icon.parentNode) icon.parentNode.removeChild(icon);
                a.textContent = h2Text.textContent.trim();
                a.classList.add('block', 'py-1', 'px-2', 'rounded');
                // smooth scroll on click và active ngay lập tức
                a.addEventListener('click', (e) => {
                    e.preventDefault();
                    
                    const target = activeContent.querySelector(`#${section.id}`);
                    if (target) {
                        // Active ngay lập tức khi click
                        navLinksContainer.querySelectorAll('a').forEach(link => link.classList.remove('active'));
                        a.classList.add('active');
                        
                        // Scroll tới target
                        target.scrollIntoView({ behavior: 'smooth', block: 'start' });
                    }
                });
                li.appendChild(a);
                navLinksContainer.appendChild(li);
            }
        });

        const navLinks = navLinksContainer.querySelectorAll('a');

        // Quan sát các section để tự động active nav link khi scroll
        const observer = new IntersectionObserver(() => {
            // More deterministic selection:
            // Choose the section whose top is the closest to the anchor line (20% from top)
            // Preference: sections with top <= anchor (the one closest below the anchor). If none, pick the nearest section below the anchor.
            const viewportHeight = window.innerHeight;
            const anchor = viewportHeight * 0.2; // 20% from top

            let bestSection = null;
            let bestTop = -Infinity; // for tops <= anchor we want the maximum (closest to anchor from above)

            // First pass: find section top <= anchor and still at least partially visible
            reportSections.forEach(section => {
                const rect = section.getBoundingClientRect();
                // ignore sections that are completely scrolled past
                if (rect.bottom <= 0 || rect.top >= viewportHeight) return;
                if (rect.top <= anchor) {
                    if (rect.top > bestTop) {
                        bestTop = rect.top;
                        bestSection = section;
                    }
                }
            });

            // Second pass: if none found, pick the section whose top is the smallest positive distance below anchor
            if (!bestSection) {
                let minBelow = Infinity;
                reportSections.forEach(section => {
                    const rect = section.getBoundingClientRect();
                    if (rect.bottom <= 0 || rect.top >= viewportHeight) return;
                    if (rect.top > anchor && rect.top < minBelow) {
                        minBelow = rect.top;
                        bestSection = section;
                    }
                });
            }

            if (bestSection) {
                const targetId = bestSection.id;
                navLinks.forEach(link => {
                    const isTarget = link.getAttribute('href').substring(1) === targetId;
                    link.classList.toggle('active', isTarget);
                });
            }
        }, {
            root: null,
            rootMargin: "0px",
            threshold: [0, 0.1, 0.25, 0.5, 1.0]
        });

        // Quan sát tất cả sections
        reportSections.forEach(section => {
            observer.observe(section);
        });

        // Thiết lập nav link đầu tiên làm active ban đầu nếu chưa có active nào
        if (navLinks.length > 0 && !navLinksContainer.querySelector('a.active')) {
            navLinks[0].classList.add('active');
        }

        // Lưu observer vào DOM node để có thể disconnect khi tạo lại nav
        reportContainer._navObserver = observer;

        // Note: initializeAllVisuals_report() is now called from language-toggle.js
 
    } catch (error) {
        console.error('Lỗi tải báo cáo:', error);
        document.getElementById('report-container').innerHTML = '<p class="text-red-600 font-semibold">Lỗi: Không thể tải nội dung báo cáo chi tiết.</p>';
    }
}


/**
 * Hàm khởi tạo dashboard
 */
function initDashboard() {
    // Chỉ chạy nếu đang ở trang dashboard (có các element dashboard)
    if (document.getElementById('market-cap-container') || 
        document.getElementById('volume-24h-container') || 
        document.getElementById('btc-price-container')) {
        
        // Gọi hàm tổng hợp một lần khi tải trang
        fetchDashboardSummary();
        
        // Đặt lịch gọi lại hàm tổng hợp sau mỗi 10 phút
        setInterval(fetchDashboardSummary, 600000);
        
        // Lắng nghe sự kiện thay đổi ngôn ngữ — chỉ cập nhật UI (nav & visuals), không re-fetch dữ liệu
        window.addEventListener('languageChanged', (e) => {
            const lang = e?.detail?.language;
            // Rebuild navigation to match the newly visible report content (VI/EN)
            try { CreateNav(); } catch(err) { console.error('CreateNav lỗi sau khi đổi ngôn ngữ', err); }
            // Re-render dashboard cards & small charts from cached summary if available (no network call)
            try { if (window.dashboardSummaryCache) renderDashboardFromCache(lang); } catch(err) { console.error('renderDashboardFromCache lỗi', err); }
        });
    }
    
    CreateNav();


}

// Khởi tạo dashboard khi DOM ready
document.addEventListener('DOMContentLoaded', () => {
    initDashboard();
});
