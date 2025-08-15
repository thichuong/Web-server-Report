// status-manager.js - System Status Management
import { APIClient } from './api-client.js';
import { LogManager } from './log-manager.js';
import { wsClient } from './websocket-client.js';

export class StatusManager {
    constructor() {
        this.refreshInterval = null;
        this.wsConnected = false;
        this.fallbackToPolling = false;
        this.statusUpdateHandler = null;
    }
    
    async init() {
        // Try WebSocket first, fallback to polling if needed
        await this.initializeWebSocket();
        
        // Initial status fetch
        await this.refresh();
        
        // Setup fallback polling if WebSocket fails
        this.setupFallbackPolling();
    }
    
    async initializeWebSocket() {
        try {
            // Setup WebSocket connection
            await wsClient.connect();
            
            // Subscribe to system status updates
            wsClient.subscribe('system_status');
            
            // Listen for status updates
            this.statusUpdateHandler = wsClient.onMessage('status_update', (data) => {
                console.log('[StatusManager] Received real-time status update:', data);
                this.updateUI(data.data);
                LogManager.add(`Real-time cập nhật trạng thái: ${data.data.status}`, 'info');
            });
            
            // Listen for connection status
            wsClient.onConnectionChange((state, data) => {
                this.wsConnected = (state === 'connected');
                
                if (state === 'connected') {
                    console.log('[StatusManager] WebSocket connected, stopping polling fallback');
                    this.stopPolling();
                    LogManager.add('Kết nối WebSocket thành công - Chuyển sang real-time updates', 'success');
                } else if (state === 'error' || state === 'max_reconnect_attempts') {
                    console.warn('[StatusManager] WebSocket failed, falling back to polling');
                    this.fallbackToPolling = true;
                    this.startPolling();
                    LogManager.add('WebSocket lỗi - Chuyển sang polling mode', 'warning');
                }
            });
            
        } catch (error) {
            console.error('[StatusManager] WebSocket initialization failed:', error);
            this.fallbackToPolling = true;
            LogManager.add('Không thể kết nối WebSocket - Sử dụng polling mode', 'warning');
        }
    }
    
    setupFallbackPolling() {
        // Start polling if WebSocket is not connected after 5 seconds
        setTimeout(() => {
            if (!this.wsConnected) {
                console.log('[StatusManager] WebSocket not connected, starting polling fallback');
                this.fallbackToPolling = true;
                this.startPolling();
            }
        }, 5000);
    }
    
    startPolling() {
        if (this.refreshInterval) {
            return; // Already polling
        }
        
        console.log('[StatusManager] Starting polling mode');
        
        // Auto refresh status every 30 seconds (fallback mode)
        this.refreshInterval = setInterval(() => {
            // Only poll if WebSocket is not connected
            if (!this.wsConnected) {
                this.refresh();
            }
        }, 30000);
    }
    
    stopPolling() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
            console.log('[StatusManager] Stopped polling mode');
        }
    }
    
    async refresh() {
        try {
            const data = await APIClient.getSchedulerStatus();
            this.updateUI(data);
            LogManager.add(`Cập nhật trạng thái: ${data.status}`, 'info');
        } catch (error) {
            LogManager.add(`Lỗi khi tải trạng thái: ${error.message}`, 'error');
        }
    }
    
    updateUI(data) {
        this.updateSchedulerStatus(data);
        this.updateInterval(data);
        this.updateAPIKeyStatus(data);
        this.updateReportStats(data);
    }
    
    updateSchedulerStatus(data) {
        const statusElement = document.getElementById('scheduler-status');
        if (!statusElement) return;
        
        if (data.status === 'active') {
            statusElement.className = 'status-indicator status-active';
            statusElement.innerHTML = '<i class="fas fa-circle mr-2"></i>Đang hoạt động';
        } else {
            statusElement.className = 'status-indicator status-inactive';
            statusElement.innerHTML = '<i class="fas fa-circle mr-2"></i>Không hoạt động';
        }
    }
    
    updateInterval(data) {
        const intervalElement = document.getElementById('interval-info');
        if (intervalElement) {
            intervalElement.textContent = `${data.interval_hours} giờ`;
        }
    }
    
    updateAPIKeyStatus(data) {
        const apiKeyElement = document.getElementById('api-key-status');
        if (apiKeyElement) {
            apiKeyElement.textContent = data.has_api_key ? 'Đã cấu hình' : 'Chưa cấu hình';
            apiKeyElement.style.color = data.has_api_key ? 'var(--positive-color)' : 'var(--negative-color)';
        }
    }
    
    updateReportStats(data) {
        // Update total reports
        const totalReportsElement = document.getElementById('total-reports');
        if (totalReportsElement) {
            totalReportsElement.textContent = data.total_reports || 0;
        }
        
        // Update latest report time
        const latestTimeElement = document.getElementById('latest-report-time');
        if (latestTimeElement) {
            if (data.latest_report_time) {
                const date = new Date(data.latest_report_time);
                latestTimeElement.textContent = date.toLocaleString('vi-VN');
            } else {
                latestTimeElement.textContent = 'Chưa có báo cáo';
            }
        }
    }
}
