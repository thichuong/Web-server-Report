// ui-controller.js - Main UI Controller (Enhanced with WebSocket + PWA)
import { ProgressTracker } from './progress-tracker.js';
import { StatusManager } from './status-manager.js';
import { LogManager } from './log-manager.js';
import { APIClient } from './api-client.js';
import { wsClient } from './websocket-client.js';
import { pwaManager } from './pwa-manager.js';

export class UIController {
    constructor() {
        this.progressTracker = new ProgressTracker();
        this.statusManager = new StatusManager();
        this.isInitialized = false;
    }
    
    async init() {
        if (this.isInitialized) {
            return;
        }
        
        console.log('[UIController] Initializing Enhanced PWA Dashboard...');
        
        // Initialize core components
        LogManager.init();
        
        // Initialize PWA manager first
        await this.initializePWA();
        
        // Initialize WebSocket and Status Manager
        await this.statusManager.init();
        
        // Setup event listeners
        this.setupEventListeners();
        this.setupProgressTracking();
        this.setupWebSocketEventHandlers();
        this.setupPWAEventHandlers();
        
        this.isInitialized = true;
        console.log('[UIController] Enhanced PWA Dashboard initialized successfully');
        
        // Show initialization success message
        LogManager.add('🚀 Enhanced PWA Dashboard khởi động thành công với WebSocket + PWA support', 'success');
    }
    
    async initializePWA() {
        try {
            // PWA Manager is auto-initialized on import
            console.log('[UIController] PWA Manager initialized');
            
            // Request notification permission if not already granted
            if ('Notification' in window && Notification.permission === 'default') {
                setTimeout(async () => {
                    const granted = await pwaManager.requestNotificationPermission();
                    if (granted) {
                        LogManager.add('🔔 Push notifications enabled', 'success');
                    }
                }, 3000); // Wait 3 seconds before asking
            }
            
        } catch (error) {
            console.error('[UIController] PWA initialization error:', error);
            LogManager.add('⚠️ PWA features không khả dụng', 'warning');
        }
    }
    
    setupEventListeners() {
        // Global functions for HTML onclick handlers
        window.triggerManualReport = () => this.triggerManualReport();
        window.viewLatestReport = () => this.viewLatestReport();
        window.clearLog = () => LogManager.clear();
        window.refreshStatus = () => this.statusManager.refresh();
        window.closeSuccessOverlay = () => this.closeSuccessOverlay();
        window.closeErrorOverlay = () => this.closeErrorOverlay();
        window.cancelProgress = () => this.cancelProgress();
        
        // PWA functions
        window.installPWA = () => pwaManager.installApp();
        window.updatePWA = () => pwaManager.updateApp();
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (event) => {
            // Ctrl+R: Refresh status
            if (event.ctrlKey && event.key === 'r') {
                event.preventDefault();
                this.statusManager.refresh();
            }
            
            // Ctrl+Shift+C: Clear logs
            if (event.ctrlKey && event.shiftKey && event.key === 'C') {
                event.preventDefault();
                LogManager.clear();
            }
        });
        
