// log-manager.js - Activity Log Management
export class LogManager {
    static logEntries = [];
    
    static init() {
        this.add('🔐 Phiên truy cập an toàn được khởi tạo', 'info');
        this.add('🚀 Khởi tạo Auto Update System với Combined Research + Validation', 'info');
        
        // Setup scroll listener for activity log
        this.setupScrollListener();
        
        // Security reminder
        setTimeout(() => {
            this.add('⚠️ Nhắc nhở: Không chia sẻ URL này với người khác', 'info');
        }, 5000);
    }
    
    static add(message, type = 'info') {
        const timestamp = new Date().toLocaleString('vi-VN');
        const entry = {
            timestamp: timestamp,
            message: message,
            type: type
        };
        
        this.logEntries.unshift(entry);
        
        // Keep only last 50 entries
        if (this.logEntries.length > 50) {
            this.logEntries = this.logEntries.slice(0, 50);
        }
        
        this.updateDisplay();
        
        // Show notification badge if user is not scrolled to bottom
        this.showNewLogIndicator();
    }
    
    static updateDisplay() {
        const logContainer = document.getElementById('activity-log');
        if (!logContainer) return;
        
        // Store scroll position to check if user was at bottom
        const wasScrolledToBottom = this.isScrolledToBottom(logContainer);
        
        logContainer.innerHTML = '';
        
        // Insert entries from newest to oldest
        this.logEntries.forEach(entry => {
            const logDiv = document.createElement('div');
            logDiv.className = `log-entry log-${entry.type}`;
            logDiv.innerHTML = `<span class="log-timestamp">[${entry.timestamp}]</span> ${entry.message}`;
            logContainer.insertBefore(logDiv, logContainer.firstChild);
        });
        
        // Auto-scroll to bottom if user was already at bottom or this is a new entry
        if (wasScrolledToBottom || this.logEntries.length === 1) {
            this.scrollToBottom(logContainer);
        }
    }
    
    static isScrolledToBottom(container) {
        // Check if user is scrolled to bottom (within 5px tolerance)
        const threshold = 5;
        return container.scrollTop >= (container.scrollHeight - container.clientHeight - threshold);
    }
    
    static scrollToBottom(container) {
        // Smooth scroll to bottom
        setTimeout(() => {
            container.scrollTo({
                top: container.scrollHeight,
                behavior: 'smooth'
            });
        }, 100); // Small delay to ensure content is rendered
    }
    
    static clear() {
        this.logEntries = [];
        this.updateDisplay();
        this.hideNewLogIndicator();
        this.add('Đã xóa nhật ký hoạt động', 'info');
    }
    
    static showNewLogIndicator() {
        const logContainer = document.getElementById('activity-log');
        if (!logContainer || this.isScrolledToBottom(logContainer)) {
            return;
        }
        
        // Create or show new log indicator
        let indicator = document.getElementById('new-log-indicator');
        if (!indicator) {
            indicator = document.createElement('div');
            indicator.id = 'new-log-indicator';
            indicator.className = 'new-log-indicator';
            indicator.innerHTML = '<i class="fas fa-arrow-down mr-1"></i>Log mới';
            indicator.onclick = () => this.scrollToBottomAndHideIndicator();
            
            const activityLogSection = logContainer.closest('.status-card');
            if (activityLogSection) {
                activityLogSection.style.position = 'relative';
                activityLogSection.appendChild(indicator);
            }
        }
        
        indicator.style.display = 'flex';
    }
    
    static hideNewLogIndicator() {
        const indicator = document.getElementById('new-log-indicator');
        if (indicator) {
            indicator.style.display = 'none';
        }
    }
    
    static scrollToBottomAndHideIndicator() {
        const logContainer = document.getElementById('activity-log');
        if (logContainer) {
            this.scrollToBottom(logContainer);
            this.hideNewLogIndicator();
        }
    }
    
    static setupScrollListener() {
        // Setup scroll listener to hide indicator when user scrolls to bottom
        setTimeout(() => {
            const logContainer = document.getElementById('activity-log');
            if (logContainer) {
                logContainer.addEventListener('scroll', () => {
                    if (this.isScrolledToBottom(logContainer)) {
                        this.hideNewLogIndicator();
                    }
                });
            }
        }, 1000); // Wait for DOM to be ready
    }
}
