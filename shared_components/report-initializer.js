/**
 * report-initializer.js
 * 
 * File chuyên dụng để khởi tạo các visualization trong báo cáo.
 * Được sử dụng riêng cho trang PDF template và các trang báo cáo khác.
 * 
 * NOTE: Logic gọi initializeAllVisuals_report() đã được chuyển vào language-toggle.js
 */

/**
 * Khởi tạo tất cả các visualization cho báo cáo
 * Đảm bảo gọi hàm này sau khi DOM và các thư viện cần thiết đã được tải
 */
function initializeReportVisuals() {
    console.log("🎨 Bắt đầu khởi tạo report visuals...");
    
    // Logic moved to language-toggle.js to avoid multiple calls
    console.log("ℹ️ Logic đã được chuyển vào language-toggle.js");
}

/**
 * Khởi tạo với retry mechanism
 * Đảm bảo các visualization được tạo thành công
 * NOTE: Disabled - logic moved to language-toggle.js
 */
function initializeReportVisualsWithRetry(maxRetries = 5, retryDelay = 1000) {
    console.log("ℹ️ initializeReportVisualsWithRetry: Logic đã được chuyển vào language-toggle.js");
    // Logic moved to language-toggle.js to avoid multiple calls
}

/**
 * Khởi tạo khi DOM ready
 * NOTE: Disabled - logic moved to language-toggle.js
 */
document.addEventListener('DOMContentLoaded', function() {
    console.log("📄 DOM đã sẵn sàng, nhưng logic khởi tạo đã được chuyển vào language-toggle.js");
    // Logic moved to language-toggle.js to avoid multiple calls
});

/**
 * Khởi tạo khi window load (backup)
 */
window.addEventListener('load', function() {
    console.log("🌐 Window đã load hoàn toàn");
    
    // Chỉ thử lại nếu chưa được khởi tạo
    if (typeof window.reportVisualsInitialized === 'undefined') {
        setTimeout(() => {
            initializeReportVisualsWithRetry(3, 500);
        }, 200);
    }
});

// Export functions for manual use
window.initializeReportVisuals = initializeReportVisuals;
window.initializeReportVisualsWithRetry = initializeReportVisualsWithRetry;
