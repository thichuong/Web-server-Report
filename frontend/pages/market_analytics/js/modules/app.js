/**
 * Main Controller & Diagnostics Manager for Market Analytics
 * (High-Performance Architecture with Frame Scheduling, Incremental Updates & Zero-Overhead Logging)
 */

import { formatNumber, formatPrice, formatDate, getI18nText } from './utils.js';
import { TABLE_COLUMNS, STACKED_COLUMNS, PRESETS } from './table-columns.js';
import { BinanceApiClient } from './api-client.js';
import { BinanceWebSocketManager } from './websocket-manager.js';
import { AnalyticsEngine } from './analytics-engine.js';
import { ChartManager } from './chart-manager.js';

export class MarketAnalyticsApp {
    constructor() {
        this.apiClient = new BinanceApiClient();
        this.chartManager = new ChartManager();

        this.wsManager = new BinanceWebSocketManager({
            onSpotKline: (k) => this.handleSpotKlineTick(k),
            onFuturesKline: (k) => this.handleFuturesKlineTick(k),
            onSpotTicker: (t) => this.handleSpotTickerTick(t),
            onFuturesTicker: (t) => this.handleFuturesTickerTick(t),
            onStatusChange: (market, status) => this.handleWsStatusChange(market, status),
            onLog: (cat, msg) => this.addDebugLog(cat, msg)
        });

        this.state = {
            symbol: 'BTCUSDT',
            timeframe: '1h',
            limit: 100,
            tableLimit: 30,
            tableLayout: 'stacked', // 'stacked' (nhóm xuống hàng) hoặc 'expanded' (từng cột riêng)
            visibleColumns: new Set(PRESETS.standard),
            autoRefresh: true,
            bgSyncIntervalSec: 25,
            bgSyncId: null,
            isLoading: false,
            currentData: null,
            rawSpotKlines: [],
            rawFutKlines: [],
            rawOiHistory: [],
            rawLsHistory: [],
            latestTickers: { spot: null, futures: null, currentOi: null },
            renderCount: 0
        };

        this.debugLogBuffer = [];
        this.maxDebugLogs = 100;
        this.rafPending = false;
        this.lastRenderTime = 0;
        this.isTabHidden = false;
        this.rafId = null;
        this.themeObserver = null;

        this.init();
    }

    init() {
        this.bindElements();
        this.initColumnVisibility();
        this.bindEvents();
        this.listenThemeChanges();
        this.listenVisibilityChanges();
        this.listenPageUnload();
        this.fetchInitialSnapshot();
        this.startBackgroundSync();

        // Expose globally for developer inspection & diagnostics
        window.marketAnalyticsApp = this;
        window.marketAnalyticsDebug = {
            app: this,
            getStatus: () => ({
                symbol: this.state.symbol,
                timeframe: this.state.timeframe,
                spotConnected: this.wsManager.spotConnected,
                futuresConnected: this.wsManager.futuresConnected,
                spotMsgCount: this.wsManager.spotMsgCount,
                futuresMsgCount: this.wsManager.futuresMsgCount,
                spotKlines: this.state.rawSpotKlines.length,
                futKlines: this.state.rawFutKlines.length,
                renderCount: this.state.renderCount,
                tableLayout: this.state.tableLayout,
                visibleColumns: Array.from(this.state.visibleColumns)
            }),
            reconnect: () => this.wsManager.subscribe(this.state.symbol, this.state.timeframe),
            fetchSnapshot: () => this.fetchInitialSnapshot(),
            simulateTick: () => this.simulateTestTick(),
            destroy: () => this.destroy()
        };
    }

    bindElements() {
        this.el = {
            symbolInput: document.getElementById('custom-symbol-input'),
            symbolButtons: document.querySelectorAll('.symbol-btn'),
            timeframeButtons: document.querySelectorAll('.timeframe-btn'),
            limitButtons: document.querySelectorAll('.limit-btn'),
            btnRefresh: document.getElementById('btn-manual-refresh'),
            autoRefreshToggle: document.getElementById('auto-refresh-toggle'),
            countdownBadge: document.getElementById('refresh-countdown'),
            apiLatency: document.getElementById('api-latency-badge'),
            wsPulseDot: document.getElementById('ws-pulse-dot'),
            wsStatusTitle: document.getElementById('ws-status-title'),
            loadingOverlay: document.getElementById('loading-overlay'),
            errorMessage: document.getElementById('error-banner'),
            errorText: document.getElementById('error-banner-text'),
            btnExportCsv: document.getElementById('btn-export-csv'),

            // Column Customizer & Presets & Layout Elements
            wrapperColumnCustomizer: document.getElementById('wrapper-column-customizer'),
            wrapperTablePresets: document.getElementById('wrapper-table-presets'),
            btnToggleColumnMenu: document.getElementById('btn-toggle-column-menu'),
            columnCustomizerDropdown: document.getElementById('column-customizer-dropdown'),
            btnCloseColumnMenu: document.getElementById('btn-close-column-menu'),
            badgeColumnsCount: document.getElementById('badge-columns-count'),
            btnColSelectAll: document.getElementById('btn-col-select-all'),
            btnColResetDefault: document.getElementById('btn-col-reset-default'),
            btnColClearAll: document.getElementById('btn-col-clear-all'),
            columnCheckboxesContainer: document.getElementById('column-checkboxes-container'),
            layoutButtons: document.querySelectorAll('.btn-table-layout'),
            presetButtons: document.querySelectorAll('.btn-table-preset'),
            tableLimitButtons: document.querySelectorAll('.btn-table-limit'),

            // Debug Panel Elements
            debugPanel: document.getElementById('debug-panel'),
            btnToggleDebug: document.getElementById('btn-toggle-debug'),
            btnCloseDebug: document.getElementById('btn-close-debug'),
            btnDbgReconnect: document.getElementById('btn-dbg-reconnect'),
            btnDbgRefresh: document.getElementById('btn-dbg-refresh'),
            btnDbgSimTick: document.getElementById('btn-dbg-sim-tick'),
            btnDbgClear: document.getElementById('btn-dbg-clear'),
            dbgSpotStatus: document.getElementById('dbg-spot-status'),
            dbgFutStatus: document.getElementById('dbg-fut-status'),
            dbgSpotCount: document.getElementById('dbg-spot-count'),
            dbgFutCount: document.getElementById('dbg-fut-count'),
            dbgSpotLast: document.getElementById('dbg-spot-last'),
            dbgFutLast: document.getElementById('dbg-fut-last'),
            dbgBufferCounts: document.getElementById('dbg-buffer-counts'),
            dbgAlignedCount: document.getElementById('dbg-aligned-count'),
            dbgRenderCount: document.getElementById('dbg-render-count'),
            dbgLastRenderTime: document.getElementById('dbg-last-render-time'),
            dbgLogContainer: document.getElementById('dbg-log-container'),

            // KPI Elements
            kpiSpotVol: document.getElementById('kpi-spot-vol'),
            kpiSpotBuySell: document.getElementById('kpi-spot-buy-sell'),
            kpiFutVol: document.getElementById('kpi-fut-vol'),
            kpiFutBuySell: document.getElementById('kpi-fut-buy-sell'),
            kpiFsRatio: document.getElementById('kpi-fs-ratio'),
            kpiBsRatio: document.getElementById('kpi-bs-ratio'),
            kpiOiUsdt: document.getElementById('kpi-oi-usdt'),
            kpiOiCoins: document.getElementById('kpi-oi-coins'),
            kpiLongPos: document.getElementById('kpi-long-pos'),
            kpiLongDesc: document.getElementById('kpi-long-desc'),
            kpiShortPos: document.getElementById('kpi-short-pos'),
            kpiShortDesc: document.getElementById('kpi-short-desc'),
            kpiCorrelation: document.getElementById('kpi-correlation'),
            kpiCorrelationStatus: document.getElementById('kpi-correlation-status'),
            kpiMarketStateBadge: document.getElementById('kpi-market-state-badge'),
            kpiMarketStateDesc: document.getElementById('kpi-market-state-desc'),

            // Table element
            dataTableHead: document.getElementById('analytics-data-table-head'),
            dataTableBody: document.getElementById('analytics-data-table-body'),
            tableSymbolLabel: document.getElementById('table-current-symbol'),
            analyticsUpdatedAt: document.getElementById('analytics-updated-at')
        };
    }

