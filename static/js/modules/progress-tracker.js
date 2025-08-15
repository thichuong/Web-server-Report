// progress-tracker.js - WebSocket-based Progress Tracking
import { APIClient } from './api-client.js';
import { LogManager } from './log-manager.js';
import { wsClient } from './websocket-client.js';

export class ProgressTracker {
    constructor() {
        this.sessionId = null;
        this.lastUpdateTime = 0;
        this.wsUnsubscribeFunc = null;
        this.pollingInterval = null;
        this.useWebSocket = true;
    }
    
    startTracking(sessionId) {
        this.sessionId = sessionId;
        this.showProgressCard();
        
        if (this.useWebSocket && wsClient.isConnected) {
            this.startWebSocketTracking();
            LogManager.add('📡 Bắt đầu theo dõi tiến độ qua WebSocket', 'info');
        } else {
            this.startPollingFallback();
            LogManager.add('📡 Bắt đầu theo dõi tiến độ qua Polling (fallback)', 'info');
        }
    }
    
    stopTracking() {
        // Stop WebSocket tracking
        if (this.wsUnsubscribeFunc) {
            this.wsUnsubscribeFunc();
            this.wsUnsubscribeFunc = null;
            LogManager.add('⏹️ Dừng theo dõi tiến độ WebSocket', 'info');
        }
        
        // Stop polling fallback
        if (this.pollingInterval) {
            clearInterval(this.pollingInterval);
            this.pollingInterval = null;
            LogManager.add('⏹️ Dừng theo dõi tiến độ Polling', 'info');
        }
        
        // Unsubscribe from WebSocket channel
        if (this.sessionId) {
            wsClient.unsubscribe(`progress_${this.sessionId}`);
        }
        
        this.sessionId = null;
        this.lastUpdateTime = 0;
    }
    
    cancelTracking() {
        this.stopTracking();
        this.hideProgressCard();
        this.restoreButton();
    }

    async startWebSocketTracking() {
        try {
            // Ensure WebSocket is connected
            if (!wsClient.isConnected) {
                await wsClient.connect();
            }
            
            // Subscribe to progress updates for this session
            wsClient.subscribe(`progress_${this.sessionId}`);
            
            // Register message handler for progress updates
            this.wsUnsubscribeFunc = wsClient.onMessage('progress_update', (data) => {
                if (data.session_id === this.sessionId) {
                    this.processUpdate(data.data);
                }
            });
            
            // Also get initial progress state via API as fallback
            const progress = await APIClient.getProgress(this.sessionId);
            if (progress) {
                this.processUpdate(progress);
            }
        } catch (error) {
            console.warn('[ProgressTracker] WebSocket tracking failed, falling back to polling:', error);
            LogManager.add('⚠️ WebSocket thất bại, chuyển sang polling', 'warning');
            this.startPollingFallback();
        }
    }
    
    startPollingFallback() {
        this.useWebSocket = false;
        this.pollingInterval = setInterval(async () => {
            const progress = await APIClient.getProgress(this.sessionId);
            if (progress) {
                this.processUpdate(progress);
                
                if (['completed', 'error'].includes(progress.status)) {
                    this.stopTracking();
                }
            }
        }, 2000);
    }
    
    processUpdate(progress) {
        // Only update if there's actual change
        const currentUpdateTime = progress.last_update || 0;
        if (currentUpdateTime <= this.lastUpdateTime) {
            return;
        }
        this.lastUpdateTime = currentUpdateTime;
        
        // Update UI
        this.updateProgressBar(progress);
        this.updateProgressDetails(progress);
        this.updateProgressLog(progress);
        
        // Handle completion states
        if (progress.status === 'completed') {
            this.handleCompletion(progress);
            this.stopTracking(); // Stop WebSocket tracking
        } else if (progress.status === 'error') {
            this.handleError(progress);
            this.stopTracking(); // Stop WebSocket tracking
        }
    }
    
