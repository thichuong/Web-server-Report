// main.js - DEPRECATED FILE
// 
// This file is deprecated and should not be used in new code.
// Functionality has been moved to:
// - dashboard.js for dashboard-specific logic  
// - theme-manager.js for theme switching
// - upload.js for upload page functionality
//
// This file is kept only for backward compatibility and will be removed in future versions.

console.warn('⚠️  main.js is deprecated. Please use dashboard.js and theme-manager.js instead.');

// If you see this warning, update your HTML templates to use the new JS files:
// - Replace main.js with dashboard.js for index.html
// - Add theme-manager.js for all pages that need theme switching
// - Use only upload.js for upload.html (no main.js needed)

/**
 * Fetch toàn bộ dữ liệu cho dashboard từ endpoint tổng hợp.
 */
async function fetchDashboardSummary() {
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
        const marketCapContainer = document.getElementById('market-cap-container');
        if (marketCapContainer) {
            marketCapContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(data.market_cap)}</p>
                <p class="text-sm text-gray-500">Toàn thị trường</p>`;
        }

        // Cập nhật Khối lượng giao dịch
        const volumeContainer = document.getElementById('volume-24h-container');
        if (volumeContainer) {
            volumeContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">${'$' + formatNumber(data.volume_24h)}</p>
                <p class="text-sm text-gray-500">Toàn thị trường</p>`;
        }

        // Cập nhật giá BTC
        const btcContainer = document.getElementById('btc-price-container');
        if (btcContainer) {
            const change = data.btc_change_24h;
            const changeClass = change >= 0 ? 'text-green-600' : 'text-red-600';
            btcContainer.innerHTML = `
                <p class="text-3xl font-bold text-gray-900">${'$' + (data.btc_price_usd ? data.btc_price_usd.toLocaleString('en-US') : 'N/A')}</p>
                <p class="text-sm font-semibold ${changeClass}">${change !== null ? change.toFixed(2) : 'N/A'}% (24h)</p>`;
        }

        // Cập nhật chỉ số Sợ hãi & Tham lam
        const fngContainer = document.getElementById('fear-greed-container');
        const fngValue = parseInt(data.fng_value, 10);
        if (!isNaN(fngValue)) {
            const fngConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 24, color: 'var(--fng-extreme-fear-color)', label: 'Sợ hãi Cực độ' },
                    { limit: 49, color: 'var(--fng-fear-color)', label: 'Sợ hãi' },
                    { limit: 54, color: 'var(--fng-neutral-color)', label: 'Trung tính' },
                    { limit: 74, color: 'var(--fng-greed-color)', label: 'Tham lam' },
                    { limit: 100, color: 'var(--fng-extreme-greed-color)', label: 'Tham lam Cực độ' }
                ]
            };
            createGauge(fngContainer, fngValue, fngConfig);
        } else {
            displayError('fear-greed-container', 'Giá trị F&G không hợp lệ.');
        }

        // Cập nhật chỉ số RSI
        const rsiContainer = document.getElementById('rsi-container');
        const rsiValue = data.rsi_14;
        if (rsiValue !== null && rsiValue !== undefined) {
            const rsiConfig = {
                min: 0, max: 100,
                segments: [
                    { limit: 30, color: 'var(--rsi-oversold-color)', label: 'Quá bán' },
                    { limit: 70, color: 'var(--rsi-neutral-color)', label: 'Trung tính' },
                    { limit: 100, color: 'var(--rsi-overbought-color)', label: 'Quá mua' }
                ]
            };
            createGauge(rsiContainer, rsiValue, rsiConfig);
        } else {
             displayError('rsi-container', 'Không nhận được giá trị RSI.');
        }

    } catch (error) {
        console.error('Lỗi fetchDashboardSummary:', error);
        console.error('Error stack:', error.stack);
        
        // Hiển thị fallback data thay vì chỉ hiển thị lỗi
        displayFallbackData();
        
        // Hiển thị thông báo lỗi nhẹ nhàng
        showErrorNotification('Đang gặp sự cố kết nối. Hiển thị dữ liệu mặc định.');
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
            <p class="text-3xl font-bold text-gray-400">Đang tải...</p>
            <p class="text-sm text-gray-500">Toàn thị trường</p>`;
    }

    // Hiển thị volume fallback
    const volumeContainer = document.getElementById('volume-24h-container');
    if (volumeContainer) {
        volumeContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">Đang tải...</p>
            <p class="text-sm text-gray-500">Toàn thị trường</p>`;
    }

    // Hiển thị BTC price fallback
    const btcContainer = document.getElementById('btc-price-container');
    if (btcContainer) {
        btcContainer.innerHTML = `
            <p class="text-3xl font-bold text-gray-400">Đang tải...</p>
            <p class="text-sm text-gray-500">Bitcoin USD</p>`;
    }

    // Hiển thị F&G fallback
    const fngContainer = document.getElementById('fear-greed-container');
    if (fngContainer) {
        const fngConfig = {
            min: 0, max: 100,
            segments: [
                { limit: 24, color: 'var(--fng-extreme-fear-color)', label: 'Sợ hãi Cực độ' },
                { limit: 49, color: 'var(--fng-fear-color)', label: 'Sợ hãi' },
                { limit: 54, color: 'var(--fng-neutral-color)', label: 'Trung tính' },
                { limit: 74, color: 'var(--fng-greed-color)', label: 'Tham lam' },
                { limit: 100, color: 'var(--fng-extreme-greed-color)', label: 'Tham lam Cực độ' }
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
                { limit: 30, color: 'var(--rsi-oversold-color)', label: 'Quá bán' },
                { limit: 70, color: 'var(--rsi-neutral-color)', label: 'Trung tính' },
                { limit: 100, color: 'var(--rsi-overbought-color)', label: 'Quá mua' }
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
        
        // Xóa nội dung cũ của mục lục trước khi tạo mới để tránh trùng lặp
        navLinksContainer.innerHTML = '';

        const reportSections = reportContainer.querySelectorAll('section');

        reportSections.forEach(section => {
            const h2 = section.querySelector('h2');
            if (h2 && section.id) {
                const li = document.createElement('li');
                const a = document.createElement('a');
                a.href = `#${section.id}`;
                const h2Text = h2.cloneNode(true);
                h2Text.querySelector('i')?.remove(); // Sửa selector cho icon để chính xác hơn
                a.textContent = h2Text.textContent.trim();
                li.appendChild(a);
                navLinksContainer.appendChild(li);
            }
        });

        const navLinks = navLinksContainer.querySelectorAll('a');

        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    navLinks.forEach(link => {
                        link.classList.remove('active');
                        if (link.getAttribute('href').substring(1) === entry.target.id) {
                            link.classList.add('active');
                        }
                    });
                }
            });
        }, { rootMargin: "-30% 0px -70% 0px" });

        reportSections.forEach(section => {
            observer.observe(section);
        });

        // Gọi hàm vẽ các biểu đồ từ report.js SAU KHI nội dung đã được tải.
        // Điều này đảm bảo các phần tử DOM đã tồn tại để các biểu đồ có thể được vẽ.
        if (typeof initializeAllVisuals === 'function') {
            initializeAllVisuals();
        }
        // Note: initializeAllVisuals_report() is now called from language-toggle.js


    } catch (error) {
        console.error('Lỗi tải báo cáo:', error);
        document.getElementById('report-container').innerHTML = '<p class="text-red-600 font-semibold">Lỗi: Không thể tải nội dung báo cáo chi tiết.</p>';
    }
}

/**
 * Hàm khởi tạo chính
 */
function init() {
    CreateNav();
    
// Gọi hàm tổng hợp một lần khi tải trang
    fetchDashboardSummary();
    
    // Đặt lịch gọi lại hàm tổng hợp sau mỗi 10 phút
    setInterval(fetchDashboardSummary, 600000); 
}



document.addEventListener('DOMContentLoaded', () => {
    init();
});