    bindEvents() {
        // Symbol buttons click
        this.el.symbolButtons.forEach(btn => {
            btn.addEventListener('click', () => {
                const sym = btn.getAttribute('data-symbol');
                if (sym) this.setSymbol(sym);
            });
        });

        // Custom symbol input
        if (this.el.symbolInput) {
            this.el.symbolInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    let sym = this.el.symbolInput.value.trim().toUpperCase();
                    if (sym) {
                        if (!sym.endsWith('USDT')) sym += 'USDT';
                        this.setSymbol(sym);
                    }
                }
            });
        }

        // Timeframe buttons
        this.el.timeframeButtons.forEach(btn => {
            btn.addEventListener('click', () => {
                const tf = btn.getAttribute('data-timeframe');
                if (tf) this.setTimeframe(tf);
            });
        });

        // Limit buttons
        this.el.limitButtons.forEach(btn => {
            btn.addEventListener('click', () => {
                const lim = parseInt(btn.getAttribute('data-limit'), 10);
                if (lim) this.setLimit(lim);
            });
        });

        // Refresh button
        if (this.el.btnRefresh) {
            this.el.btnRefresh.addEventListener('click', () => {
                this.fetchInitialSnapshot();
            });
        }

        // Auto-refresh / Live stream toggle
        if (this.el.autoRefreshToggle) {
            this.el.autoRefreshToggle.addEventListener('change', (e) => {
                this.state.autoRefresh = e.target.checked;
                if (this.state.autoRefresh) {
                    this.addDebugLog('control', '▶️ Resumed Live WebSocket & background sync');
                    this.wsManager.subscribe(this.state.symbol, this.state.timeframe);
                    this.startBackgroundSync();
                    if (this.el.countdownBadge) {
                        this.el.countdownBadge.innerHTML = '<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping inline-block mr-1"></span>Live';
                    }
                } else {
                    this.addDebugLog('control', '⏸️ Paused Live WebSocket & background sync');
                    this.wsManager.disconnect();
                    this.stopBackgroundSync();
                    if (this.el.countdownBadge) {
                        this.el.countdownBadge.textContent = 'Paused';
                    }
                }
            });
        }

        // Column Customizer Toggle & Close
        if (this.el.btnToggleColumnMenu && this.el.columnCustomizerDropdown) {
            this.el.btnToggleColumnMenu.addEventListener('click', (e) => {
                e.stopPropagation();
                this.el.columnCustomizerDropdown.classList.toggle('hidden');
            });
        }

        if (this.el.btnCloseColumnMenu && this.el.columnCustomizerDropdown) {
            this.el.btnCloseColumnMenu.addEventListener('click', () => {
                this.el.columnCustomizerDropdown.classList.add('hidden');
            });
        }

        // Close column menu when clicking outside
        document.addEventListener('click', (e) => {
            if (this.el.columnCustomizerDropdown && !this.el.columnCustomizerDropdown.classList.contains('hidden')) {
                const isInside = this.el.columnCustomizerDropdown.contains(e.target) ||
                    (this.el.btnToggleColumnMenu && this.el.btnToggleColumnMenu.contains(e.target));
                if (!isInside) {
                    this.el.columnCustomizerDropdown.classList.add('hidden');
                }
            }
        });

        // Column Action Buttons (Select All, Reset Default, Clear All)
        if (this.el.btnColSelectAll) {
            this.el.btnColSelectAll.addEventListener('click', () => {
                this.state.visibleColumns = new Set(TABLE_COLUMNS.map(c => c.id));
                this.saveColumnVisibility();
                this.updatePresetButtonsUI();
                this.renderColumnCheckboxes();
                this.updateColumnsBadge();
                if (this.state.currentData) {
                    this.renderTable(this.state.currentData.items);
                }
            });
        }

        if (this.el.btnColResetDefault) {
            this.el.btnColResetDefault.addEventListener('click', () => {
                this.state.visibleColumns = new Set(PRESETS.standard);
                this.saveColumnVisibility();
                this.updatePresetButtonsUI();
                this.renderColumnCheckboxes();
                this.updateColumnsBadge();
                if (this.state.currentData) {
                    this.renderTable(this.state.currentData.items);
                }
            });
        }

        if (this.el.btnColClearAll) {
            this.el.btnColClearAll.addEventListener('click', () => {
                this.state.visibleColumns = new Set(['time', 'price']);
                this.saveColumnVisibility();
                this.updatePresetButtonsUI();
                this.renderColumnCheckboxes();
                this.updateColumnsBadge();
                if (this.state.currentData) {
                    this.renderTable(this.state.currentData.items);
                }
            });
        }

        // Layout Style Toggle Buttons (Stacked Multi-line vs Expanded Columns)
        this.el.layoutButtons.forEach(btn => {
            btn.addEventListener('click', () => {
                const layout = btn.getAttribute('data-layout');
                if (layout && (layout === 'stacked' || layout === 'expanded')) {
                    this.state.tableLayout = layout;
                    try {
                        localStorage.setItem('market_analytics_table_layout', layout);
                    } catch (e) {}
                    this.updateLayoutButtonsUI();
                    this.updatePresetButtonsUI();
                    if (this.state.currentData) {
                        this.renderTable(this.state.currentData.items);
                    }
                }
            });
        });

        // Quick View Preset Buttons
        this.el.presetButtons.forEach(btn => {
            btn.addEventListener('click', () => {
                const presetKey = btn.getAttribute('data-preset');
                if (presetKey && PRESETS[presetKey]) {
                    this.state.visibleColumns = new Set(PRESETS[presetKey]);
                    if (this.state.tableLayout === 'stacked') {
                        this.state.tableLayout = 'expanded';
                        try {
                            localStorage.setItem('market_analytics_table_layout', 'expanded');
                        } catch (e) {}
                        this.updateLayoutButtonsUI();
                    }
                    this.saveColumnVisibility();
                    this.updatePresetButtonsUI();
                    this.renderColumnCheckboxes();
                    this.updateColumnsBadge();
                    if (this.state.currentData) {
                        this.renderTable(this.state.currentData.items);
                    }
                }
            });
        });

        // Table Row Limit Buttons (20, 30, 50, 100)
        this.el.tableLimitButtons.forEach(btn => {
            btn.addEventListener('click', () => {
                const lim = parseInt(btn.getAttribute('data-limit'), 10);
                if (lim) {
                    this.state.tableLimit = lim;
                    this.el.tableLimitButtons.forEach(b => {
                        const bLim = parseInt(b.getAttribute('data-limit'), 10);
                        if (bLim === lim) {
                            b.className = 'btn-table-limit px-2.5 py-0.5 text-xs font-bold bg-blue-600 text-white border-t border-b border-gray-600 transition-all';
                        } else {
                            b.className = 'btn-table-limit px-2.5 py-0.5 text-xs font-medium bg-gray-700/80 hover:bg-gray-700 text-gray-300 border border-gray-600 transition-all';
                        }
                    });
                    if (this.state.currentData) {
                        this.renderTable(this.state.currentData.items);
                    }
                }
            });
        });

        // Debug Panel Controls
        if (this.el.btnToggleDebug && this.el.debugPanel) {
            this.el.btnToggleDebug.addEventListener('click', () => {
                const isHidden = this.el.debugPanel.classList.toggle('hidden');
                if (!isHidden) {
                    this.flushDebugLogsToDOM();
                }
                this.updateDebugStats();
            });
        }

        if (this.el.btnCloseDebug && this.el.debugPanel) {
            this.el.btnCloseDebug.addEventListener('click', () => {
                this.el.debugPanel.classList.add('hidden');
            });
        }

        if (this.el.btnDbgReconnect) {
            this.el.btnDbgReconnect.addEventListener('click', () => {
                this.addDebugLog('debug', 'Manual WS reconnect triggered by user');
                this.wsManager.subscribe(this.state.symbol, this.state.timeframe);
            });
        }

        if (this.el.btnDbgRefresh) {
            this.el.btnDbgRefresh.addEventListener('click', () => {
                this.addDebugLog('debug', 'Manual REST snapshot triggered by user');
                this.fetchInitialSnapshot();
            });
        }

        if (this.el.btnDbgSimTick) {
            this.el.btnDbgSimTick.addEventListener('click', () => {
                this.simulateTestTick();
            });
        }

        if (this.el.btnDbgClear && this.el.dbgLogContainer) {
            this.el.btnDbgClear.addEventListener('click', () => {
                this.debugLogBuffer = [];
                this.el.dbgLogContainer.innerHTML = '<div class="text-gray-500">// Debug log cleared</div>';
            });
        }

        // Export CSV
        if (this.el.btnExportCsv) {
            this.el.btnExportCsv.addEventListener('click', () => {
                this.exportCsv();
            });
        }

        // Listen to language changes
        window.addEventListener('languageChanged', () => {
            this.updateLayoutButtonsUI();
            this.updatePresetButtonsUI();
            this.renderColumnCheckboxes();
            if (this.state.currentData) {
                this.updateKpiDisplays(this.state.currentData);
                this.renderTable(this.state.currentData.items);
            }
        });
    }

    listenVisibilityChanges() {
        document.addEventListener('visibilitychange', () => {
            this.isTabHidden = document.hidden;
            if (!this.isTabHidden && this.state.autoRefresh) {
                // When switching back to tab, perform a single full sync
                this.executeFullProcessAndRender();
            }
        });
    }

    listenThemeChanges() {
        this.themeObserver = new MutationObserver(() => {
            this.chartManager.updateTheme();
        });
        this.themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme', 'class'] });
        window.addEventListener('themeChanged', () => {
            this.chartManager.updateTheme();
        });
    }

    listenPageUnload() {
        window.addEventListener('pagehide', () => this.destroy());
        window.addEventListener('beforeunload', () => this.destroy());
    }

    addDebugLog(category, message) {
        const now = new Date();
        const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}.${String(now.getMilliseconds()).padStart(3, '0')}`;

        let colorClass = 'text-gray-300';
        if (category === 'spot') colorClass = 'text-cyan-400';
        else if (category === 'futures') colorClass = 'text-yellow-400';
        else if (category === 'render') colorClass = 'text-emerald-400';
        else if (category === 'error') colorClass = 'text-rose-400';

        const logEntry = { timeStr, category, message, colorClass };
        this.debugLogBuffer.push(logEntry);
        if (this.debugLogBuffer.length > this.maxDebugLogs) {
            this.debugLogBuffer.shift();
        }

        // Zero-overhead: Only manipulate DOM if debug panel is currently open
        if (this.el.debugPanel && !this.el.debugPanel.classList.contains('hidden') && this.el.dbgLogContainer) {
            const logLine = document.createElement('div');
            logLine.className = 'py-0.5 border-b border-gray-900 flex items-start space-x-1.5';
            logLine.innerHTML = `<span class="text-gray-500 font-mono">[${timeStr}]</span> <span class="font-bold uppercase text-[10px] px-1 rounded bg-gray-800 text-gray-400">${category}</span> <span class="${colorClass}">${message}</span>`;
            this.el.dbgLogContainer.appendChild(logLine);

            while (this.el.dbgLogContainer.children.length > this.maxDebugLogs) {
                this.el.dbgLogContainer.removeChild(this.el.dbgLogContainer.firstChild);
            }
            this.el.dbgLogContainer.scrollTop = this.el.dbgLogContainer.scrollHeight;
        }
    }

    flushDebugLogsToDOM() {
        if (!this.el.dbgLogContainer) return;
        if (!this.debugLogBuffer.length) {
            this.el.dbgLogContainer.innerHTML = '<div class="text-gray-500">// WebSocket and rendering logs will stream here in real time...</div>';
            return;
        }

        let html = '';
        this.debugLogBuffer.forEach(entry => {
            html += `<div class="py-0.5 border-b border-gray-900 flex items-start space-x-1.5"><span class="text-gray-500 font-mono">[${entry.timeStr}]</span> <span class="font-bold uppercase text-[10px] px-1 rounded bg-gray-800 text-gray-400">${entry.category}</span> <span class="${entry.colorClass}">${entry.message}</span></div>`;
        });
        this.el.dbgLogContainer.innerHTML = html;
        this.el.dbgLogContainer.scrollTop = this.el.dbgLogContainer.scrollHeight;
    }

    handleWsStatusChange(market, status) {
        this.updateDebugStats();

        if (!this.el.apiLatency || !this.el.wsPulseDot) return;
        const isVi = (document.documentElement.lang || 'vi') === 'vi';

        const isAllConnected = this.wsManager.spotConnected && this.wsManager.futuresConnected;
        const isAnyConnecting = !isAllConnected && (this.wsManager.spotWs || this.wsManager.futuresWs);

        if (isAllConnected) {
            this.el.wsPulseDot.className = 'w-2 h-2 rounded-full bg-emerald-500 animate-pulse';
            this.el.apiLatency.textContent = 'Live WS';
            this.el.apiLatency.className = 'px-2 py-0.5 text-xs font-semibold rounded bg-emerald-500/20 text-emerald-400 border border-emerald-500/30';
        } else if (isAnyConnecting) {
            this.el.wsPulseDot.className = 'w-2 h-2 rounded-full bg-amber-500 animate-pulse';
            this.el.apiLatency.textContent = isVi ? 'Đang kết nối...' : 'Connecting...';
            this.el.apiLatency.className = 'px-2 py-0.5 text-xs font-semibold rounded bg-amber-500/20 text-amber-400 border border-amber-500/30';
        } else {
            this.el.wsPulseDot.className = 'w-2 h-2 rounded-full bg-rose-500';
            this.el.apiLatency.textContent = isVi ? 'REST Polling' : 'REST Polling';
            this.el.apiLatency.className = 'px-2 py-0.5 text-xs font-semibold rounded bg-rose-500/20 text-rose-400 border border-rose-500/30';
        }
    }

    updateDebugStats() {
        if (this.el.dbgSpotStatus) {
            this.el.dbgSpotStatus.textContent = this.wsManager.spotConnected ? 'Connected' : 'Disconnected';
            this.el.dbgSpotStatus.className = this.wsManager.spotConnected
                ? 'px-1.5 py-0.5 rounded text-[10px] font-bold bg-emerald-500/20 text-emerald-400'
                : 'px-1.5 py-0.5 rounded text-[10px] font-bold bg-rose-500/20 text-rose-400';
        }
        if (this.el.dbgFutStatus) {
            this.el.dbgFutStatus.textContent = this.wsManager.futuresConnected ? 'Connected' : 'Disconnected';
            this.el.dbgFutStatus.className = this.wsManager.futuresConnected
                ? 'px-1.5 py-0.5 rounded text-[10px] font-bold bg-emerald-500/20 text-emerald-400'
                : 'px-1.5 py-0.5 rounded text-[10px] font-bold bg-rose-500/20 text-rose-400';
        }
        if (this.el.dbgSpotCount) this.el.dbgSpotCount.textContent = this.wsManager.spotMsgCount;
        if (this.el.dbgFutCount) this.el.dbgFutCount.textContent = this.wsManager.futuresMsgCount;
        if (this.el.dbgBufferCounts) {
            this.el.dbgBufferCounts.textContent = `${this.state.rawSpotKlines.length} / ${this.state.rawFutKlines.length}`;
        }
        if (this.el.dbgAlignedCount) {
            this.el.dbgAlignedCount.textContent = this.state.currentData ? this.state.currentData.items.length : 0;
        }
        if (this.el.dbgRenderCount) this.el.dbgRenderCount.textContent = this.state.renderCount;
    }

    setSymbol(sym) {
        this.state.symbol = sym.toUpperCase();
        this.el.symbolButtons.forEach(btn => {
            if (btn.getAttribute('data-symbol') === this.state.symbol) {
                btn.className = 'symbol-btn px-3 py-1.5 rounded-lg text-xs font-bold transition-all bg-blue-600 text-white border border-blue-500 shadow-sm';
            } else {
                btn.className = 'symbol-btn px-3 py-1.5 rounded-lg text-xs font-bold transition-all bg-gray-800/40 hover:bg-gray-700/60 text-gray-300 border border-gray-700/50';
            }
        });
        if (this.el.symbolInput) this.el.symbolInput.value = this.state.symbol;
        if (this.el.tableSymbolLabel) this.el.tableSymbolLabel.textContent = this.state.symbol;
        this.addDebugLog('control', `Switched symbol to: ${this.state.symbol}`);
        this.fetchInitialSnapshot();
    }

    setTimeframe(tf) {
        this.state.timeframe = tf;
        this.el.timeframeButtons.forEach(btn => {
            if (btn.getAttribute('data-timeframe') === tf) {
                btn.className = 'timeframe-btn px-2 py-1 rounded text-xs font-semibold bg-blue-600 text-white shadow-sm transition-colors';
            } else {
                btn.className = 'timeframe-btn px-2 py-1 rounded text-xs font-semibold text-gray-300 hover:text-white transition-colors';
            }
        });
        this.addDebugLog('control', `Switched timeframe to: ${this.state.timeframe}`);
        this.fetchInitialSnapshot();
    }

    setLimit(lim) {
        this.state.limit = lim;
        this.el.limitButtons.forEach(btn => {
            if (parseInt(btn.getAttribute('data-limit'), 10) === lim) {
                btn.className = 'limit-btn px-2 py-1 rounded text-xs font-semibold bg-blue-600 text-white shadow-sm transition-colors';
            } else {
                btn.className = 'limit-btn px-2 py-1 rounded text-xs font-semibold text-gray-300 hover:text-white transition-colors';
            }
        });
        this.addDebugLog('control', `Set candle limit to: ${this.state.limit}`);
        this.fetchInitialSnapshot();
    }

    showLoading(show) {
        this.state.isLoading = show;
        if (this.el.loadingOverlay) {
            this.el.loadingOverlay.style.display = show ? 'flex' : 'none';
        }
    }

    showError(msg) {
        if (this.el.errorMessage && this.el.errorText) {
            if (msg) {
                this.el.errorText.textContent = msg;
                this.el.errorMessage.classList.remove('hidden');
            } else {
                this.el.errorMessage.classList.add('hidden');
            }
        }
    }

    async fetchInitialSnapshot() {
        this.showLoading(true);
        this.showError(null);

        try {
            const { symbol, timeframe, limit } = this.state;
            this.addDebugLog('rest', `Fetching initial REST snapshot for ${symbol} (${timeframe}, limit=${limit})...`);

            const [spotRes, futRes, oiRes, lsRes, tickers] = await Promise.all([
                this.apiClient.getSpotKlines(symbol, timeframe, limit),
                this.apiClient.getFuturesKlines(symbol, timeframe, limit),
                this.apiClient.getOpenInterestHist(symbol, timeframe, limit).catch(() => ({ data: [], latency: 0 })),
                this.apiClient.getTopLongShortPositionRatio(symbol, timeframe, limit)
                    .catch(() => this.apiClient.getGlobalLongShortAccountRatio(symbol, timeframe, limit))
                    .catch(() => ({ data: [], latency: 0 })),
                this.apiClient.get24hTickers(symbol)
            ]);

            this.state.rawSpotKlines = spotRes.data || [];
            this.state.rawFutKlines = futRes.data || [];
            this.state.rawOiHistory = oiRes.data || [];
            this.state.rawLsHistory = lsRes.data || [];
            this.state.latestTickers = tickers || {};

            this.addDebugLog('rest', `✅ Snapshot loaded: ${this.state.rawSpotKlines.length} Spot klines, ${this.state.rawFutKlines.length} Fut klines`);

            this.executeFullProcessAndRender();

            if (this.state.autoRefresh) {
                this.wsManager.subscribe(symbol, timeframe);
            }

        } catch (err) {
            this.addDebugLog('error', `Failed to fetch snapshot: ${err.message}`);
            const isVi = (document.documentElement.lang || 'vi') === 'vi';
            this.showError((isVi ? 'Không thể tải dữ liệu snapshot: ' : 'Failed to fetch snapshot data: ') + err.message);
        } finally {
            this.showLoading(false);
        }
    }

    startBackgroundSync() {
        this.stopBackgroundSync();
        this.state.bgSyncId = setInterval(async () => {
            if (!this.state.autoRefresh || this.isTabHidden) return;
            try {
                const { symbol, timeframe, limit } = this.state;
                const [oiRes, lsRes, tickers] = await Promise.all([
                    this.apiClient.getOpenInterestHist(symbol, timeframe, limit).catch(() => ({ data: [] })),
                    this.apiClient.getTopLongShortPositionRatio(symbol, timeframe, limit)
                        .catch(() => this.apiClient.getGlobalLongShortAccountRatio(symbol, timeframe, limit))
                        .catch(() => ({ data: [] })),
                    this.apiClient.get24hTickers(symbol).catch(() => ({}))
                ]);

                if (oiRes.data && oiRes.data.length) this.state.rawOiHistory = oiRes.data;
                if (lsRes.data && lsRes.data.length) this.state.rawLsHistory = lsRes.data;
                if (tickers.currentOi) this.state.latestTickers.currentOi = tickers.currentOi;
                if (tickers.spot) this.state.latestTickers.spot = tickers.spot;
                if (tickers.futures) this.state.latestTickers.futures = tickers.futures;

                this.addDebugLog('sync', `🔄 Background sync refreshed OI & LS Ratios`);
                this.scheduleFullUpdate();
            } catch (e) {
                this.addDebugLog('sync', `⚠️ Background sync warning: ${e.message}`);
            }
        }, this.state.bgSyncIntervalSec * 1000);
    }

    stopBackgroundSync() {
        if (this.state.bgSyncId) {
            clearInterval(this.state.bgSyncId);
            this.state.bgSyncId = null;
        }
    }

    /**
     * In-place kline array update
     * @returns {boolean} isNewCandle
     */
    updateKlineArray(arr, k) {
        const openTime = parseInt(k.t, 10);
        const formatted = [
            openTime,
            k.o,
            k.h,
            k.l,
            k.c,
            k.v,
            parseInt(k.T, 10),
            k.q,
            parseInt(k.n, 10) || 0,
            k.V,
            k.Q
        ];

        if (arr.length === 0) {
            arr.push(formatted);
            return true;
        }

        const last = arr[arr.length - 1];
        const lastOpenTime = parseInt(last[0], 10);

        if (lastOpenTime === openTime) {
            arr[arr.length - 1] = formatted;
            return false; // Existing candle updated
        } else if (openTime > lastOpenTime) {
            arr.push(formatted);
            if (arr.length > this.state.limit * 1.5) arr.shift();
            return true; // Brand new candle opened
        } else {
            const idx = arr.findIndex(item => parseInt(item[0], 10) === openTime);
            if (idx !== -1) arr[idx] = formatted;
            return false;
        }
    }

    handleSpotKlineTick(k) {
        const isNew = this.updateKlineArray(this.state.rawSpotKlines, k);
        if (this.el.dbgSpotLast) {
            this.el.dbgSpotLast.textContent = `$${formatPrice(k.c)} (vol: ${formatNumber(k.q)})`;
        }
        if (isNew) {
            this.scheduleFullUpdate();
        } else {
            this.scheduleIncrementalTickUpdate();
        }
    }

    handleFuturesKlineTick(k) {
        const isNew = this.updateKlineArray(this.state.rawFutKlines, k);
        if (this.el.dbgFutLast) {
            this.el.dbgFutLast.textContent = `$${formatPrice(k.c)} (vol: ${formatNumber(k.q)})`;
        }
        if (isNew) {
            this.scheduleFullUpdate();
        } else {
            this.scheduleIncrementalTickUpdate();
        }
    }

    handleSpotTickerTick(data) {
        if (!this.state.latestTickers.spot) this.state.latestTickers.spot = {};
        if (data.q) this.state.latestTickers.spot.quoteVolume = data.q;
        if (data.c) this.state.latestTickers.spot.lastPrice = data.c;
        this.scheduleIncrementalTickUpdate();
    }

    handleFuturesTickerTick(data) {
        if (!this.state.latestTickers.futures) this.state.latestTickers.futures = {};
        if (data.q) this.state.latestTickers.futures.quoteVolume = data.q;
        if (data.c) this.state.latestTickers.futures.lastPrice = data.c;
        this.scheduleIncrementalTickUpdate();
    }

    simulateTestTick() {
        if (!this.state.rawFutKlines.length) {
            this.addDebugLog('debug', 'Cannot simulate: no klines in memory');
            return;
        }
        const lastFut = this.state.rawFutKlines[this.state.rawFutKlines.length - 1];
        const currentClose = parseFloat(lastFut[4]);
        const delta = (Math.random() - 0.48) * (currentClose * 0.002);
        const newClose = (currentClose + delta).toFixed(2);

        const simKline = {
            t: lastFut[0],
            T: lastFut[6],
            o: lastFut[1],
            h: Math.max(parseFloat(lastFut[2]), parseFloat(newClose)).toFixed(2),
            l: Math.min(parseFloat(lastFut[3]), parseFloat(newClose)).toFixed(2),
            c: newClose,
            v: (parseFloat(lastFut[5]) + Math.random() * 5).toFixed(4),
            q: (parseFloat(lastFut[7]) + Math.random() * 50000).toFixed(2),
            n: (parseInt(lastFut[8], 10) + 10),
            V: (parseFloat(lastFut[9]) + Math.random() * 3).toFixed(4),
            Q: (parseFloat(lastFut[10]) + Math.random() * 30000).toFixed(2),
            x: false
        };

        this.addDebugLog('debug', `⚡ Simulating tick: close=$${newClose}`);
        this.handleFuturesKlineTick(simKline);
        this.handleSpotKlineTick(simKline);
    }

    /**
     * High-speed frame scheduled incremental tick update (60 FPS coalesced)
     */
    scheduleIncrementalTickUpdate() {
        if (this.isTabHidden) return;
        if (this.rafPending) return;

        this.rafPending = true;
        this.rafId = requestAnimationFrame(() => {
            this.rafPending = false;
            this.rafId = null;
            this.executeIncrementalTickRender();
        });
    }

    scheduleFullUpdate() {
        if (this.isTabHidden) return;
        if (this.rafPending) return;

        this.rafPending = true;
        this.rafId = requestAnimationFrame(() => {
            this.rafPending = false;
            this.rafId = null;
            this.executeFullProcessAndRender();
        });
    }

    /**
     * Fast incremental tick render: O(1) computation + targeted 1-row table update + in-place chart data mutation
     */
    executeIncrementalTickRender() {
        if (!this.state.currentData || !this.state.rawSpotKlines.length || !this.state.rawFutKlines.length) {
            this.executeFullProcessAndRender();
            return;
        }

        const startTime = performance.now();

        // 1. Incremental Analytical Update
        AnalyticsEngine.updateLatestCandle(
            this.state.currentData,
            this.state.rawSpotKlines,
            this.state.rawFutKlines,
            this.state.timeframe
        );
        this.state.currentData.tickers = this.state.latestTickers;

        // 2. Targeted Component Updates
        this.state.renderCount++;
        this.updateKpiDisplays(this.state.currentData);
        this.chartManager.updateLatestTick(this.state.currentData);

        const items = this.state.currentData.items;
        if (items.length > 0) {
            this.updateTopTableRow(items[items.length - 1]);
        }

        const duration = Math.round((performance.now() - startTime) * 100) / 100;
        if (this.el.dbgLastRenderTime) {
            const now = new Date();
            this.el.dbgLastRenderTime.textContent = `${now.toLocaleTimeString()} (${duration}ms [tick])`;
        }
        this.updateDebugStats();
    }

    /**
     * Full dataset render: recalculates all items & re-renders full table and all charts
     */
    executeFullProcessAndRender() {
        if (!this.state.rawSpotKlines.length || !this.state.rawFutKlines.length) return;

        const startTime = performance.now();
        const processed = AnalyticsEngine.processAndAlignData(
            this.state.rawSpotKlines,
            this.state.rawFutKlines,
            this.state.rawOiHistory,
            this.state.rawLsHistory,
            this.state.timeframe
        );
        processed.tickers = this.state.latestTickers;
        this.state.currentData = processed;

        this.state.renderCount++;
        this.updateKpiDisplays(processed);
        this.chartManager.renderAll(processed);
        this.renderTable(processed.items);

        const duration = Math.round(performance.now() - startTime);
        if (this.el.dbgLastRenderTime) {
            const now = new Date();
            this.el.dbgLastRenderTime.textContent = `${now.toLocaleTimeString()} (${duration}ms [full])`;
        }
        this.updateDebugStats();
    }

    updateKpiDisplays(data) {
        const tickers = data.tickers || {};
        const items = data.items;
        const lastItem = items.length > 0 ? items[items.length - 1] : null;

        if (this.el.analyticsUpdatedAt) {
            const now = new Date();
            const pad = n => String(n).padStart(2, '0');
            this.el.analyticsUpdatedAt.textContent = `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())} - ${pad(now.getDate())}/${pad(now.getMonth() + 1)}/${now.getFullYear()}`;
        }

        let spot24hVol = 0;
        let spotBuy24h = 0;
        let spotSell24h = 0;
        const isVi = (document.documentElement.lang || 'vi') === 'vi';
        const buyText = isVi ? 'Mua' : 'Buy';
        const sellText = isVi ? 'Bán' : 'Sell';
        const bsRatioText = isVi ? 'Tỷ lệ Mua/Bán' : 'Buy/Sell Ratio';

        if (this.el.kpiSpotVol) {
            spot24hVol = (tickers.spot && tickers.spot.quoteVolume) ? parseFloat(tickers.spot.quoteVolume) : (lastItem ? lastItem.spotVolumeUsdt * 24 : 0);
            this.el.kpiSpotVol.textContent = '$' + formatNumber(spot24hVol);

            const spotBuyPct = lastItem ? (lastItem.spotBuyPct / 100) : 0.5;
            spotBuy24h = spot24hVol * spotBuyPct;
            spotSell24h = Math.max(0, spot24hVol - spotBuy24h);

            if (this.el.kpiSpotBuySell) {
                this.el.kpiSpotBuySell.innerHTML = `${buyText}: <span class="font-bold text-cyan-400">$${formatNumber(spotBuy24h)}</span> | ${sellText}: <span class="font-bold text-blue-400">$${formatNumber(spotSell24h)}</span>`;
            }
        }

        let fut24hVol = 0;
        let futBuy24h = 0;
        let futSell24h = 0;
        if (this.el.kpiFutVol) {
            fut24hVol = (tickers.futures && tickers.futures.quoteVolume) ? parseFloat(tickers.futures.quoteVolume) : (lastItem ? lastItem.futuresVolumeUsdt * 24 : 0);
            this.el.kpiFutVol.textContent = '$' + formatNumber(fut24hVol);

            const futBuyPct = lastItem ? (lastItem.futuresBuyPct / 100) : 0.5;
            futBuy24h = fut24hVol * futBuyPct;
            futSell24h = Math.max(0, fut24hVol - futBuy24h);

            if (this.el.kpiFutBuySell) {
                this.el.kpiFutBuySell.innerHTML = `${buyText}: <span class="font-bold text-emerald-400">$${formatNumber(futBuy24h)}</span> | ${sellText}: <span class="font-bold text-rose-400">$${formatNumber(futSell24h)}</span>`;
            }
        }

        if (this.el.kpiFsRatio) {
            const spotVol = spot24hVol > 0 ? spot24hVol : 1;
            const futVol = fut24hVol > 0 ? fut24hVol : 1;
            const ratio = spotVol > 0 ? (futVol / spotVol) : 0;
            this.el.kpiFsRatio.textContent = ratio.toFixed(2) + 'x';
        }
        if (this.el.kpiBsRatio) {
            const futBsRatio = futSell24h > 0 ? (futBuy24h / futSell24h).toFixed(2) : '--';
            const spotBsRatio = spotSell24h > 0 ? (spotBuy24h / spotSell24h).toFixed(2) : '--';
            this.el.kpiBsRatio.textContent = `${bsRatioText}: ${futBsRatio} (Fut) | ${spotBsRatio} (Spot)`;
        }

        let totalOiUsdt = 0;
        if (this.el.kpiOiUsdt && this.el.kpiOiCoins) {
            if (tickers.currentOi && lastItem) {
                const oiCoins = parseFloat(tickers.currentOi.openInterest);
                totalOiUsdt = oiCoins * lastItem.price;
                this.el.kpiOiUsdt.textContent = '$' + formatNumber(totalOiUsdt);
                this.el.kpiOiCoins.textContent = `${formatNumber(oiCoins)} ${this.state.symbol.replace('USDT', '')}`;
            } else if (lastItem && lastItem.openInterestUsdt) {
                totalOiUsdt = lastItem.openInterestUsdt;
                this.el.kpiOiUsdt.textContent = '$' + formatNumber(lastItem.openInterestUsdt);
                this.el.kpiOiCoins.textContent = `${formatNumber(lastItem.openInterestCoins)} ${this.state.symbol.replace('USDT', '')}`;
            } else {
                this.el.kpiOiUsdt.textContent = '--';
                this.el.kpiOiCoins.textContent = '--';
            }
        }

        if (this.el.kpiLongPos && this.el.kpiLongDesc) {
            if (lastItem) {
                const longRatio = lastItem.longRatio;
                const oiVal = totalOiUsdt > 0 ? totalOiUsdt : (lastItem.openInterestUsdt || 0);
                const longPosVal = oiVal * longRatio;
                const longSpotRatio = lastItem.longSpotRatio ? lastItem.longSpotRatio.toFixed(2) : '--';
                const longPct = (longRatio * 100).toFixed(1);

                this.el.kpiLongPos.textContent = '$' + formatNumber(longPosVal);
                this.el.kpiLongDesc.textContent = `${longPct}% OI | vs Spot: ${longSpotRatio}x`;
            } else {
                this.el.kpiLongPos.textContent = '--';
                this.el.kpiLongDesc.textContent = '--% OI | vs Spot: --x';
            }
        }

        if (this.el.kpiShortPos && this.el.kpiShortDesc) {
            if (lastItem) {
                const shortRatio = lastItem.shortRatio;
                const oiVal = totalOiUsdt > 0 ? totalOiUsdt : (lastItem.openInterestUsdt || 0);
                const shortPosVal = oiVal * shortRatio;
                const shortSpotRatio = lastItem.shortSpotRatio ? lastItem.shortSpotRatio.toFixed(2) : '--';
                const shortPct = (shortRatio * 100).toFixed(1);

                this.el.kpiShortPos.textContent = '$' + formatNumber(shortPosVal);
                this.el.kpiShortDesc.textContent = `${shortPct}% OI | vs Spot: ${shortSpotRatio}x`;
            } else {
                this.el.kpiShortPos.textContent = '--';
                this.el.kpiShortDesc.textContent = '--% OI | vs Spot: --x';
            }
        }

        if (this.el.kpiCorrelation && this.el.kpiCorrelationStatus) {
            const corr = data.overallCorrelation;
            this.el.kpiCorrelation.textContent = corr.toFixed(3);

            if (corr >= 0.7) {
                this.el.kpiCorrelationStatus.textContent = isVi ? 'Đồng thuận cao (Thị trường thực)' : 'High Alignment (Organic)';
                this.el.kpiCorrelationStatus.className = 'text-xs font-semibold text-emerald-500';
            } else if (corr <= 0.3) {
                this.el.kpiCorrelationStatus.textContent = isVi ? 'Phân kỳ / Đầu cơ phái sinh' : 'Divergence (Speculative)';
                this.el.kpiCorrelationStatus.className = 'text-xs font-semibold text-amber-500';
            } else {
                this.el.kpiCorrelationStatus.textContent = isVi ? 'Tương quan trung bình' : 'Moderate Correlation';
                this.el.kpiCorrelationStatus.className = 'text-xs font-semibold text-blue-500';
            }
        }

        if (this.el.kpiMarketStateBadge && this.el.kpiMarketStateDesc && lastItem) {
            const state = lastItem.marketState;

            switch (state) {
                case 'LONG_BUILDUP':
                    this.el.kpiMarketStateBadge.textContent = '🟢 Long Build-up';
                    this.el.kpiMarketStateBadge.className = 'text-lg font-bold text-emerald-500';
                    this.el.kpiMarketStateDesc.textContent = isVi ? 'Giá tăng + Vị thế mở tăng: Dòng tiền mở Long áp đảo' : 'Price ↑ + OI ↑: Aggressive Long build-up';
                    break;
                case 'SHORT_SQUEEZE':
                    this.el.kpiMarketStateBadge.textContent = '🟡 Short Squeeze';
                    this.el.kpiMarketStateBadge.className = 'text-lg font-bold text-yellow-500';
                    this.el.kpiMarketStateDesc.textContent = isVi ? 'Giá tăng + Vị thế mở giảm: Tăng do Short bị thanh lý/cắt lỗ' : 'Price ↑ + OI ↓: Rally driven by Short covering';
                    break;
                case 'SHORT_BUILDUP':
                    this.el.kpiMarketStateBadge.textContent = '🔴 Short Build-up';
                    this.el.kpiMarketStateBadge.className = 'text-lg font-bold text-rose-500';
                    this.el.kpiMarketStateDesc.textContent = isVi ? 'Giá giảm + Vị thế mở tăng: Dòng tiền mở Short áp đảo' : 'Price ↓ + OI ↑: Aggressive Short build-up';
                    break;
                case 'LONG_LIQUIDATION':
                    this.el.kpiMarketStateBadge.textContent = '⚪ Long Liquidation';
                    this.el.kpiMarketStateBadge.className = 'text-lg font-bold text-indigo-500';
                    this.el.kpiMarketStateDesc.textContent = isVi ? 'Giá giảm + Vị thế mở giảm: Long bị thanh lý / rời bỏ vị thế' : 'Price ↓ + OI ↓: Long capitulation & liquidation';
                    break;
                default:
                    this.el.kpiMarketStateBadge.textContent = '⚪ Neutral';
                    this.el.kpiMarketStateBadge.className = 'text-lg font-bold text-gray-500';
                    this.el.kpiMarketStateDesc.textContent = isVi ? 'Thị trường cân bằng hoặc chưa đủ nến để xác định' : 'Balanced market state';
            }
        }
    }

    initColumnVisibility() {
        let saved = null;
        try {
            const stored = localStorage.getItem('market_analytics_visible_columns');
            if (stored) saved = JSON.parse(stored);
        } catch (e) {
            saved = null;
        }

        if (Array.isArray(saved) && saved.length > 0) {
            this.state.visibleColumns = new Set(saved);
        } else {
            this.state.visibleColumns = new Set(PRESETS.standard);
        }
        this.state.visibleColumns.add('time');

        try {
            const storedLayout = localStorage.getItem('market_analytics_table_layout');
            if (storedLayout === 'stacked' || storedLayout === 'expanded') {
                this.state.tableLayout = storedLayout;
            } else {
                this.state.tableLayout = 'stacked';
            }
        } catch (e) {
            this.state.tableLayout = 'stacked';
        }

        this.updateLayoutButtonsUI();
        this.updatePresetButtonsUI();
        this.renderColumnCheckboxes();
        this.updateColumnsBadge();
    }

    saveColumnVisibility() {
        try {
            const arr = Array.from(this.state.visibleColumns);
            localStorage.setItem('market_analytics_visible_columns', JSON.stringify(arr));
        } catch (e) {}
    }

    updateLayoutButtonsUI() {
        if (!this.el.layoutButtons) return;
        const currentLayout = this.state.tableLayout || 'stacked';
        this.el.layoutButtons.forEach(btn => {
            const l = btn.getAttribute('data-layout');
            if (l === currentLayout) {
                btn.className = 'btn-table-layout px-2.5 py-1 text-xs font-bold rounded-md bg-blue-600 text-white shadow-sm transition-all flex items-center space-x-1.5';
            } else {
                btn.className = 'btn-table-layout px-2.5 py-1 text-xs font-semibold rounded-md text-gray-700 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-300 dark:hover:text-white dark:hover:bg-gray-700/50 transition-all flex items-center space-x-1.5';
            }
        });

        const isStacked = currentLayout === 'stacked';
        if (this.el.wrapperColumnCustomizer) {
            if (isStacked) {
                this.el.wrapperColumnCustomizer.classList.add('hidden');
                if (this.el.columnCustomizerDropdown) {
                    this.el.columnCustomizerDropdown.classList.add('hidden');
                }
            } else {
                this.el.wrapperColumnCustomizer.classList.remove('hidden');
            }
        }

        if (this.el.wrapperTablePresets) {
            if (isStacked) {
                this.el.wrapperTablePresets.classList.add('hidden');
            } else {
                this.el.wrapperTablePresets.classList.remove('hidden');
            }
        }
    }

    updatePresetButtonsUI() {
        if (!this.el.presetButtons) return;
        const currentSet = this.state.visibleColumns;

        let activePreset = null;
        for (const [presetKey, colList] of Object.entries(PRESETS)) {
            if (colList.length === currentSet.size && colList.every(id => currentSet.has(id))) {
                activePreset = presetKey;
                break;
            }
        }

        const effectivePreset = activePreset || 'standard';

        this.el.presetButtons.forEach(btn => {
            const p = btn.getAttribute('data-preset');
            if (p === effectivePreset && this.state.tableLayout === 'expanded') {
                btn.className = 'btn-table-preset px-2.5 py-1 rounded-md text-xs font-semibold transition-all bg-indigo-600 text-white shadow-sm';
            } else {
                btn.className = 'btn-table-preset px-2.5 py-1 rounded-md text-xs font-semibold transition-all bg-gray-200/80 hover:bg-gray-300 text-gray-700 dark:bg-gray-700/60 dark:hover:bg-gray-700 dark:text-gray-300';
            }
        });
    }

    updateColumnsBadge() {
        if (this.el.badgeColumnsCount) {
            this.el.badgeColumnsCount.textContent = `${this.state.visibleColumns.size}/${TABLE_COLUMNS.length}`;
        }
    }

    renderColumnCheckboxes() {
        if (!this.el.columnCheckboxesContainer) return;

        const groups = [
            { id: 'price', titleI18n: 'column-group-price', titleDefault: 'Giá & Nến', icon: 'fa-chart-line text-blue-600 dark:text-blue-400' },
            { id: 'flow', titleI18n: 'column-group-flow', titleDefault: 'Khối Lượng & Dòng Tiền Taker', icon: 'fa-water text-teal-600 dark:text-cyan-400' },
            { id: 'positions', titleI18n: 'column-group-positions', titleDefault: 'Vị Thế & Đòn Bẩy', icon: 'fa-scale-balanced text-emerald-600 dark:text-emerald-400' },
            { id: 'market', titleI18n: 'column-group-market', titleDefault: 'OI & Trạng Thái Thị Trường', icon: 'fa-compass text-fuchsia-600 dark:text-pink-400' }
        ];

        let html = '';
        groups.forEach(g => {
            const groupCols = TABLE_COLUMNS.filter(c => c.groupId === g.id);
            if (!groupCols.length) return;

            const groupTitle = getI18nText(g.titleI18n, g.titleDefault);
            html += `
                <div class="mb-3">
                    <div class="flex items-center space-x-1.5 pb-1 mb-1.5 border-b border-gray-200 dark:border-gray-700/40 text-[11px] font-bold text-gray-800 dark:text-gray-300">
                        <i class="fas ${g.icon} text-xs"></i>
                        <span data-i18n="${g.titleI18n}">${groupTitle}</span>
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-1.5">
            `;

            groupCols.forEach(col => {
                const isChecked = this.state.visibleColumns.has(col.id);
                const label = getI18nText(col.labelI18n, col.labelDefault);
                const isDisabled = col.fixed ? 'disabled cursor-not-allowed opacity-75' : 'cursor-pointer';

                html += `
                    <label class="flex items-center space-x-2 p-1.5 rounded hover:bg-gray-100 dark:hover:bg-white/5 transition-colors ${isDisabled}">
                        <input type="checkbox" class="col-toggle-checkbox form-checkbox h-3.5 w-3.5 text-indigo-600 rounded bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 focus:ring-indigo-500"
                            data-col-id="${col.id}" ${isChecked ? 'checked' : ''} ${col.fixed ? 'disabled' : ''}>
                        <span class="text-xs text-gray-800 dark:text-gray-200 select-none font-medium" data-i18n="${col.labelI18n}">${label}</span>
                    </label>
                `;
            });

            html += `
                    </div>
                </div>
            `;
        });

        this.el.columnCheckboxesContainer.innerHTML = html;

        this.el.columnCheckboxesContainer.querySelectorAll('.col-toggle-checkbox').forEach(chk => {
            chk.addEventListener('change', (e) => {
                const colId = e.target.getAttribute('data-col-id');
                if (!colId) return;

                if (e.target.checked) {
                    this.state.visibleColumns.add(colId);
                } else {
                    this.state.visibleColumns.delete(colId);
                }
                this.state.visibleColumns.add('time');

                if (this.state.tableLayout === 'stacked') {
                    this.state.tableLayout = 'expanded';
                    try {
                        localStorage.setItem('market_analytics_table_layout', 'expanded');
                    } catch (err) {}
                    this.updateLayoutButtonsUI();
                }

                this.saveColumnVisibility();
                this.updatePresetButtonsUI();
                this.updateColumnsBadge();
                if (this.state.currentData) {
                    this.renderTable(this.state.currentData.items);
                }
            });
        });
    }

    /**
     * Targeted update of top table row (0.05ms) for live streaming ticks
     */
    updateTopTableRow(latestItem) {
        if (!this.el.dataTableBody || !this.el.dataTableBody.firstElementChild) {
            if (this.state.currentData) this.renderTable(this.state.currentData.items);
            return;
        }

        const topRow = this.el.dataTableBody.firstElementChild;
        const isStacked = (this.state.tableLayout || 'stacked') === 'stacked';

        let cellsHtml = '';
        if (isStacked) {
            STACKED_COLUMNS.forEach(col => {
                cellsHtml += col.render(latestItem);
            });
        } else {
            const activeCols = TABLE_COLUMNS.filter(c => this.state.visibleColumns.has(c.id));
            activeCols.forEach(col => {
                cellsHtml += col.render(latestItem);
            });
        }

        topRow.innerHTML = cellsHtml;
    }

    renderTable(items) {
        if (!this.el.dataTableBody) return;

        const isStacked = (this.state.tableLayout || 'stacked') === 'stacked';

        // 1. Render Table Header
        if (this.el.dataTableHead) {
            let headHtml = '<tr style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color);">';
            if (isStacked) {
                STACKED_COLUMNS.forEach(col => {
                    const colLabel = getI18nText(col.labelI18n, col.labelDefault);
                    headHtml += `<th class="${col.thClass}" data-i18n="${col.labelI18n}">${colLabel}</th>`;
                });
            } else {
                const activeCols = TABLE_COLUMNS.filter(c => this.state.visibleColumns.has(c.id));
                activeCols.forEach(col => {
                    const colLabel = getI18nText(col.labelI18n, col.labelDefault);
                    headHtml += `<th class="${col.thClass}" data-i18n="${col.labelI18n}">${colLabel}</th>`;
                });
            }
            headHtml += '</tr>';
            this.el.dataTableHead.innerHTML = headHtml;
        }

        if (!items || !items.length) {
            const colSpan = isStacked ? STACKED_COLUMNS.length : 20;
            this.el.dataTableBody.innerHTML = `<tr><td colspan="${colSpan}" class="text-center py-6 text-gray-500 font-medium">No data available</td></tr>`;
            return;
        }

        // 2. Limit rows based on tableLimit (default 30)
        const limit = this.state.tableLimit || 30;
        const displayItems = items.slice().reverse().slice(0, limit);

        let bodyHtml = '';
        const activeCols = isStacked ? null : TABLE_COLUMNS.filter(c => this.state.visibleColumns.has(c.id));

        displayItems.forEach((row, idx) => {
            bodyHtml += `<tr id="tr-candle-${idx}" class="hover:bg-slate-100/70 dark:hover:bg-black/20 transition-all duration-200" style="border-bottom: 1px solid var(--border-color);">`;
            if (isStacked) {
                STACKED_COLUMNS.forEach(col => {
                    bodyHtml += col.render(row);
                });
            } else {
                activeCols.forEach(col => {
                    bodyHtml += col.render(row);
                });
            }
            bodyHtml += '</tr>';
        });

        this.el.dataTableBody.innerHTML = bodyHtml;
    }

    exportCsv() {
        if (!this.state.currentData || !this.state.currentData.items.length) {
            const isVi = (document.documentElement.lang || 'vi') === 'vi';
            alert(isVi ? 'Không có dữ liệu để xuất CSV!' : 'No data to export!');
            return;
        }

        const items = this.state.currentData.items;
        const headers = [
            'Timestamp',
            'DateTime',
            'Futures_Open_USDT',
            'Futures_High_USDT',
            'Futures_Low_USDT',
            'Futures_Close_USDT',
            'Price_Change_Pct',
            'Futures_BaseVolume_Coins',
            'Futures_QuoteVolume_USDT',
            'Futures_TakerBuyVolume_USDT',
            'Futures_TakerSellVolume_USDT',
            'Futures_BuyRatio',
            'Futures_BuyPct',
            'Futures_NetDelta_USDT',
            'Futures_Cumulative_CVD_USDT',
            'Futures_TradesCount',
            'Spot_Open_USDT',
            'Spot_High_USDT',
            'Spot_Low_USDT',
            'Spot_Close_USDT',
            'Spot_BaseVolume_Coins',
            'Spot_QuoteVolume_USDT',
            'Spot_TakerBuyVolume_USDT',
            'Spot_TakerSellVolume_USDT',
            'Spot_BuyRatio',
            'Spot_BuyPct',
            'Spot_NetDelta_USDT',
            'Spot_Cumulative_CVD_USDT',
            'Spot_TradesCount',
            'Volume_Delta_Fut_Minus_Spot_USDT',
            'Futures_Spot_Volume_Ratio',
            'OpenInterest_USDT',
            'OpenInterest_Coins',
            'OpenInterest_Change_Pct',
            'TopTrader_Long_Account_Ratio',
            'TopTrader_Short_Account_Ratio',
            'TopTrader_LongShort_Ratio',
            'Estimated_Long_Position_USDT',
            'Estimated_Short_Position_USDT',
            'Estimated_Net_Position_USDT',
            'Long_Spot_Ratio',
            'Short_Spot_Ratio',
            'Correlation_14_Spot_Fut',
            'Market_State'
        ];

        const rows = items.map(d => [
            d.timestamp,
            `"${d.label}"`,
            d.open !== undefined ? d.open : '',
            d.high !== undefined ? d.high : '',
            d.low !== undefined ? d.low : '',
            d.close !== undefined ? d.close : d.price,
            d.priceChangePct !== undefined ? d.priceChangePct.toFixed(2) : '0.00',
            d.futuresBaseVolume !== undefined ? d.futuresBaseVolume.toFixed(4) : '',
            d.futuresVolumeUsdt !== undefined ? d.futuresVolumeUsdt.toFixed(2) : '0.00',
            d.futuresBuyVolumeUsdt !== undefined ? d.futuresBuyVolumeUsdt.toFixed(2) : '0.00',
            d.futuresSellVolumeUsdt !== undefined ? d.futuresSellVolumeUsdt.toFixed(2) : '0.00',
            d.futuresBuyRatio !== undefined ? d.futuresBuyRatio.toFixed(2) : '1.00',
            d.futuresBuyPct !== undefined ? d.futuresBuyPct.toFixed(2) : '50.00',
            d.futuresNetDelta !== undefined ? d.futuresNetDelta.toFixed(2) : '0.00',
            d.futuresCvd !== undefined ? d.futuresCvd.toFixed(2) : '0.00',
            d.tradesCount !== undefined ? d.tradesCount : '',
            d.spotOpen !== undefined ? d.spotOpen : '',
            d.spotHigh !== undefined ? d.spotHigh : '',
            d.spotLow !== undefined ? d.spotLow : '',
            d.spotClose !== undefined ? d.spotClose : '',
            d.spotBaseVolume !== undefined ? d.spotBaseVolume.toFixed(4) : '',
            d.spotVolumeUsdt !== undefined ? d.spotVolumeUsdt.toFixed(2) : '0.00',
            d.spotBuyVolumeUsdt !== undefined ? d.spotBuyVolumeUsdt.toFixed(2) : '0.00',
            d.spotSellVolumeUsdt !== undefined ? d.spotSellVolumeUsdt.toFixed(2) : '0.00',
            d.spotBuyRatio !== undefined ? d.spotBuyRatio.toFixed(2) : '1.00',
            d.spotBuyPct !== undefined ? d.spotBuyPct.toFixed(2) : '50.00',
            d.spotNetDelta !== undefined ? d.spotNetDelta.toFixed(2) : '0.00',
            d.spotCvd !== undefined ? d.spotCvd.toFixed(2) : '0.00',
            d.spotTrades !== undefined ? d.spotTrades : '',
            d.volumeDelta !== undefined ? d.volumeDelta.toFixed(2) : '0.00',
            d.volumeRatio !== undefined ? d.volumeRatio.toFixed(2) : '0.00',
            d.openInterestUsdt !== null && d.openInterestUsdt !== undefined ? d.openInterestUsdt.toFixed(2) : '',
            d.openInterestCoins !== null && d.openInterestCoins !== undefined ? d.openInterestCoins.toFixed(4) : '',
            d.oiChangePct !== null && d.oiChangePct !== undefined ? d.oiChangePct.toFixed(2) : '',
            d.longRatio !== undefined ? d.longRatio.toFixed(4) : '',
            d.shortRatio !== undefined ? d.shortRatio.toFixed(4) : '',
            d.longShortRatio !== null && d.longShortRatio !== undefined ? d.longShortRatio.toFixed(2) : '',
            d.longPositionUsdt !== null && d.longPositionUsdt !== undefined ? d.longPositionUsdt.toFixed(2) : '',
            d.shortPositionUsdt !== null && d.shortPositionUsdt !== undefined ? d.shortPositionUsdt.toFixed(2) : '',
            d.netPositionUsdt !== null && d.netPositionUsdt !== undefined ? d.netPositionUsdt.toFixed(2) : '',
            d.longSpotRatio !== null && d.longSpotRatio !== undefined ? d.longSpotRatio.toFixed(2) : '',
            d.shortSpotRatio !== null && d.shortSpotRatio !== undefined ? d.shortSpotRatio.toFixed(2) : '',
            d.correlation !== null && d.correlation !== undefined ? d.correlation.toFixed(3) : '',
            `"${d.marketState || 'NEUTRAL'}"`
        ]);

        const csvString = '\uFEFF' + [headers.join(','), ...rows.map(e => e.join(','))].join('\r\n');
        const blob = new Blob([csvString], { type: 'text/csv;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.setAttribute('href', url);
        link.setAttribute('download', `${this.state.symbol}_${this.state.timeframe}_full_market_analytics.csv`);
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
    }

    /**
     * Complete lifecycle disposal & memory release
     */
    destroy() {
        // 1. Cancel background polling interval
        this.stopBackgroundSync();

        // 2. Cancel pending frame animation
        if (this.rafId) {
            cancelAnimationFrame(this.rafId);
            this.rafId = null;
        }
        this.rafPending = false;

        // 3. Disconnect WebSocket and clear reconnect timers
        if (this.wsManager) {
            this.wsManager.destroy();
        }

        // 4. Destroy all 8 Chart.js instances and disconnect IntersectionObserver
        if (this.chartManager) {
            this.chartManager.destroyAll();
        }

        // 5. Disconnect MutationObserver
        if (this.themeObserver) {
            this.themeObserver.disconnect();
            this.themeObserver = null;
        }

        // 6. Release in-memory data structures
        this.state.rawSpotKlines = [];
        this.state.rawFutKlines = [];
        this.state.rawOiHistory = [];
        this.state.rawLsHistory = [];
        this.state.currentData = null;
        this.debugLogBuffer = [];
    }
}