    updateProgressBar(progress) {
        // Hiển thị đúng tên bước theo workflow mới
        let cleanStepName = (progress.current_step_name || "").replace(/^[\d\[\]: ]*/, '');
        // Nếu là bước mới, dùng formatStepName để chuyển đổi
        cleanStepName = this.formatStepName(cleanStepName);
        const percentage = progress.percentage || 0;

        const progressBar = document.getElementById('progress-bar');
        const progressPercentage = document.getElementById('progress-percentage');
        const progressStepName = document.getElementById('progress-step-name');

        // Đổi màu progress bar theo từng bước (ví dụ: màu khác cho HTML, JS, CSS)
        if (progress.current_step_name && progress.current_step_name.includes('html')) {
            progressBar.style.backgroundColor = '#4F46E5'; // Indigo cho HTML
        } else if (progress.current_step_name && progress.current_step_name.includes('javascript')) {
            progressBar.style.backgroundColor = '#F59E42'; // Orange cho JS
        } else if (progress.current_step_name && progress.current_step_name.includes('css')) {
            progressBar.style.backgroundColor = '#10B981'; // Green cho CSS
        } else {
            progressBar.style.backgroundColor = '#2563EB'; // Blue mặc định
        }

        progressBar.style.width = `${percentage}%`;
        progressPercentage.textContent = `${percentage}%`;
        progressStepName.textContent = cleanStepName;
    }
    
    updateProgressDetails(progress) {
        const cleanDetails = (progress.details || "").replace(/^\[\d{2}:\d{2}:\d{2}\]\s*/, '');
        if (cleanDetails) {
            const progressDetailsText = document.getElementById('progress-details-text');
            progressDetailsText.textContent = cleanDetails;
        }
    }
    