        // Page visibility change handler
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                // Refresh data when page becomes visible
                this.statusManager.refresh();
                LogManager.add('📱 Page visible - refreshing data', 'info');
            }
        });
    }
    
    setupProgressTracking() {
        this.progressTracker.setupProgressLogScrollListener();
    }
    
    setupWebSocketEventHandlers() {
        // Listen for WebSocket connection status
        wsClient.onConnectionChange((state, data) => {
            switch (state) {
                case 'connected':
                    LogManager.add('🔗 WebSocket connected - Real-time updates active', 'success');
                    this.updateConnectionStatus('websocket', true);
                    break;
                    
                case 'disconnected':
                    LogManager.add('⚠️ WebSocket disconnected - Attempting reconnection', 'warning');
                    this.updateConnectionStatus('websocket', false);
                    break;
                    
                case 'error':
                    LogManager.add(`❌ WebSocket error: ${data?.message || 'Connection failed'}`, 'error');
                    this.updateConnectionStatus('polling', false);
                    break;
                    
                case 'max_reconnect_attempts':
                    LogManager.add('🔄 WebSocket max reconnect attempts reached - Using polling fallback', 'warning');
                    this.updateConnectionStatus('polling', false);
                    break;
            }
        });
        
        // Listen for real-time report completion
        wsClient.onMessage('report_completed', (data) => {
            console.log('[UIController] Report completed via WebSocket:', data);
            LogManager.add(`📊 New report completed: Report #${data.data.id}`, 'success');
            
            // Show notification if app is in background
            if (document.hidden && 'Notification' in window && Notification.permission === 'granted') {
                new Notification('Report Completed', {
                    body: `New crypto analysis report is ready`,
                    icon: '/static/icons/icon-192x192.png',
                    badge: '/static/icons/badge-72x72.png'
                });
            }
            
            // Refresh status to show new report
            this.statusManager.refresh();
        });
    }
    
    setupPWAEventHandlers() {
        // Listen for PWA events
        window.addEventListener('network-status-change', (event) => {
            const isOnline = event.detail.isOnline;
            
            if (isOnline) {
                LogManager.add('🌐 Network connection restored', 'success');
                // Trigger background sync when back online
                pwaManager.triggerBackgroundSync();
            } else {
                LogManager.add('📱 Working offline - Limited functionality', 'warning');
            }
        });
        
        // Listen for background sync completion
        window.addEventListener('dashboard-data-updated', (event) => {
            LogManager.add('🔄 Dashboard data synced in background', 'info');
            this.statusManager.refresh();
        });
        
        window.addEventListener('reports-updated', (event) => {
            LogManager.add('📋 Reports synced in background', 'info');
        });
        
        // Listen for PWA installation
        window.addEventListener('appinstalled', () => {
            LogManager.add('📲 App installed successfully!', 'success');
        });
    }
    
    updateConnectionStatus(type, isConnected) {
        const connectionTypeElement = document.getElementById('connection-type');
        if (!connectionTypeElement) return;
        
        if (type === 'websocket' && isConnected) {
            connectionTypeElement.className = 'status-indicator status-active';
            connectionTypeElement.innerHTML = '<i class="fas fa-bolt mr-2"></i>Real-time (WebSocket)';
        } else if (type === 'polling') {
            connectionTypeElement.className = 'status-indicator status-inactive';
            connectionTypeElement.innerHTML = '<i class="fas fa-clock mr-2"></i>Polling (Fallback)';
        }
    }
    
    async triggerManualReport() {
        // Confirmation dialog
        if (!confirm('Bạn có chắc chắn muốn tạo báo cáo mới với Combined Research + Validation? Quá trình này có thể mất vài phút.')) {
            return;
        }
        
        const btn = document.getElementById('trigger-report-btn');
        const originalContent = btn.innerHTML;
        
        // Show loading state
        btn.innerHTML = '<i class="fas fa-spinner fa-spin mr-2"></i>Đang tạo báo cáo (Combined)...';
        btn.disabled = true;
        
        LogManager.add('🚀 Bắt đầu tạo báo cáo với Combined Research + Validation', 'info');
        
        try {
            const data = await APIClient.triggerReport();
            
            if (data.success) {
                LogManager.add(`📡 Đã kết nối Combined Workflow tracking: ${data.session_id}`, 'info');
                this.progressTracker.startTracking(data.session_id);
                LogManager.add('✅ Combined Research + Validation Workflow đã được khởi chạy', 'success');
            } else {
                this.showError(data.message);
                LogManager.add(`❌ Lỗi Combined Workflow: ${data.message}`, 'error');
                this.restoreButton(btn, originalContent);
            }
            
        } catch (error) {
            const errorMsg = `Lỗi kết nối Combined Workflow: ${error.message}`;
            this.showError(errorMsg);
            LogManager.add(`🔌 ${errorMsg}`, 'error');
            this.restoreButton(btn, originalContent);
        }
    }
    
    viewLatestReport() {
        LogManager.add('📄 Chuyển đến trang chủ để xem báo cáo mới nhất', 'info');
        window.open('/', '_blank');
    }
    
    closeSuccessOverlay() {
        document.getElementById('success-overlay').style.display = 'none';
        this.progressTracker.stopTracking();
        this.statusManager.refresh(); // Refresh to show new report count
    }
    
    closeErrorOverlay() {
        document.getElementById('error-overlay').style.display = 'none';
        this.progressTracker.stopTracking();
    }
    
    cancelProgress() {
        if (this.progressTracker.sessionId) {
            if (confirm('Bạn có chắc chắn muốn dừng quá trình Combined Research + Validation?')) {
                LogManager.add('🛑 Người dùng đã dừng Combined Workflow', 'info');
                this.progressTracker.cancelTracking();
            }
        }
    }
    
    showError(message) {
        document.getElementById('error-message').textContent = message;
        document.getElementById('error-overlay').style.display = 'flex';
    }
    
    restoreButton(btn, originalContent) {
        btn.innerHTML = originalContent;
        btn.disabled = false;
    }
}
