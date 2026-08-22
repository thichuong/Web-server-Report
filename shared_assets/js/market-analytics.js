/**
 * Market Analytics Client-side Engine
 * Pure client-side fetching from Binance Spot & Futures APIs,
 * mathematical calculation of Volume Correlation, Volume Delta, Futures/Spot Ratio,
 * Open Interest analysis, CVD, and Chart.js rendering.
 */

(function () {
    'use strict';

    // Helper to format currency and numbers
    const formatNumber = (num, decimals = 2) => {
        if (num === null || num === undefined || isNaN(num)) return '--';
        if (Math.abs(num) >= 1e9) return (num / 1e9).toFixed(decimals) + 'B';
        if (Math.abs(num) >= 1e6) return (num / 1e6).toFixed(decimals) + 'M';
        if (Math.abs(num) >= 1e3) return (num / 1e3).toFixed(decimals) + 'K';
        return Number(num).toLocaleString('en-US', { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
    };

    const formatPrice = (price) => {
        if (price === null || price === undefined || isNaN(price)) return '--';
        const num = Number(price);
        if (num >= 1000) return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
        if (num >= 1) return num.toFixed(4);
        return num.toFixed(6);
    };

    const formatDate = (timestamp, timeframe = '1h') => {
        const d = new Date(timestamp);
        if (timeframe === '1d') {
            return `${d.getMonth() + 1}/${d.getDate()}`;
        }
        return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')} (${d.getDate()}/${d.getMonth() + 1})`;
    };

    // 1. BINANCE API CLIENT (Client-side execution directly in browser)
    class BinanceApiClient {
        constructor() {
            this.spotBaseUrl = 'https://api.binance.com';
            this.futuresBaseUrl = 'https://fapi.binance.com';
        }

        async fetchWithTiming(url) {
            const start = performance.now();
            const response = await fetch(url);
            const latency = Math.round(performance.now() - start);
            if (!response.ok) {
                const errText = await response.text();
                throw new Error(`Binance API error (${response.status}): ${errText}`);
            }
            const data = await response.json();
            return { data, latency };
        }

        async getSpotKlines(symbol, interval, limit) {
            const url = `${this.spotBaseUrl}/api/v3/klines?symbol=${encodeURIComponent(symbol)}&interval=${interval}&limit=${limit}`;
            return this.fetchWithTiming(url);
        }

        async getFuturesKlines(symbol, interval, limit) {
            const url = `${this.futuresBaseUrl}/fapi/v1/klines?symbol=${encodeURIComponent(symbol)}&interval=${interval}&limit=${limit}`;
            return this.fetchWithTiming(url);
        }

        async getOpenInterestHist(symbol, period, limit) {
            // Map timeframe to valid Binance OI period: 5m, 15m, 30m, 1h, 2h, 4h, 6h, 12h, 1d
            let oiPeriod = period;
            if (period === '1m') oiPeriod = '5m';
            if (period === '3m') oiPeriod = '5m';
            const url = `${this.futuresBaseUrl}/futures/data/openInterestHist?symbol=${encodeURIComponent(symbol)}&period=${oiPeriod}&limit=${limit}`;
            return this.fetchWithTiming(url);
        }

        async get24hTickers(symbol) {
            const [spotRes, futRes, oiRes] = await Promise.allSettled([
                fetch(`${this.spotBaseUrl}/api/v3/ticker/24hr?symbol=${encodeURIComponent(symbol)}`).then(r => r.json()),
                fetch(`${this.futuresBaseUrl}/fapi/v1/ticker/24hr?symbol=${encodeURIComponent(symbol)}`).then(r => r.json()),
                fetch(`${this.futuresBaseUrl}/fapi/v1/openInterest?symbol=${encodeURIComponent(symbol)}`).then(r => r.json())
            ]);

            return {
                spot: spotRes.status === 'fulfilled' ? spotRes.value : null,
                futures: futRes.status === 'fulfilled' ? futRes.value : null,
                currentOi: oiRes.status === 'fulfilled' ? oiRes.value : null
            };
        }
    }

    // 2. MATHEMATICAL & ANALYTICAL COMPUTATION ENGINE
    class AnalyticsEngine {
        static calculatePearsonCorrelation(x, y) {
            const n = x.length;
            if (n < 2) return 0;

            let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0, sumY2 = 0;
            for (let i = 0; i < n; i++) {
                sumX += x[i];
                sumY += y[i];
                sumXY += x[i] * y[i];
                sumX2 += x[i] * x[i];
                sumY2 += y[i] * y[i];
            }

            const numerator = n * sumXY - sumX * sumY;
            const denominator = Math.sqrt((n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY));

            if (denominator === 0) return 0;
            const corr = numerator / denominator;
            return Math.max(-1, Math.min(1, corr));
        }

        static calculateRollingCorrelation(spotVol, futVol, window = 14) {
            const rolling = [];
            for (let i = 0; i < spotVol.length; i++) {
                if (i < window - 1) {
                    rolling.push(null);
                } else {
                    const sliceX = spotVol.slice(i - window + 1, i + 1);
                    const sliceY = futVol.slice(i - window + 1, i + 1);
                    rolling.push(this.calculatePearsonCorrelation(sliceX, sliceY));
                }
            }
            return rolling;
        }

        static processAndAlignData(spotKlines, futKlines, oiHistory, timeframe) {
            // Build lookup maps by openTime
            const spotMap = new Map();
            spotKlines.forEach(k => {
                spotMap.set(k[0], {
                    openTime: k[0],
                    open: parseFloat(k[1]),
                    high: parseFloat(k[2]),
                    low: parseFloat(k[3]),
                    close: parseFloat(k[4]),
                    volume: parseFloat(k[5]),
                    closeTime: k[6],
                    quoteVolume: parseFloat(k[7]),
                    trades: parseInt(k[8], 10),
                    takerBuyBaseVol: parseFloat(k[9]),
                    takerBuyQuoteVol: parseFloat(k[10])
                });
            });

            const futMap = new Map();
            futKlines.forEach(k => {
                futMap.set(k[0], {
                    openTime: k[0],
                    open: parseFloat(k[1]),
                    high: parseFloat(k[2]),
                    low: parseFloat(k[3]),
                    close: parseFloat(k[4]),
                    volume: parseFloat(k[5]),
                    closeTime: k[6],
                    quoteVolume: parseFloat(k[7]),
                    trades: parseInt(k[8], 10),
                    takerBuyBaseVol: parseFloat(k[9]),
                    takerBuyQuoteVol: parseFloat(k[10])
                });
            });

            // Process OI history sorted by timestamp
            const oiMap = new Map();
            if (Array.isArray(oiHistory)) {
                oiHistory.forEach(item => {
                    const ts = parseInt(item.timestamp, 10);
                    oiMap.set(ts, {
                        sumOpenInterest: parseFloat(item.sumOpenInterest),
                        sumOpenInterestValue: parseFloat(item.sumOpenInterestValue)
                    });
                });
            }

            // Find matching timestamps in chronological order
            const sortedTimestamps = Array.from(futMap.keys()).sort((a, b) => a - b);
            const alignedData = [];

            let spotCumulativeCvd = 0;
            let futCumulativeCvd = 0;

            for (let i = 0; i < sortedTimestamps.length; i++) {
                const ts = sortedTimestamps[i];
                const fut = futMap.get(ts);
                const spot = spotMap.get(ts);

                // If spot candle doesn't exist for exact ts, find closest or default
                const spotQuoteVol = spot ? spot.quoteVolume : 0;
                const futQuoteVol = fut ? fut.quoteVolume : 0;
                const spotPrice = spot ? spot.close : fut.close;
                const futPrice = fut.close;

                const volumeDelta = futQuoteVol - spotQuoteVol;
                const volumeRatio = spotQuoteVol > 0 ? futQuoteVol / spotQuoteVol : 0;

                // Taker CVD calculation (Quote Volume based in USDT)
                if (spot) {
                    const spotTakerSell = spot.quoteVolume - spot.takerBuyQuoteVol;
                    spotCumulativeCvd += (spot.takerBuyQuoteVol - spotTakerSell);
                }
                const futTakerSell = fut.quoteVolume - fut.takerBuyQuoteVol;
                futCumulativeCvd += (fut.takerBuyQuoteVol - futTakerSell);

                // Find matching or closest OI record within timeframe window
                let matchedOi = null;
                if (oiMap.has(ts)) {
                    matchedOi = oiMap.get(ts);
                } else {
                    // Find closest OI timestamp <= ts
                    let closestTs = 0;
                    for (const oiTs of oiMap.keys()) {
                        if (oiTs <= ts && oiTs > closestTs) {
                            closestTs = oiTs;
                        }
                    }
                    if (closestTs > 0) {
                        matchedOi = oiMap.get(closestTs);
                    }
                }

                alignedData.push({
                    timestamp: ts,
                    label: formatDate(ts, timeframe),
                    price: futPrice,
                    spotPrice: spotPrice,
                    open: fut.open,
                    high: fut.high,
                    low: fut.low,
                    close: fut.close,
                    spotVolumeUsdt: spotQuoteVol,
                    futuresVolumeUsdt: futQuoteVol,
                    volumeDelta: volumeDelta,
                    volumeRatio: volumeRatio,
                    spotCvd: spotCumulativeCvd,
                    futuresCvd: futCumulativeCvd,
                    openInterestCoins: matchedOi ? matchedOi.sumOpenInterest : null,
                    openInterestUsdt: matchedOi ? matchedOi.sumOpenInterestValue : null,
                    tradesCount: fut.trades
                });
            }

            // Calculate rolling correlation
            const spotVols = alignedData.map(d => d.spotVolumeUsdt);
            const futVols = alignedData.map(d => d.futuresVolumeUsdt);
            const rollingCorr = this.calculateRollingCorrelation(spotVols, futVols, 14);

            // Compute OI changes and 4-quadrant market classification
            for (let i = 0; i < alignedData.length; i++) {
                alignedData[i].correlation = rollingCorr[i];

                if (i > 0) {
                    const prevPrice = alignedData[i - 1].price;
                    const currPrice = alignedData[i].price;
                    const prevOi = alignedData[i - 1].openInterestUsdt;
                    const currOi = alignedData[i].openInterestUsdt;

                    const priceChange = currPrice - prevPrice;
                    const priceChangePct = prevPrice > 0 ? (priceChange / prevPrice) * 100 : 0;
                    alignedData[i].priceChangePct = priceChangePct;

                    let oiChangePct = 0;
                    let state = 'NEUTRAL';

                    if (currOi !== null && prevOi !== null && prevOi > 0) {
                        const oiChange = currOi - prevOi;
                        oiChangePct = (oiChange / prevOi) * 100;
                        alignedData[i].oiChangePct = oiChangePct;

                        if (priceChange >= 0 && oiChange >= 0) {
                            state = 'LONG_BUILDUP';
                        } else if (priceChange >= 0 && oiChange < 0) {
                            state = 'SHORT_SQUEEZE';
                        } else if (priceChange < 0 && oiChange >= 0) {
                            state = 'SHORT_BUILDUP';
                        } else if (priceChange < 0 && oiChange < 0) {
                            state = 'LONG_LIQUIDATION';
                        }
                    }

                    alignedData[i].marketState = state;
                } else {
                    alignedData[i].priceChangePct = 0;
                    alignedData[i].oiChangePct = 0;
                    alignedData[i].marketState = 'NEUTRAL';
                }
            }

            // Global Pearson correlation across entire dataset
            const overallCorrelation = this.calculatePearsonCorrelation(spotVols, futVols);

            return {
                items: alignedData,
                overallCorrelation
            };
        }
    }

    // 3. CHART MANAGER (Chart.js Multi-Chart Rendering Engine)
    class ChartManager {
        constructor() {
            this.charts = {
                priceVolume: null,
                volumeDelta: null,
                openInterest: null,
                correlationCvd: null
            };
        }

        getThemeColors() {
            const isDark = document.documentElement.getAttribute('data-theme') === 'dark' ||
                (!document.documentElement.getAttribute('data-theme') && window.matchMedia('(prefers-color-scheme: dark)').matches);

            return {
                isDark,
                text: isDark ? '#e2e8f0' : '#1e293b',
                mutedText: isDark ? '#94a3b8' : '#64748b',
                grid: isDark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.06)',
                border: isDark ? 'rgba(255, 255, 255, 0.12)' : 'rgba(0, 0, 0, 0.1)',
                spotColor: '#3b82f6', // Blue
                spotColorBg: 'rgba(59, 130, 246, 0.4)',
                futuresColor: '#f59e0b', // Amber / Orange
                futuresColorBg: 'rgba(245, 158, 11, 0.4)',
                deltaPositive: '#10b981', // Green (Futures > Spot)
                deltaNegative: '#6366f1', // Indigo (Spot > Futures)
                oiColor: '#ec4899', // Pink / Magenta
                oiColorBg: 'rgba(236, 72, 153, 0.15)',
                priceColor: isDark ? '#f8fafc' : '#0f172a',
                corrColor: '#8b5cf6', // Purple
                cvdSpotColor: '#06b6d4', // Cyan
                cvdFutColor: '#f97316' // Orange
            };
        }

        destroyAll() {
            Object.keys(this.charts).forEach(key => {
                if (this.charts[key]) {
                    this.charts[key].destroy();
                    this.charts[key] = null;
                }
            });
        }

        renderAll(data) {
            this.destroyAll();
            const colors = this.getThemeColors();
            const labels = data.items.map(d => d.label);

            this.renderPriceVolumeChart(data.items, labels, colors);
            this.renderVolumeDeltaChart(data.items, labels, colors);
            this.renderOpenInterestChart(data.items, labels, colors);
            this.renderCorrelationCvdChart(data.items, labels, colors);
        }

        renderPriceVolumeChart(items, labels, colors) {
            const ctx = document.getElementById('chart-price-volume');
            if (!ctx) return;

            const prices = items.map(d => d.price);
            const spotVols = items.map(d => d.spotVolumeUsdt);
            const futVols = items.map(d => d.futuresVolumeUsdt);

            this.charts.priceVolume = new Chart(ctx, {
                type: 'bar',
                data: {
                    labels: labels,
                    datasets: [
                        {
                            type: 'line',
                            label: 'Price ($)',
                            data: prices,
                            borderColor: colors.priceColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            pointRadius: 0,
                            pointHoverRadius: 5,
                            yAxisID: 'yPrice',
                            order: 1
                        },
                        {
                            type: 'bar',
                            label: 'Spot Volume ($)',
                            data: spotVols,
                            backgroundColor: colors.spotColorBg,
                            borderColor: colors.spotColor,
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 2
                        },
                        {
                            type: 'bar',
                            label: 'Futures Volume ($)',
                            data: futVols,
                            backgroundColor: colors.futuresColorBg,
                            borderColor: colors.futuresColor,
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 3
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: { labels: { color: colors.text, font: { family: 'Inter', size: 12 } } },
                        tooltip: {
                            callbacks: {
                                label: function (context) {
                                    if (context.dataset.yAxisID === 'yPrice') {
                                        return ` Price: $${formatPrice(context.raw)}`;
                                    }
                                    return ` ${context.dataset.label}: $${formatNumber(context.raw)}`;
                                }
                            }
                        }
                    },
                    scales: {
                        x: {
                            grid: { color: colors.grid },
                            ticks: { color: colors.mutedText, maxRotation: 0, autoSkip: true, maxTicksLimit: 12 }
                        },
                        yPrice: {
                            type: 'linear',
                            position: 'left',
                            grid: { color: colors.grid },
                            ticks: {
                                color: colors.text,
                                callback: val => '$' + formatPrice(val)
                            }
                        },
                        yVolume: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: {
                                color: colors.mutedText,
                                callback: val => '$' + formatNumber(val)
                            }
                        }
                    }
                }
            });
        }

        renderVolumeDeltaChart(items, labels, colors) {
            const ctx = document.getElementById('chart-volume-delta');
            if (!ctx) return;

            const deltas = items.map(d => d.volumeDelta);
            const ratios = items.map(d => d.volumeRatio);
            const barBgColors = deltas.map(v => v >= 0 ? colors.deltaPositive : colors.deltaNegative);

            this.charts.volumeDelta = new Chart(ctx, {
                data: {
                    labels: labels,
                    datasets: [
                        {
                            type: 'line',
                            label: 'Futures / Spot Ratio',
                            data: ratios,
                            borderColor: colors.futuresColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            pointRadius: 0,
                            pointHoverRadius: 4,
                            yAxisID: 'yRatio',
                            order: 1
                        },
                        {
                            type: 'bar',
                            label: 'Volume Delta (Futures - Spot $)',
                            data: deltas,
                            backgroundColor: barBgColors,
                            borderWidth: 0,
                            yAxisID: 'yDelta',
                            order: 2
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: { labels: { color: colors.text, font: { family: 'Inter', size: 12 } } },
                        tooltip: {
                            callbacks: {
                                label: function (context) {
                                    if (context.dataset.yAxisID === 'yRatio') {
                                        return ` F/S Ratio: ${Number(context.raw).toFixed(2)}x`;
                                    }
                                    const sign = context.raw >= 0 ? '+' : '';
                                    return ` Delta: ${sign}$${formatNumber(context.raw)}`;
                                }
                            }
                        }
                    },
                    scales: {
                        x: {
                            grid: { color: colors.grid },
                            ticks: { color: colors.mutedText, maxRotation: 0, autoSkip: true, maxTicksLimit: 12 }
                        },
                        yDelta: {
                            type: 'linear',
                            position: 'left',
                            grid: { color: colors.grid },
                            ticks: {
                                color: colors.text,
                                callback: val => '$' + formatNumber(val)
                            }
                        },
                        yRatio: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: {
                                color: colors.futuresColor,
                                callback: val => Number(val).toFixed(1) + 'x'
                            }
                        }
                    }
                }
            });
        }

        renderOpenInterestChart(items, labels, colors) {
            const ctx = document.getElementById('chart-open-interest');
            if (!ctx) return;

            const prices = items.map(d => d.price);
            const oiUsdt = items.map(d => d.openInterestUsdt);

            this.charts.openInterest = new Chart(ctx, {
                data: {
                    labels: labels,
                    datasets: [
                        {
                            type: 'line',
                            label: 'Price ($)',
                            data: prices,
                            borderColor: colors.priceColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            pointRadius: 0,
                            yAxisID: 'yPrice',
                            order: 1
                        },
                        {
                            type: 'line',
                            label: 'Open Interest ($)',
                            data: oiUsdt,
                            borderColor: colors.oiColor,
                            backgroundColor: colors.oiColorBg,
                            fill: true,
                            borderWidth: 2.5,
                            pointRadius: 0,
                            pointHoverRadius: 5,
                            yAxisID: 'yOi',
                            order: 2
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: { labels: { color: colors.text, font: { family: 'Inter', size: 12 } } },
                        tooltip: {
                            callbacks: {
                                label: function (context) {
                                    if (context.dataset.yAxisID === 'yPrice') {
                                        return ` Price: $${formatPrice(context.raw)}`;
                                    }
                                    return ` Open Interest: $${formatNumber(context.raw)}`;
                                }
                            }
                        }
                    },
                    scales: {
                        x: {
                            grid: { color: colors.grid },
                            ticks: { color: colors.mutedText, maxRotation: 0, autoSkip: true, maxTicksLimit: 12 }
                        },
                        yPrice: {
                            type: 'linear',
                            position: 'left',
                            grid: { color: colors.grid },
                            ticks: {
                                color: colors.text,
                                callback: val => '$' + formatPrice(val)
                            }
                        },
                        yOi: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: {
                                color: colors.oiColor,
                                callback: val => '$' + formatNumber(val)
                            }
                        }
                    }
                }
            });
        }

        renderCorrelationCvdChart(items, labels, colors) {
            const ctx = document.getElementById('chart-correlation-cvd');
            if (!ctx) return;

            const corrs = items.map(d => d.correlation);
            const spotCvd = items.map(d => d.spotCvd);
            const futCvd = items.map(d => d.futuresCvd);

            this.charts.correlationCvd = new Chart(ctx, {
                data: {
                    labels: labels,
                    datasets: [
                        {
                            type: 'line',
                            label: 'Rolling Vol Correlation (14)',
                            data: corrs,
                            borderColor: colors.corrColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            pointRadius: 0,
                            yAxisID: 'yCorr',
                            order: 1
                        },
                        {
                            type: 'line',
                            label: 'Spot CVD ($)',
                            data: spotCvd,
                            borderColor: colors.cvdSpotColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            borderDash: [4, 4],
                            pointRadius: 0,
                            yAxisID: 'yCvd',
                            order: 2
                        },
                        {
                            type: 'line',
                            label: 'Futures CVD ($)',
                            data: futCvd,
                            borderColor: colors.cvdFutColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            pointRadius: 0,
                            yAxisID: 'yCvd',
                            order: 3
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: { labels: { color: colors.text, font: { family: 'Inter', size: 12 } } },
                        tooltip: {
                            callbacks: {
                                label: function (context) {
                                    if (context.dataset.yAxisID === 'yCorr') {
                                        return ` Correlation: ${context.raw !== null ? Number(context.raw).toFixed(3) : '--'}`;
                                    }
                                    return ` ${context.dataset.label}: $${formatNumber(context.raw)}`;
                                }
                            }
                        }
                    },
                    scales: {
                        x: {
                            grid: { color: colors.grid },
                            ticks: { color: colors.mutedText, maxRotation: 0, autoSkip: true, maxTicksLimit: 12 }
                        },
                        yCorr: {
                            type: 'linear',
                            position: 'left',
                            min: -1.0,
                            max: 1.0,
                            grid: { color: colors.grid },
                            ticks: {
                                color: colors.corrColor,
                                callback: val => Number(val).toFixed(2)
                            }
                        },
                        yCvd: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: {
                                color: colors.mutedText,
                                callback: val => '$' + formatNumber(val)
                            }
                        }
                    }
                }
            });
        }
    }

    // 4. MAIN CONTROLLER & UI MANAGER
    class MarketAnalyticsApp {
        constructor() {
            this.apiClient = new BinanceApiClient();
            this.chartManager = new ChartManager();

            this.state = {
                symbol: 'BTCUSDT',
                timeframe: '1h',
                limit: 100,
                autoRefresh: true,
                refreshIntervalSec: 20,
                countdown: 20,
                timerId: null,
                countdownId: null,
                isLoading: false,
                currentData: null
            };

            this.init();
        }

        init() {
            this.bindElements();
            this.bindEvents();
            this.listenThemeChanges();
            this.fetchAndRender();
            this.startAutoRefresh();
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
                loadingOverlay: document.getElementById('loading-overlay'),
                errorMessage: document.getElementById('error-banner'),
                errorText: document.getElementById('error-banner-text'),
                btnExportCsv: document.getElementById('btn-export-csv'),

                // KPI Elements
                kpiSpotVol: document.getElementById('kpi-spot-vol'),
                kpiFutVol: document.getElementById('kpi-fut-vol'),
                kpiFsRatio: document.getElementById('kpi-fs-ratio'),
                kpiOiUsdt: document.getElementById('kpi-oi-usdt'),
                kpiOiCoins: document.getElementById('kpi-oi-coins'),
                kpiCorrelation: document.getElementById('kpi-correlation'),
                kpiCorrelationStatus: document.getElementById('kpi-correlation-status'),
                kpiMarketStateBadge: document.getElementById('kpi-market-state-badge'),
                kpiMarketStateDesc: document.getElementById('kpi-market-state-desc'),

                // Table element
                dataTableBody: document.getElementById('analytics-data-table-body'),
                tableSymbolLabel: document.getElementById('table-current-symbol')
            };
        }

        bindEvents() {
            // Symbol buttons click
            this.el.symbolButtons.forEach(btn => {
                btn.addEventListener('click', () => {
                    const sym = btn.getAttribute('data-symbol');
                    if (sym) {
                        this.setSymbol(sym);
                    }
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
                    if (tf) {
                        this.setTimeframe(tf);
                    }
                });
            });

            // Limit buttons
            this.el.limitButtons.forEach(btn => {
                btn.addEventListener('click', () => {
                    const lim = parseInt(btn.getAttribute('data-limit'), 10);
                    if (lim) {
                        this.setLimit(lim);
                    }
                });
            });

            // Refresh button
            if (this.el.btnRefresh) {
                this.el.btnRefresh.addEventListener('click', () => {
                    this.fetchAndRender();
                });
            }

            // Auto-refresh toggle
            if (this.el.autoRefreshToggle) {
                this.el.autoRefreshToggle.addEventListener('change', (e) => {
                    this.state.autoRefresh = e.target.checked;
                    if (this.state.autoRefresh) {
                        this.startAutoRefresh();
                    } else {
                        this.stopAutoRefresh();
                    }
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
                if (this.state.currentData) {
                    this.updateKpiDisplays(this.state.currentData);
                    this.renderTable(this.state.currentData.items);
                }
            });
        }

        listenThemeChanges() {
            const observer = new MutationObserver(() => {
                if (this.state.currentData) {
                    this.chartManager.renderAll(this.state.currentData);
                }
            });
            observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme', 'class'] });
            window.addEventListener('themeChanged', () => {
                if (this.state.currentData) {
                    this.chartManager.renderAll(this.state.currentData);
                }
            });
        }

        setSymbol(sym) {
            this.state.symbol = sym.toUpperCase();
            this.el.symbolButtons.forEach(btn => {
                if (btn.getAttribute('data-symbol') === this.state.symbol) {
                    btn.classList.add('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.remove('bg-gray-800/40', 'text-gray-300');
                } else {
                    btn.classList.remove('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.add('bg-gray-800/40', 'text-gray-300');
                }
            });
            if (this.el.symbolInput) this.el.symbolInput.value = this.state.symbol;
            if (this.el.tableSymbolLabel) this.el.tableSymbolLabel.textContent = this.state.symbol;
            this.fetchAndRender();
        }

        setTimeframe(tf) {
            this.state.timeframe = tf;
            this.el.timeframeButtons.forEach(btn => {
                if (btn.getAttribute('data-timeframe') === tf) {
                    btn.classList.add('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.remove('bg-gray-800/40', 'text-gray-300');
                } else {
                    btn.classList.remove('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.add('bg-gray-800/40', 'text-gray-300');
                }
            });
            this.fetchAndRender();
        }

        setLimit(lim) {
            this.state.limit = lim;
            this.el.limitButtons.forEach(btn => {
                if (parseInt(btn.getAttribute('data-limit'), 10) === lim) {
                    btn.classList.add('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.remove('bg-gray-800/40', 'text-gray-300');
                } else {
                    btn.classList.remove('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.add('bg-gray-800/40', 'text-gray-300');
                }
            });
            this.fetchAndRender();
        }

        startAutoRefresh() {
            this.stopAutoRefresh();
            this.state.countdown = this.state.refreshIntervalSec;
            this.updateCountdownDisplay();

            this.state.countdownId = setInterval(() => {
                if (this.state.countdown > 1) {
                    this.state.countdown--;
                    this.updateCountdownDisplay();
                } else {
                    this.state.countdown = this.state.refreshIntervalSec;
                    this.updateCountdownDisplay();
                    this.fetchAndRender(true);
                }
            }, 1000);
        }

        stopAutoRefresh() {
            if (this.state.countdownId) {
                clearInterval(this.state.countdownId);
                this.state.countdownId = null;
            }
            if (this.el.countdownBadge) {
                this.el.countdownBadge.textContent = 'Paused';
            }
        }

        updateCountdownDisplay() {
            if (this.el.countdownBadge) {
                this.el.countdownBadge.textContent = `${this.state.countdown}s`;
            }
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

        async fetchAndRender(isBackground = false) {
            if (!isBackground) this.showLoading(true);
            this.showError(null);

            try {
                const { symbol, timeframe, limit } = this.state;

                // Concurrent fetch directly from Binance Public REST API
                const [spotRes, futRes, oiRes, tickers] = await Promise.all([
                    this.apiClient.getSpotKlines(symbol, timeframe, limit),
                    this.apiClient.getFuturesKlines(symbol, timeframe, limit),
                    this.apiClient.getOpenInterestHist(symbol, timeframe, limit).catch(() => ({ data: [], latency: 0 })),
                    this.apiClient.get24hTickers(symbol)
                ]);

                // Update latency badge
                const avgLatency = Math.round((spotRes.latency + futRes.latency) / 2);
                if (this.el.apiLatency) {
                    this.el.apiLatency.textContent = `${avgLatency} ms`;
                    this.el.apiLatency.className = avgLatency < 300
                        ? 'px-2 py-0.5 text-xs font-semibold rounded bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                        : 'px-2 py-0.5 text-xs font-semibold rounded bg-amber-500/20 text-amber-400 border border-amber-500/30';
                }

                // Process and align data in client
                const processed = AnalyticsEngine.processAndAlignData(spotRes.data, futRes.data, oiRes.data, timeframe);
                processed.tickers = tickers;

                this.state.currentData = processed;

                // Render components
                this.updateKpiDisplays(processed);
                this.chartManager.renderAll(processed);
                this.renderTable(processed.items);

            } catch (err) {
                console.error('Fetch and render error:', err);
                const isVi = (document.documentElement.lang || 'vi') === 'vi';
                this.showError((isVi ? 'Không thể tải dữ liệu: ' : 'Failed to fetch data: ') + err.message);
            } finally {
                this.showLoading(false);
            }
        }

        updateKpiDisplays(data) {
            const tickers = data.tickers || {};
            const items = data.items;
            const lastItem = items.length > 0 ? items[items.length - 1] : null;

            // Spot 24h Vol
            if (this.el.kpiSpotVol) {
                const spotVol = tickers.spot ? parseFloat(tickers.spot.quoteVolume) : (lastItem ? lastItem.spotVolumeUsdt * 24 : 0);
                this.el.kpiSpotVol.textContent = '$' + formatNumber(spotVol);
            }

            // Futures 24h Vol
            if (this.el.kpiFutVol) {
                const futVol = tickers.futures ? parseFloat(tickers.futures.quoteVolume) : (lastItem ? lastItem.futuresVolumeUsdt * 24 : 0);
                this.el.kpiFutVol.textContent = '$' + formatNumber(futVol);
            }

            // F/S Ratio
            if (this.el.kpiFsRatio) {
                const spotVol = tickers.spot ? parseFloat(tickers.spot.quoteVolume) : 1;
                const futVol = tickers.futures ? parseFloat(tickers.futures.quoteVolume) : 1;
                const ratio = spotVol > 0 ? (futVol / spotVol) : 0;
                this.el.kpiFsRatio.textContent = ratio.toFixed(2) + 'x';
            }

            // Open Interest
            if (this.el.kpiOiUsdt && this.el.kpiOiCoins) {
                if (tickers.currentOi && lastItem) {
                    const oiCoins = parseFloat(tickers.currentOi.openInterest);
                    const oiUsdt = oiCoins * lastItem.price;
                    this.el.kpiOiUsdt.textContent = '$' + formatNumber(oiUsdt);
                    this.el.kpiOiCoins.textContent = `${formatNumber(oiCoins)} ${this.state.symbol.replace('USDT', '')}`;
                } else if (lastItem && lastItem.openInterestUsdt) {
                    this.el.kpiOiUsdt.textContent = '$' + formatNumber(lastItem.openInterestUsdt);
                    this.el.kpiOiCoins.textContent = `${formatNumber(lastItem.openInterestCoins)} ${this.state.symbol.replace('USDT', '')}`;
                } else {
                    this.el.kpiOiUsdt.textContent = '--';
                    this.el.kpiOiCoins.textContent = '--';
                }
            }

            // Correlation
            if (this.el.kpiCorrelation && this.el.kpiCorrelationStatus) {
                const corr = data.overallCorrelation;
                this.el.kpiCorrelation.textContent = corr.toFixed(3);

                const isVi = (document.documentElement.lang || 'vi') === 'vi';
                if (corr >= 0.7) {
                    this.el.kpiCorrelationStatus.textContent = isVi ? 'Đồng thuận cao (Thị trường thực)' : 'High Alignment (Organic)';
                    this.el.kpiCorrelationStatus.className = 'text-xs font-semibold text-emerald-400';
                } else if (corr <= 0.3) {
                    this.el.kpiCorrelationStatus.textContent = isVi ? 'Phân kỳ / Đầu cơ phái sinh' : 'Divergence (Speculative)';
                    this.el.kpiCorrelationStatus.className = 'text-xs font-semibold text-amber-400';
                } else {
                    this.el.kpiCorrelationStatus.textContent = isVi ? 'Tương quan trung bình' : 'Moderate Correlation';
                    this.el.kpiCorrelationStatus.className = 'text-xs font-semibold text-blue-400';
                }
            }

            // Market State Badge
            if (this.el.kpiMarketStateBadge && this.el.kpiMarketStateDesc && lastItem) {
                const state = lastItem.marketState;
                const isVi = (document.documentElement.lang || 'vi') === 'vi';

                switch (state) {
                    case 'LONG_BUILDUP':
                        this.el.kpiMarketStateBadge.textContent = '🟢 Long Build-up';
                        this.el.kpiMarketStateBadge.className = 'px-3 py-1 text-sm font-bold rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30';
                        this.el.kpiMarketStateDesc.textContent = isVi ? 'Giá tăng + Vị thế mở tăng: Dòng tiền mở Long áp đảo' : 'Price ↑ + OI ↑: Aggressive Long build-up';
                        break;
                    case 'SHORT_SQUEEZE':
                        this.el.kpiMarketStateBadge.textContent = '🟡 Short Squeeze';
                        this.el.kpiMarketStateBadge.className = 'px-3 py-1 text-sm font-bold rounded-full bg-yellow-500/20 text-yellow-400 border border-yellow-500/30';
                        this.el.kpiMarketStateDesc.textContent = isVi ? 'Giá tăng + Vị thế mở giảm: Tăng do Short bị thanh lý/cắt lỗ' : 'Price ↑ + OI ↓: Rally driven by Short covering';
                        break;
                    case 'SHORT_BUILDUP':
                        this.el.kpiMarketStateBadge.textContent = '🔴 Short Build-up';
                        this.el.kpiMarketStateBadge.className = 'px-3 py-1 text-sm font-bold rounded-full bg-rose-500/20 text-rose-400 border border-rose-500/30';
                        this.el.kpiMarketStateDesc.textContent = isVi ? 'Giá giảm + Vị thế mở tăng: Dòng tiền mở Short áp đảo' : 'Price ↓ + OI ↑: Aggressive Short build-up';
                        break;
                    case 'LONG_LIQUIDATION':
                        this.el.kpiMarketStateBadge.textContent = '⚪ Long Liquidation';
                        this.el.kpiMarketStateBadge.className = 'px-3 py-1 text-sm font-bold rounded-full bg-indigo-500/20 text-indigo-400 border border-indigo-500/30';
                        this.el.kpiMarketStateDesc.textContent = isVi ? 'Giá giảm + Vị thế mở giảm: Long bị thanh lý / rời bỏ vị thế' : 'Price ↓ + OI ↓: Long capitulation & liquidation';
                        break;
                    default:
                        this.el.kpiMarketStateBadge.textContent = '⚪ Neutral';
                        this.el.kpiMarketStateBadge.className = 'px-3 py-1 text-sm font-bold rounded-full bg-gray-500/20 text-gray-400 border border-gray-500/30';
                        this.el.kpiMarketStateDesc.textContent = isVi ? 'Thị trường cân bằng hoặc chưa đủ nến để xác định' : 'Balanced market state';
                }
            }
        }

        renderTable(items) {
            if (!this.el.dataTableBody) return;
            const isVi = (document.documentElement.lang || 'vi') === 'vi';

            // Show latest 25 candles in reverse order
            const displayItems = items.slice().reverse().slice(0, 30);

            let html = '';
            displayItems.forEach(row => {
                const deltaColor = row.volumeDelta >= 0 ? 'text-emerald-400' : 'text-indigo-400';
                const deltaSign = row.volumeDelta >= 0 ? '+' : '';
                const priceColor = row.priceChangePct >= 0 ? 'text-emerald-400' : 'text-rose-400';
                const priceSign = row.priceChangePct >= 0 ? '+' : '';

                let stateTag = '';
                switch (row.marketState) {
                    case 'LONG_BUILDUP':
                        stateTag = '<span class="px-2 py-0.5 rounded text-xs font-semibold bg-emerald-500/20 text-emerald-400">Long Build-up</span>';
                        break;
                    case 'SHORT_SQUEEZE':
                        stateTag = '<span class="px-2 py-0.5 rounded text-xs font-semibold bg-yellow-500/20 text-yellow-400">Short Squeeze</span>';
                        break;
                    case 'SHORT_BUILDUP':
                        stateTag = '<span class="px-2 py-0.5 rounded text-xs font-semibold bg-rose-500/20 text-rose-400">Short Build-up</span>';
                        break;
                    case 'LONG_LIQUIDATION':
                        stateTag = '<span class="px-2 py-0.5 rounded text-xs font-semibold bg-indigo-500/20 text-indigo-400">Long Liquidation</span>';
                        break;
                    default:
                        stateTag = '<span class="px-2 py-0.5 rounded text-xs font-semibold bg-gray-500/20 text-gray-400">Neutral</span>';
                }

                html += `
                    <tr class="border-b border-gray-800/40 hover:bg-white/5 transition-colors">
                        <td class="px-4 py-3 text-xs font-medium text-gray-300">${row.label}</td>
                        <td class="px-4 py-3 text-xs font-bold text-gray-100">$${formatPrice(row.price)} <span class="${priceColor} ml-1">(${priceSign}${row.priceChangePct.toFixed(2)}%)</span></td>
                        <td class="px-4 py-3 text-xs text-blue-400 font-semibold">$${formatNumber(row.spotVolumeUsdt)}</td>
                        <td class="px-4 py-3 text-xs text-amber-400 font-semibold">$${formatNumber(row.futuresVolumeUsdt)}</td>
                        <td class="px-4 py-3 text-xs font-semibold ${deltaColor}">${deltaSign}$${formatNumber(row.volumeDelta)}</td>
                        <td class="px-4 py-3 text-xs font-bold text-gray-200">${row.volumeRatio.toFixed(2)}x</td>
                        <td class="px-4 py-3 text-xs text-pink-400 font-semibold">${row.openInterestUsdt ? '$' + formatNumber(row.openInterestUsdt) : '--'}</td>
                        <td class="px-4 py-3 text-xs">${stateTag}</td>
                    </tr>
                `;
            });

            this.el.dataTableBody.innerHTML = html;
        }

        exportCsv() {
            if (!this.state.currentData || !this.state.currentData.items.length) {
                alert('No data to export!');
                return;
            }

            const items = this.state.currentData.items;
            const headers = ['Timestamp', 'Date', 'Price', 'PriceChangePct', 'SpotVolumeUSDT', 'FuturesVolumeUSDT', 'VolumeDeltaUSDT', 'FuturesSpotRatio', 'OpenInterestUSDT', 'Correlation14', 'MarketState'];
            const rows = items.map(d => [
                d.timestamp,
                `"${d.label}"`,
                d.price,
                d.priceChangePct.toFixed(2),
                d.spotVolumeUsdt.toFixed(2),
                d.futuresVolumeUsdt.toFixed(2),
                d.volumeDelta.toFixed(2),
                d.volumeRatio.toFixed(2),
                d.openInterestUsdt ? d.openInterestUsdt.toFixed(2) : '',
                d.correlation !== null ? d.correlation.toFixed(3) : '',
                `"${d.marketState}"`
            ]);

            const csvContent = 'data:text/csv;charset=utf-8,' + [headers.join(','), ...rows.map(e => e.join(','))].join('\n');
            const encodedUri = encodeURI(csvContent);
            const link = document.createElement('a');
            link.setAttribute('href', encodedUri);
            link.setAttribute('download', `${this.state.symbol}_${this.state.timeframe}_volume_analytics.csv`);
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
        }
    }

    // Auto-bootstrap on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            window.marketAnalyticsApp = new MarketAnalyticsApp();
        });
    } else {
        window.marketAnalyticsApp = new MarketAnalyticsApp();
    }

})();