    updateProgressLog(progress) {
        const progressLog = document.getElementById('progress-log');
        if (!progressLog) return;
        
        // Create log entry for current step
        const stepText = progress.current_step_name || '';
        const details = progress.details || '';
        const timestamp = new Date().toLocaleTimeString();
        
        // Only add new log entry if step or details changed
        if (stepText || details) {
            const logEntry = document.createElement('div');
            logEntry.className = 'log-entry log-info';
            
            let logText = '';
            if (stepText && !stepText.includes('Khởi tạo')) {
                logText = stepText;
            }
            if (details) {
                // Clean up details text
                const cleanDetails = details.replace(/^\[\d{2}:\d{2}:\d{2}\.\d+\s*/, '').replace(/^\[\d{2}:\d{2}:\d{2}\]\s*/, '');
                if (cleanDetails && cleanDetails !== logText) {
                    logText = cleanDetails;
                }
            }
            
            if (logText) {
                logEntry.innerHTML = `
                    <span class="log-timestamp">[${timestamp}]</span> ${logText}
                `;
                
                // Remove initial "waiting" entry
                const initialEntry = progressLog.querySelector('.log-entry');
                if (initialEntry && initialEntry.textContent.includes('Chờ bắt đầu')) {
                    initialEntry.remove();
                }
                
                // Add new entry and scroll to bottom
                progressLog.appendChild(logEntry);
                progressLog.scrollTop = progressLog.scrollHeight;
            }
        }
    }
    
    formatStepName(details) {
        // Workflow v2 step mappings để làm sạch tên bước
        const stepMappings = {
            "prepare_data": "📋 Chuẩn bị dữ liệu",
            "research_deep": "🔬 Nghiên cứu sâu", 
            "validate_report": "✅ Kiểm tra kết quả",
            "generate_report_content": "📝 Tạo nội dung báo cáo",
            "create_html": "🎨 Tạo HTML giao diện",
            "create_javascript": "💻 Tạo JavaScript giao diện", 
            "create_css": "🎨 Tạo CSS giao diện",
            "save_database": "💾 Lưu báo cáo"
        };
        
        for (const [key, value] of Object.entries(stepMappings)) {
            if (details.includes(key)) {
                details = details.replace(key, value);
            }
        }
        return details;
    }

    handleCompletion(progress) {
        this.updateProgressBar({ percentage: 100, current_step_name: 'Hoàn thành!' });
        this.updateProgressDetails({ details: `Báo cáo #${progress.report_id} đã được tạo thành công!` });
        
        // Show success overlay
        document.getElementById('success-message').textContent = 
            `Báo cáo #${progress.report_id} đã được tạo thành công!`;
        document.getElementById('success-overlay').style.display = 'flex';
        
        this.restoreButton();
        LogManager.add('🎉 Hoàn thành tạo báo cáo!', 'success');
    }
    
    handleError(progress) {
        this.updateProgressBar({ percentage: progress.percentage || 0, current_step_name: 'Lỗi xảy ra' });
        this.updateProgressDetails({ details: progress.details || 'Có lỗi xảy ra trong quá trình tạo báo cáo' });
        
        // Show error overlay
        document.getElementById('error-message').textContent = 
            progress.details || 'Có lỗi xảy ra trong quá trình tạo báo cáo';
        document.getElementById('error-overlay').style.display = 'flex';
        
        this.restoreButton();
        LogManager.add('💥 Có lỗi xảy ra!', 'error');
    }
    
    showProgressCard() {
        const progressCard = document.getElementById('progress-card');
        const sessionIdElement = document.getElementById('progress-session-id');
        const progressLog = document.getElementById('progress-log');
        
        progressCard.style.display = 'block';
        sessionIdElement.textContent = `Session: ${this.sessionId.substring(0, 8)}...`;
        
        // Reset state
        this.lastUpdateTime = 0;
        
        // Reset progress log
        if (progressLog) {
            progressLog.innerHTML = `
                <div class="log-entry log-info">
                    <span class="log-timestamp">[Chờ bắt đầu]</span> Hệ thống đã sẵn sàng tạo báo cáo...
                </div>
            `;
        }
        
        // Initialize progress display
        this.updateProgressBar({ percentage: 0, current_step_name: "Đang khởi tạo..." });
        this.updateProgressDetails({ details: "Chuẩn bị bắt đầu quy trình tạo báo cáo..." });
    }
    
    hideProgressCard() {
        const progressCard = document.getElementById('progress-card');
        const progressLog = document.getElementById('progress-log');
        
        progressCard.style.display = 'none';
        this.lastUpdateTime = 0;
        
        // Clear progress log
        if (progressLog) {
            progressLog.innerHTML = `
                <div class="log-entry log-info">
                    <span class="log-timestamp">[Chờ bắt đầu]</span> Hệ thống đã sẵn sàng tạo báo cáo...
                </div>
            `;
        }
    }
    
    restoreButton() {
        const btn = document.getElementById('trigger-report-btn');
        btn.innerHTML = '<i class="fas fa-play mr-2"></i>Tạo Báo Cáo Ngay';
        btn.disabled = false;
    }
    
    static showNewLogIndicator() {
        const progressLog = document.getElementById('progress-log');
        if (!progressLog || this.isScrolledToBottom(progressLog)) {
            return;
        }
        
        // Create or show new log indicator
        let indicator = document.getElementById('new-progress-log-indicator');
        if (!indicator) {
            indicator = document.createElement('div');
            indicator.id = 'new-progress-log-indicator';
            indicator.className = 'new-log-indicator';
            indicator.innerHTML = '<i class="fas fa-arrow-down mr-1"></i>Log mới';
            indicator.onclick = () => this.scrollToBottomAndHideIndicator();
            
            const progressLogSection = progressLog.closest('.progress-log-container') || progressLog.parentElement;
            if (progressLogSection) {
                progressLogSection.style.position = 'relative';
                progressLogSection.appendChild(indicator);
            }
        }
        
        indicator.style.display = 'flex';
    }
    
    static hideNewLogIndicator() {
        const indicator = document.getElementById('new-progress-log-indicator');
        if (indicator) {
            indicator.style.display = 'none';
        }
    }
    
    static scrollToBottomAndHideIndicator() {
        const progressLog = document.getElementById('progress-log');
        if (progressLog) {
            progressLog.scrollTo({
                top: progressLog.scrollHeight,
                behavior: 'smooth'
            });
            this.hideNewLogIndicator();
        }
    }
    
    static isScrolledToBottom(container) {
        // Check if user is scrolled to bottom (within 5px tolerance)
        const threshold = 5;
        return container.scrollTop >= (container.scrollHeight - container.clientHeight - threshold);
    }
}
