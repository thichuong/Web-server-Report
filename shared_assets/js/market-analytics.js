/**
 * Market Analytics Client-side Engine with Real-Time WebSocket Streaming & Diagnostics
 * 
 * Features:
 * 1. Historical Snapshot Bootstrap from Binance REST API (Spot & Futures Klines, OI, Long/Short Ratio).
 * 2. Real-time live streaming via Binance Spot & Futures WebSockets (wss://stream.binance.com:9443 & wss://fstream.binance.com).
 * 3. In-place Chart.js Multi-Chart Engine with persistent legend visibility & options preservation across reloads and ticks.
 * 4. High-performance throttled mathematical computation: Pearson Correlation, Net Delta, CVD, OI Valuation, Position Ratios.
 * 5. Integrated Real-time Diagnostics / Debug Console with live metrics and event logs.
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

    // 1. BINANCE REST API CLIENT (For initial snapshots and background sync)
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
            let oiPeriod = period;
            if (period === '1m' || period === '3m') oiPeriod = '5m';
            const url = `${this.futuresBaseUrl}/futures/data/openInterestHist?symbol=${encodeURIComponent(symbol)}&period=${oiPeriod}&limit=${limit}`;
            return this.fetchWithTiming(url);
        }

        async getTopLongShortPositionRatio(symbol, period, limit) {
            let oiPeriod = period;
            if (period === '1m' || period === '3m') oiPeriod = '5m';
            const url = `${this.futuresBaseUrl}/futures/data/topLongShortPositionRatio?symbol=${encodeURIComponent(symbol)}&period=${oiPeriod}&limit=${limit}`;
            return this.fetchWithTiming(url);
        }

        async getGlobalLongShortAccountRatio(symbol, period, limit) {
            let oiPeriod = period;
            if (period === '1m' || period === '3m') oiPeriod = '5m';
            const url = `${this.futuresBaseUrl}/futures/data/globalLongShortAccountRatio?symbol=${encodeURIComponent(symbol)}&period=${oiPeriod}&limit=${limit}`;
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

    // 2. BINANCE WEBSOCKET MANAGER (Dual Independent Streams for Spot & Futures)
    class BinanceWebSocketManager {
        constructor(config = {}) {
            this.spotWs = null;
            this.futuresWs = null;
            this.currentSymbol = null;
            this.currentTimeframe = null;

            this.spotConnected = false;
            this.futuresConnected = false;
            this.spotMsgCount = 0;
            this.futuresMsgCount = 0;
            this.spotReconnectAttempts = 0;
            this.futuresReconnectAttempts = 0;
            this.maxReconnectAttempts = 12;

            this.onSpotKline = config.onSpotKline || (() => {});
            this.onFuturesKline = config.onFuturesKline || (() => {});
            this.onSpotTicker = config.onSpotTicker || (() => {});
            this.onFuturesTicker = config.onFuturesTicker || (() => {});
            this.onStatusChange = config.onStatusChange || (() => {});
            this.onLog = config.onLog || (() => {});
        }

        subscribe(symbol, timeframe) {
            this.currentSymbol = symbol.toLowerCase();
            this.currentTimeframe = timeframe;
            this.disconnect();
            this.connectSpot();
            this.connectFutures();
        }

        connectSpot() {
            if (!this.currentSymbol || !this.currentTimeframe) return;
            const sym = this.currentSymbol;
            const tf = this.currentTimeframe;

            this.onLog('spot', `Initiating Spot WS connection: ${sym}@kline_${tf} & ${sym}@ticker`);
            this.onStatusChange('spot', 'connecting');

            try {
                // Try combined stream on port 9443
                const url = `wss://stream.binance.com:9443/stream?streams=${sym}@kline_${tf}/${sym}@ticker`;
                this.spotWs = new WebSocket(url);

                this.spotWs.onopen = () => {
                    this.spotConnected = true;
                    this.spotReconnectAttempts = 0;
                    this.onLog('spot', '🟢 Spot WebSocket connected successfully');
                    this.onStatusChange('spot', 'connected');
                };

                this.spotWs.onmessage = (event) => {
                    this.spotMsgCount++;
                    try {
                        const parsed = JSON.parse(event.data);
                        const data = parsed.data || parsed;
                        const streamName = parsed.stream || '';

                        if (data.e === 'kline' && data.k) {
                            this.onSpotKline(data.k);
                        } else if (data.e === '24hrTicker' || data.e === '24hrMiniTicker' || streamName.includes('@ticker')) {
                            this.onSpotTicker(data);
                        }
                    } catch (err) {
                        this.onLog('spot', `❌ Error parsing Spot message: ${err.message}`);
                    }
                };

                this.spotWs.onerror = (err) => {
                    this.onLog('spot', `⚠️ Spot WS error: ${err.message || 'Connection failed'}`);
                };

                this.spotWs.onclose = (event) => {
                    this.spotConnected = false;
                    this.onLog('spot', `🔌 Spot WS disconnected (code: ${event.code})`);
                    this.onStatusChange('spot', 'disconnected');
                    if (event.code !== 1000) {
                        this.scheduleSpotReconnect();
                    }
                };
            } catch (e) {
                this.onLog('spot', `❌ Spot WS Init Exception: ${e.message}`);
                this.scheduleSpotReconnect();
            }
        }

        connectFutures() {
            if (!this.currentSymbol || !this.currentTimeframe) return;
            const sym = this.currentSymbol;
            const tf = this.currentTimeframe;

            this.onLog('futures', `Initiating Futures WS connection: ${sym}@kline_${tf} & ${sym}@ticker`);
            this.onStatusChange('futures', 'connecting');

            try {
                // Futures combined stream (Binance upgraded USDS-M Futures to dedicated /market base endpoint)
                const url = `wss://fstream.binance.com/market/stream?streams=${sym}@kline_${tf}/${sym}@ticker`;
                this.futuresWs = new WebSocket(url);

                this.futuresWs.onopen = () => {
                    this.futuresConnected = true;
                    this.futuresReconnectAttempts = 0;
                    this.onLog('futures', '🟢 Futures WebSocket connected successfully');
                    this.onStatusChange('futures', 'connected');
                };

                this.futuresWs.onmessage = (event) => {
                    this.futuresMsgCount++;
                    try {
                        const parsed = JSON.parse(event.data);
                        const data = parsed.data || parsed;
                        const streamName = parsed.stream || '';

                        if (data.e === 'kline' && data.k) {
                            this.onFuturesKline(data.k);
                        } else if (data.e === '24hrTicker' || data.e === '24hrMiniTicker' || streamName.includes('@ticker')) {
                            this.onFuturesTicker(data);
                        }
                    } catch (err) {
                        this.onLog('futures', `❌ Error parsing Futures message: ${err.message}`);
                    }
                };

                this.futuresWs.onerror = (err) => {
                    this.onLog('futures', `⚠️ Futures WS error: ${err.message || 'Connection failed'}`);
                };

                this.futuresWs.onclose = (event) => {
                    this.futuresConnected = false;
                    this.onLog('futures', `🔌 Futures WS disconnected (code: ${event.code})`);
                    this.onStatusChange('futures', 'disconnected');
                    if (event.code !== 1000) {
                        this.scheduleFuturesReconnect();
                    }
                };
            } catch (e) {
                this.onLog('futures', `❌ Futures WS Init Exception: ${e.message}`);
                this.scheduleFuturesReconnect();
            }
        }

        scheduleSpotReconnect() {
            if (this.spotReconnectAttempts >= this.maxReconnectAttempts) {
                this.onLog('spot', '❌ Spot WS max reconnect attempts reached');
                return;
            }
            this.spotReconnectAttempts++;
            const delay = Math.min(1000 * Math.pow(1.5, this.spotReconnectAttempts), 10000);
            this.onLog('spot', `🔄 Reconnecting Spot WS in ${Math.round(delay)}ms... (Attempt ${this.spotReconnectAttempts})`);
            setTimeout(() => this.connectSpot(), delay);
        }

        scheduleFuturesReconnect() {
            if (this.futuresReconnectAttempts >= this.maxReconnectAttempts) {
                this.onLog('futures', '❌ Futures WS max reconnect attempts reached');
                return;
            }
            this.futuresReconnectAttempts++;
            const delay = Math.min(1000 * Math.pow(1.5, this.futuresReconnectAttempts), 10000);
            this.onLog('futures', `🔄 Reconnecting Futures WS in ${Math.round(delay)}ms... (Attempt ${this.futuresReconnectAttempts})`);
            setTimeout(() => this.connectFutures(), delay);
        }

        disconnect() {
            if (this.spotWs) {
                this.spotWs.onclose = null;
                this.spotWs.close();
                this.spotWs = null;
            }
            if (this.futuresWs) {
                this.futuresWs.onclose = null;
                this.futuresWs.close();
                this.futuresWs = null;
            }
            this.spotConnected = false;
            this.futuresConnected = false;
            this.onStatusChange('all', 'disconnected');
        }
    }

    // 3. MATHEMATICAL & ANALYTICAL COMPUTATION ENGINE
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

        static processAndAlignData(spotKlines, futKlines, oiHistory, lsHistory, timeframe) {
            // Build lookup maps by openTime (parsed as integer timestamp)
            const spotMap = new Map();
            spotKlines.forEach(k => {
                const ts = parseInt(k[0], 10);
                spotMap.set(ts, {
                    openTime: ts,
                    open: parseFloat(k[1]),
                    high: parseFloat(k[2]),
                    low: parseFloat(k[3]),
                    close: parseFloat(k[4]),
                    volume: parseFloat(k[5]),
                    closeTime: parseInt(k[6], 10),
                    quoteVolume: parseFloat(k[7]),
                    trades: parseInt(k[8], 10) || 0,
                    takerBuyBaseVol: parseFloat(k[9]),
                    takerBuyQuoteVol: parseFloat(k[10])
                });
            });

            const futMap = new Map();
            futKlines.forEach(k => {
                const ts = parseInt(k[0], 10);
                futMap.set(ts, {
                    openTime: ts,
                    open: parseFloat(k[1]),
                    high: parseFloat(k[2]),
                    low: parseFloat(k[3]),
                    close: parseFloat(k[4]),
                    volume: parseFloat(k[5]),
                    closeTime: parseInt(k[6], 10),
                    quoteVolume: parseFloat(k[7]),
                    trades: parseInt(k[8], 10) || 0,
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

            // Process Long/Short history sorted by timestamp
            const lsMap = new Map();
            if (Array.isArray(lsHistory)) {
                lsHistory.forEach(item => {
                    const ts = parseInt(item.timestamp, 10);
                    const longAcc = parseFloat(item.longAccount || item.longPosition || 0.5);
                    const shortAcc = parseFloat(item.shortAccount || item.shortPosition || (1.0 - longAcc));
                    const ratio = parseFloat(item.longShortRatio || (shortAcc > 0 ? longAcc / shortAcc : 1.0));
                    lsMap.set(ts, {
                        longRatio: longAcc,
                        shortRatio: shortAcc,
                        longShortRatio: ratio
                    });
                });
            }

            // Merge all timestamps from both Spot and Futures
            const allTimestamps = new Set([...spotMap.keys(), ...futMap.keys()]);
            const sortedTimestamps = Array.from(allTimestamps).sort((a, b) => a - b);
            const alignedData = [];

            let spotCumulativeCvd = 0;
            let futCumulativeCvd = 0;

            for (let i = 0; i < sortedTimestamps.length; i++) {
                const ts = sortedTimestamps[i];
                const futCandle = futMap.get(ts);
                const spotCandle = spotMap.get(ts);

                // Fallback to whichever is available if one stream has a slight lead
                const fut = futCandle || spotCandle;
                const spot = spotCandle || futCandle;
                if (!fut && !spot) continue;

                const spotQuoteVol = spot ? spot.quoteVolume : 0;
                const spotBuyQuoteVol = spot ? spot.takerBuyQuoteVol : 0;
                const spotSellQuoteVol = Math.max(0, spotQuoteVol - spotBuyQuoteVol);
                const spotNetDelta = spotBuyQuoteVol - spotSellQuoteVol;
                const spotBuyRatio = spotSellQuoteVol > 0 ? (spotBuyQuoteVol / spotSellQuoteVol) : (spotBuyQuoteVol > 0 ? 10 : 1);
                const spotBuyPct = spotQuoteVol > 0 ? (spotBuyQuoteVol / spotQuoteVol) * 100 : 50;

                const futQuoteVol = fut ? fut.quoteVolume : 0;
                const futBuyQuoteVol = fut ? fut.takerBuyQuoteVol : 0;
                const futSellQuoteVol = Math.max(0, futQuoteVol - futBuyQuoteVol);
                const futNetDelta = futBuyQuoteVol - futSellQuoteVol;
                const futBuyRatio = futSellQuoteVol > 0 ? (futBuyQuoteVol / futSellQuoteVol) : (futBuyQuoteVol > 0 ? 10 : 1);
                const futBuyPct = futQuoteVol > 0 ? (futBuyQuoteVol / futQuoteVol) * 100 : 50;

                const spotPrice = spot ? spot.close : fut.close;
                const futPrice = fut ? fut.close : spot.close;

                const volumeDelta = futQuoteVol - spotQuoteVol;
                const volumeRatio = spotQuoteVol > 0 ? futQuoteVol / spotQuoteVol : 0;

                // Taker CVD calculation (Quote Volume based in USDT)
                spotCumulativeCvd += spotNetDelta;
                futCumulativeCvd += futNetDelta;

                // Find matching or closest OI record
                let matchedOi = null;
                if (oiMap.has(ts)) {
                    matchedOi = oiMap.get(ts);
                } else {
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

                // Find matching or closest Long/Short ratio record
                let matchedLs = null;
                if (lsMap.has(ts)) {
                    matchedLs = lsMap.get(ts);
                } else {
                    let closestLsTs = 0;
                    for (const lTs of lsMap.keys()) {
                        if (lTs <= ts && lTs > closestLsTs) {
                            closestLsTs = lTs;
                        }
                    }
                    if (closestLsTs > 0) {
                        matchedLs = lsMap.get(closestLsTs);
                    }
                }

                const longRatio = matchedLs ? matchedLs.longRatio : 0.5;
                const shortRatio = matchedLs ? matchedLs.shortRatio : 0.5;
                const lsRatio = matchedLs ? matchedLs.longShortRatio : 1.0;

                const openInterestCoins = matchedOi ? matchedOi.sumOpenInterest : null;
                const openInterestUsdt = matchedOi ? matchedOi.sumOpenInterestValue : null;

                // Estimated notional position values ($)
                const longPositionUsdt = openInterestUsdt !== null ? (openInterestUsdt * longRatio) : null;
                const shortPositionUsdt = openInterestUsdt !== null ? (openInterestUsdt * shortRatio) : null;

                // Comparison with Spot Volume:
                const longSpotRatio = (spotQuoteVol > 0 && longPositionUsdt !== null) ? (longPositionUsdt / spotQuoteVol) : 0;
                const shortSpotRatio = (spotQuoteVol > 0 && shortPositionUsdt !== null) ? (shortPositionUsdt / spotQuoteVol) : 0;
                const netPositionUsdt = (longPositionUsdt !== null && shortPositionUsdt !== null) ? (longPositionUsdt - shortPositionUsdt) : 0;

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
                    spotBuyVolumeUsdt: spotBuyQuoteVol,
                    spotSellVolumeUsdt: spotSellQuoteVol,
                    spotNetDelta: spotNetDelta,
                    spotBuyRatio: spotBuyRatio,
                    spotBuyPct: spotBuyPct,
                    futuresVolumeUsdt: futQuoteVol,
                    futuresBuyVolumeUsdt: futBuyQuoteVol,
                    futuresSellVolumeUsdt: futSellQuoteVol,
                    futuresNetDelta: futNetDelta,
                    futuresBuyRatio: futBuyRatio,
                    futuresBuyPct: futBuyPct,
                    volumeDelta: volumeDelta,
                    volumeRatio: volumeRatio,
                    spotCvd: spotCumulativeCvd,
                    futuresCvd: futCumulativeCvd,
                    openInterestCoins: openInterestCoins,
                    openInterestUsdt: openInterestUsdt,
                    longRatio: longRatio,
                    shortRatio: shortRatio,
                    longShortRatio: lsRatio,
                    longPositionUsdt: longPositionUsdt,
                    shortPositionUsdt: shortPositionUsdt,
                    longSpotRatio: longSpotRatio,
                    shortSpotRatio: shortSpotRatio,
                    netPositionUsdt: netPositionUsdt,
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

            // Global Pearson correlation
            const overallCorrelation = this.calculatePearsonCorrelation(spotVols, futVols);

            return {
                items: alignedData,
                overallCorrelation
            };
        }
    }

    // 4. CHART MANAGER (In-Place Updates with Absolute Legend State Preservation)
    class ChartManager {
        constructor() {
            this.charts = {
                priceVolume: null,
                buySellBreakdown: null,
                volumeDelta: null,
                longShortSpot: null,
                longShortRatio: null,
                openInterest: null,
                netTakerDelta: null,
                correlationCvd: null
            };
            this.savedVisibility = {};
        }

        saveVisibility(chartKey) {
            const chart = this.charts[chartKey];
            if (!chart || !chart.data || !chart.data.datasets) return;
            if (!this.savedVisibility[chartKey]) this.savedVisibility[chartKey] = {};

            chart.data.datasets.forEach((ds, idx) => {
                const isVisible = chart.isDatasetVisible(idx);
                const label = ds.label || `dataset_${idx}`;
                this.savedVisibility[chartKey][label] = isVisible;
            });
        }

        restoreVisibility(chartKey) {
            const chart = this.charts[chartKey];
            const saved = this.savedVisibility[chartKey];
            if (!chart || !saved || !chart.data || !chart.data.datasets) return;

            chart.data.datasets.forEach((ds, idx) => {
                const label = ds.label || `dataset_${idx}`;
                if (label in saved) {
                    chart.setDatasetVisibility(idx, saved[label]);
                }
            });
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
                spotColor: '#3b82f6',
                spotColorBg: 'rgba(59, 130, 246, 0.4)',
                futuresColor: '#f59e0b',
                futuresColorBg: 'rgba(245, 158, 11, 0.4)',
                buyColor: '#10b981',
                buyColorBg: 'rgba(16, 185, 129, 0.5)',
                sellColor: '#f43f5e',
                sellColorBg: 'rgba(244, 63, 94, 0.5)',
                spotBuyColor: '#06b6d4',
                spotSellColor: '#3b82f6',
                futBuyColor: '#10b981',
                futSellColor: '#f43f5e',
                deltaPositive: '#10b981',
                deltaNegative: '#6366f1',
                longColor: '#10b981',
                longColorBg: 'rgba(16, 185, 129, 0.4)',
                shortColor: '#f43f5e',
                shortColorBg: 'rgba(244, 63, 94, 0.4)',
                lsRatioColor: '#818cf8',
                oiColor: '#ec4899',
                oiColorBg: 'rgba(236, 72, 153, 0.15)',
                priceColor: isDark ? '#f8fafc' : '#0f172a',
                corrColor: '#8b5cf6',
                cvdSpotColor: '#06b6d4',
                cvdFutColor: '#f97316'
            };
        }

        renderAll(data) {
            const colors = this.getThemeColors();
            const labels = data.items.map(d => d.label);

            this.renderPriceVolumeChart(data.items, labels, colors);
            this.renderBuySellBreakdownChart(data.items, labels, colors);
            this.renderNetTakerDeltaChart(data.items, labels, colors);
            this.renderCorrelationCvdChart(data.items, labels, colors);
            this.renderOpenInterestChart(data.items, labels, colors);
            this.renderLongShortRatioChart(data.items, labels, colors);
            this.renderVolumeDeltaChart(data.items, labels, colors);
            this.renderLongShortSpotChart(data.items, labels, colors);
        }

        renderPriceVolumeChart(items, labels, colors) {
            const ctx = document.getElementById('chart-price-volume');
            if (!ctx) return;

            const prices = items.map(d => d.price);
            const spotVols = items.map(d => d.spotVolumeUsdt);
            const futVols = items.map(d => d.futuresVolumeUsdt);

            if (this.charts.priceVolume) {
                const chart = this.charts.priceVolume;
                chart.data.labels = labels;
                chart.data.datasets[0].data = prices;
                chart.data.datasets[1].data = spotVols;
                chart.data.datasets[2].data = futVols;
                chart.update('none');
                return;
            }

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
                    animation: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            labels: { color: colors.text, font: { family: 'Inter', size: 12 } },
                            onClick: (e, legendItem, legend) => {
                                const index = legendItem.datasetIndex;
                                const ci = legend.chart;
                                if (ci.isDatasetVisible(index)) {
                                    ci.hide(index);
                                    legendItem.hidden = true;
                                } else {
                                    ci.show(index);
                                    legendItem.hidden = false;
                                }
                                this.saveVisibility('priceVolume');
                            }
                        },
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
                            ticks: { color: colors.text, callback: val => '$' + formatPrice(val) }
                        },
                        yVolume: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: { color: colors.mutedText, callback: val => '$' + formatNumber(val) }
                        }
                    }
                }
            });
            this.restoreVisibility('priceVolume');
        }

        renderBuySellBreakdownChart(items, labels, colors) {
            const ctx = document.getElementById('chart-buy-sell-breakdown');
            if (!ctx) return;

            const spotBuyVols = items.map(d => d.spotBuyVolumeUsdt);
            const spotSellVols = items.map(d => d.spotSellVolumeUsdt);
            const futBuyVols = items.map(d => d.futuresBuyVolumeUsdt);
            const futSellVols = items.map(d => d.futuresSellVolumeUsdt);
            const futBuyRatios = items.map(d => d.futuresBuyRatio);

            if (this.charts.buySellBreakdown) {
                const chart = this.charts.buySellBreakdown;
                chart.data.labels = labels;
                chart.data.datasets[0].data = futBuyRatios;
                chart.data.datasets[1].data = spotBuyVols;
                chart.data.datasets[2].data = spotSellVols;
                chart.data.datasets[3].data = futBuyVols;
                chart.data.datasets[4].data = futSellVols;
                chart.update('none');
                return;
            }

            this.charts.buySellBreakdown = new Chart(ctx, {
                type: 'bar',
                data: {
                    labels: labels,
                    datasets: [
                        {
                            type: 'line',
                            label: 'Futures Buy/Sell Ratio',
                            data: futBuyRatios,
                            borderColor: colors.lsRatioColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            pointRadius: 0,
                            pointHoverRadius: 4,
                            yAxisID: 'yRatio',
                            order: 1
                        },
                        {
                            type: 'bar',
                            label: 'Spot Buy ($)',
                            data: spotBuyVols,
                            backgroundColor: 'rgba(6, 182, 212, 0.65)',
                            borderColor: '#06b6d4',
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 2
                        },
                        {
                            type: 'bar',
                            label: 'Spot Sell ($)',
                            data: spotSellVols,
                            backgroundColor: 'rgba(59, 130, 246, 0.45)',
                            borderColor: '#3b82f6',
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 3
                        },
                        {
                            type: 'bar',
                            label: 'Futures Buy / Long ($)',
                            data: futBuyVols,
                            backgroundColor: 'rgba(16, 185, 129, 0.7)',
                            borderColor: '#10b981',
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 4
                        },
                        {
                            type: 'bar',
                            label: 'Futures Sell / Short ($)',
                            data: futSellVols,
                            backgroundColor: 'rgba(244, 63, 94, 0.7)',
                            borderColor: '#f43f5e',
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 5
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    animation: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            labels: { color: colors.text, font: { family: 'Inter', size: 11 } },
                            onClick: (e, legendItem, legend) => {
                                const index = legendItem.datasetIndex;
                                const ci = legend.chart;
                                if (ci.isDatasetVisible(index)) {
                                    ci.hide(index);
                                    legendItem.hidden = true;
                                } else {
                                    ci.show(index);
                                    legendItem.hidden = false;
                                }
                                this.saveVisibility('buySellBreakdown');
                            }
                        },
                        tooltip: {
                            callbacks: {
                                label: function (context) {
                                    if (context.dataset.yAxisID === 'yRatio') {
                                        return ` Buy/Sell Ratio: ${Number(context.raw).toFixed(2)}`;
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
                        yVolume: {
                            type: 'linear',
                            position: 'left',
                            grid: { color: colors.grid },
                            ticks: { color: colors.text, callback: val => '$' + formatNumber(val) }
                        },
                        yRatio: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: { color: colors.lsRatioColor, callback: val => Number(val).toFixed(1) }
                        }
                    }
                }
            });
            this.restoreVisibility('buySellBreakdown');
        }

        renderVolumeDeltaChart(items, labels, colors) {
            const ctx = document.getElementById('chart-volume-delta');
            if (!ctx) return;

            const deltas = items.map(d => d.volumeDelta);
            const ratios = items.map(d => d.volumeRatio);
            const barBgColors = deltas.map(v => v >= 0 ? colors.deltaPositive : colors.deltaNegative);

            if (this.charts.volumeDelta) {
                const chart = this.charts.volumeDelta;
                chart.data.labels = labels;
                chart.data.datasets[0].data = ratios;
                chart.data.datasets[1].data = deltas;
                chart.data.datasets[1].backgroundColor = barBgColors;
                chart.update('none');
                return;
            }

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
                    animation: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            labels: { color: colors.text, font: { family: 'Inter', size: 12 } },
                            onClick: (e, legendItem, legend) => {
                                const index = legendItem.datasetIndex;
                                const ci = legend.chart;
                                if (ci.isDatasetVisible(index)) {
                                    ci.hide(index);
                                    legendItem.hidden = true;
                                } else {
                                    ci.show(index);
                                    legendItem.hidden = false;
                                }
                                this.saveVisibility('volumeDelta');
                            }
                        },
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
                            ticks: { color: colors.text, callback: val => '$' + formatNumber(val) }
                        },
                        yRatio: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: { color: colors.futuresColor, callback: val => Number(val).toFixed(1) + 'x' }
                        }
                    }
                }
            });
            this.restoreVisibility('volumeDelta');
        }

        renderLongShortSpotChart(items, labels, colors) {
            const ctx = document.getElementById('chart-long-short-spot');
            if (!ctx) return;

            const spotVols = items.map(d => d.spotVolumeUsdt);
            const longPositions = items.map(d => d.longPositionUsdt);
            const shortPositions = items.map(d => d.shortPositionUsdt);
            const longSpotRatios = items.map(d => d.longSpotRatio);
            const shortSpotRatios = items.map(d => d.shortSpotRatio);

            if (this.charts.longShortSpot) {
                const chart = this.charts.longShortSpot;
                chart.data.labels = labels;
                chart.data.datasets[0].data = longSpotRatios;
                chart.data.datasets[1].data = shortSpotRatios;
                chart.data.datasets[2].data = spotVols;
                chart.data.datasets[3].data = longPositions;
                chart.data.datasets[4].data = shortPositions;
                chart.update('none');
                return;
            }

            this.charts.longShortSpot = new Chart(ctx, {
                data: {
                    labels: labels,
                    datasets: [
                        {
                            type: 'line',
                            label: 'Long / Spot Ratio',
                            data: longSpotRatios,
                            borderColor: colors.longColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            pointRadius: 0,
                            pointHoverRadius: 4,
                            yAxisID: 'yRatio',
                            order: 1
                        },
                        {
                            type: 'line',
                            label: 'Short / Spot Ratio',
                            data: shortSpotRatios,
                            borderColor: colors.shortColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2,
                            borderDash: [3, 3],
                            pointRadius: 0,
                            pointHoverRadius: 4,
                            yAxisID: 'yRatio',
                            order: 2
                        },
                        {
                            type: 'bar',
                            label: 'Spot Volume ($)',
                            data: spotVols,
                            backgroundColor: colors.spotColorBg,
                            borderColor: colors.spotColor,
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 3
                        },
                        {
                            type: 'bar',
                            label: 'Vị thế Long ($)',
                            data: longPositions,
                            backgroundColor: colors.longColorBg,
                            borderColor: colors.longColor,
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 4
                        },
                        {
                            type: 'bar',
                            label: 'Vị thế Short ($)',
                            data: shortPositions,
                            backgroundColor: colors.shortColorBg,
                            borderColor: colors.shortColor,
                            borderWidth: 1,
                            yAxisID: 'yVolume',
                            order: 5
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    animation: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            labels: { color: colors.text, font: { family: 'Inter', size: 12 } },
                            onClick: (e, legendItem, legend) => {
                                const index = legendItem.datasetIndex;
                                const ci = legend.chart;
                                if (ci.isDatasetVisible(index)) {
                                    ci.hide(index);
                                    legendItem.hidden = true;
                                } else {
                                    ci.show(index);
                                    legendItem.hidden = false;
                                }
                                this.saveVisibility('longShortSpot');
                            }
                        },
                        tooltip: {
                            callbacks: {
                                label: function (context) {
                                    if (context.dataset.yAxisID === 'yRatio') {
                                        return ` ${context.dataset.label}: ${Number(context.raw).toFixed(2)}x`;
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
                        yVolume: {
                            type: 'linear',
                            position: 'left',
                            grid: { color: colors.grid },
                            ticks: { color: colors.text, callback: val => '$' + formatNumber(val) }
                        },
                        yRatio: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: { color: colors.mutedText, callback: val => Number(val).toFixed(1) + 'x' }
                        }
                    }
                }
            });
            this.restoreVisibility('longShortSpot');
        }

        renderLongShortRatioChart(items, labels, colors) {
            const ctx = document.getElementById('chart-long-short-ratio');
            if (!ctx) return;

            const longPcts = items.map(d => d.longRatio * 100);
            const shortPcts = items.map(d => d.shortRatio * 100);
            const lsRatios = items.map(d => d.longShortRatio);

            if (this.charts.longShortRatio) {
                const chart = this.charts.longShortRatio;
                chart.data.labels = labels;
                chart.data.datasets[0].data = lsRatios;
                chart.data.datasets[1].data = longPcts;
                chart.data.datasets[2].data = shortPcts;
                chart.update('none');
                return;
            }

            this.charts.longShortRatio = new Chart(ctx, {
                data: {
                    labels: labels,
                    datasets: [
                        {
                            type: 'line',
                            label: 'Tỷ lệ L/S Ratio',
                            data: lsRatios,
                            borderColor: colors.lsRatioColor,
                            backgroundColor: 'transparent',
                            borderWidth: 2.5,
                            pointRadius: 0,
                            pointHoverRadius: 4,
                            yAxisID: 'yRatio',
                            order: 1
                        },
                        {
                            type: 'line',
                            label: '% Vị thế Long',
                            data: longPcts,
                            borderColor: colors.longColor,
                            backgroundColor: colors.longColorBg,
                            fill: true,
                            borderWidth: 1.5,
                            pointRadius: 0,
                            yAxisID: 'yPct',
                            order: 2
                        },
                        {
                            type: 'line',
                            label: '% Vị thế Short',
                            data: shortPcts,
                            borderColor: colors.shortColor,
                            backgroundColor: colors.shortColorBg,
                            fill: true,
                            borderWidth: 1.5,
                            pointRadius: 0,
                            yAxisID: 'yPct',
                            order: 3
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    animation: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            labels: { color: colors.text, font: { family: 'Inter', size: 12 } },
                            onClick: (e, legendItem, legend) => {
                                const index = legendItem.datasetIndex;
                                const ci = legend.chart;
                                if (ci.isDatasetVisible(index)) {
                                    ci.hide(index);
                                    legendItem.hidden = true;
                                } else {
                                    ci.show(index);
                                    legendItem.hidden = false;
                                }
                                this.saveVisibility('longShortRatio');
                            }
                        },
                        tooltip: {
                            callbacks: {
                                label: function (context) {
                                    if (context.dataset.yAxisID === 'yRatio') {
                                        return ` L/S Ratio: ${Number(context.raw).toFixed(2)}`;
                                    }
                                    return ` ${context.dataset.label}: ${Number(context.raw).toFixed(1)}%`;
                                }
                            }
                        }
                    },
                    scales: {
                        x: {
                            grid: { color: colors.grid },
                            ticks: { color: colors.mutedText, maxRotation: 0, autoSkip: true, maxTicksLimit: 12 }
                        },
                        yPct: {
                            type: 'linear',
                            position: 'left',
                            min: 0,
                            max: 100,
                            grid: { color: colors.grid },
                            ticks: { color: colors.text, callback: val => val + '%' }
                        },
                        yRatio: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: { color: colors.lsRatioColor, callback: val => Number(val).toFixed(2) }
                        }
                    }
                }
            });
            this.restoreVisibility('longShortRatio');
        }

        renderOpenInterestChart(items, labels, colors) {
            const ctx = document.getElementById('chart-open-interest');
            if (!ctx) return;

            const prices = items.map(d => d.price);
            const oiUsdt = items.map(d => d.openInterestUsdt);

            if (this.charts.openInterest) {
                const chart = this.charts.openInterest;
                chart.data.labels = labels;
                chart.data.datasets[0].data = prices;
                chart.data.datasets[1].data = oiUsdt;
                chart.update('none');
                return;
            }

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
                    animation: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            labels: { color: colors.text, font: { family: 'Inter', size: 12 } },
                            onClick: (e, legendItem, legend) => {
                                const index = legendItem.datasetIndex;
                                const ci = legend.chart;
                                if (ci.isDatasetVisible(index)) {
                                    ci.hide(index);
                                    legendItem.hidden = true;
                                } else {
                                    ci.show(index);
                                    legendItem.hidden = false;
                                }
                                this.saveVisibility('openInterest');
                            }
                        },
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
                            ticks: { color: colors.text, callback: val => '$' + formatPrice(val) }
                        },
                        yOi: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: { color: colors.oiColor, callback: val => '$' + formatNumber(val) }
                        }
                    }
                }
            });
            this.restoreVisibility('openInterest');
        }

        renderNetTakerDeltaChart(items, labels, colors) {
            const ctx = document.getElementById('chart-net-taker-delta');
            if (!ctx) return;

            const spotDeltas = items.map(d => d.spotNetDelta);
            const futDeltas = items.map(d => d.futuresNetDelta);
            const prices = items.map(d => d.price);

            const spotBgColors = spotDeltas.map(v => v >= 0 ? 'rgba(6, 182, 212, 0.75)' : 'rgba(59, 130, 246, 0.75)');
            const futBgColors = futDeltas.map(v => v >= 0 ? 'rgba(16, 185, 129, 0.75)' : 'rgba(244, 63, 94, 0.75)');
            const futBorderColors = futDeltas.map(v => v >= 0 ? '#10b981' : '#f43f5e');

            if (this.charts.netTakerDelta) {
                const chart = this.charts.netTakerDelta;
                chart.data.labels = labels;
                chart.data.datasets[0].data = prices;
                chart.data.datasets[1].data = spotDeltas;
                chart.data.datasets[1].backgroundColor = spotBgColors;
                chart.data.datasets[2].data = futDeltas;
                chart.data.datasets[2].backgroundColor = futBgColors;
                chart.data.datasets[2].borderColor = futBorderColors;
                chart.update('none');
                return;
            }

            this.charts.netTakerDelta = new Chart(ctx, {
                data: {
                    labels: labels,
                    datasets: [
                        {
                            type: 'line',
                            label: 'Price ($)',
                            data: prices,
                            borderColor: colors.priceColor,
                            backgroundColor: 'transparent',
                            borderWidth: 1.5,
                            pointRadius: 0,
                            yAxisID: 'yPrice',
                            order: 1
                        },
                        {
                            type: 'bar',
                            label: 'Spot Net Delta ($)',
                            data: spotDeltas,
                            backgroundColor: spotBgColors,
                            borderColor: '#06b6d4',
                            borderWidth: 1,
                            yAxisID: 'yDelta',
                            order: 2
                        },
                        {
                            type: 'bar',
                            label: 'Futures Net Delta ($)',
                            data: futDeltas,
                            backgroundColor: futBgColors,
                            borderColor: futBorderColors,
                            borderWidth: 1,
                            yAxisID: 'yDelta',
                            order: 3
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    animation: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            labels: { color: colors.text, font: { family: 'Inter', size: 11 } },
                            onClick: (e, legendItem, legend) => {
                                const index = legendItem.datasetIndex;
                                const ci = legend.chart;
                                if (ci.isDatasetVisible(index)) {
                                    ci.hide(index);
                                    legendItem.hidden = true;
                                } else {
                                    ci.show(index);
                                    legendItem.hidden = false;
                                }
                                this.saveVisibility('netTakerDelta');
                            }
                        },
                        tooltip: {
                            callbacks: {
                                label: function (context) {
                                    if (context.dataset.yAxisID === 'yPrice') {
                                        return ` Price: $${formatPrice(context.raw)}`;
                                    }
                                    const sign = context.raw >= 0 ? '+' : '';
                                    return ` ${context.dataset.label}: ${sign}$${formatNumber(context.raw)}`;
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
                            ticks: { color: colors.text, callback: val => '$' + formatNumber(val) }
                        },
                        yPrice: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: { color: colors.mutedText, callback: val => '$' + formatPrice(val) }
                        }
                    }
                }
            });
            this.restoreVisibility('netTakerDelta');
        }

        renderCorrelationCvdChart(items, labels, colors) {
            const ctx = document.getElementById('chart-correlation-cvd');
            if (!ctx) return;

            const corrs = items.map(d => d.correlation);
            const spotCvd = items.map(d => d.spotCvd);
            const futCvd = items.map(d => d.futuresCvd);

            if (this.charts.correlationCvd) {
                const chart = this.charts.correlationCvd;
                chart.data.labels = labels;
                chart.data.datasets[0].data = corrs;
                chart.data.datasets[1].data = spotCvd;
                chart.data.datasets[2].data = futCvd;
                chart.update('none');
                return;
            }

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
                    animation: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            labels: { color: colors.text, font: { family: 'Inter', size: 12 } },
                            onClick: (e, legendItem, legend) => {
                                const index = legendItem.datasetIndex;
                                const ci = legend.chart;
                                if (ci.isDatasetVisible(index)) {
                                    ci.hide(index);
                                    legendItem.hidden = true;
                                } else {
                                    ci.show(index);
                                    legendItem.hidden = false;
                                }
                                this.saveVisibility('correlationCvd');
                            }
                        },
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
                            ticks: { color: colors.corrColor, callback: val => Number(val).toFixed(2) }
                        },
                        yCvd: {
                            type: 'linear',
                            position: 'right',
                            grid: { drawOnChartArea: false },
                            ticks: { color: colors.mutedText, callback: val => '$' + formatNumber(val) }
                        }
                    }
                }
            });
            this.restoreVisibility('correlationCvd');
        }

        updateTheme() {
            const colors = this.getThemeColors();
            Object.keys(this.charts).forEach(key => {
                const chart = this.charts[key];
                if (!chart) return;

                if (chart.options && chart.options.scales) {
                    Object.keys(chart.options.scales).forEach(scaleKey => {
                        const scale = chart.options.scales[scaleKey];
                        if (scale.grid) scale.grid.color = colors.grid;
                        if (scale.ticks) scale.ticks.color = colors.mutedText;
                    });
                }
                if (chart.options && chart.options.plugins && chart.options.plugins.legend) {
                    chart.options.plugins.legend.labels.color = colors.text;
                }
                chart.update('none');
            });
        }
    }

    // 5. MAIN CONTROLLER & DIAGNOSTICS MANAGER
    class MarketAnalyticsApp {
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

            this.throttleTimer = null;
            this.lastRenderTime = 0;
            this.minRenderIntervalMs = 120; // 60fps throttling

            this.init();
        }

        init() {
            this.bindElements();
            this.bindEvents();
            this.listenThemeChanges();
            this.fetchInitialSnapshot();
            this.startBackgroundSync();

            // Expose globally for developer inspection & test
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
                    renderCount: this.state.renderCount
                }),
                reconnect: () => this.wsManager.subscribe(this.state.symbol, this.state.timeframe),
                fetchSnapshot: () => this.fetchInitialSnapshot(),
                simulateTick: () => this.simulateTestTick()
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

            // Debug Panel Controls
            if (this.el.btnToggleDebug && this.el.debugPanel) {
                this.el.btnToggleDebug.addEventListener('click', () => {
                    this.el.debugPanel.classList.toggle('hidden');
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
                if (this.state.currentData) {
                    this.updateKpiDisplays(this.state.currentData);
                    this.renderTable(this.state.currentData.items);
                }
            });
        }

        listenThemeChanges() {
            const observer = new MutationObserver(() => {
                this.chartManager.updateTheme();
            });
            observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme', 'class'] });
            window.addEventListener('themeChanged', () => {
                this.chartManager.updateTheme();
            });
        }

        addDebugLog(category, message) {
            console.log(`[MarketAnalytics:${category}] ${message}`);

            if (!this.el.dbgLogContainer) return;
            const now = new Date();
            const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}.${String(now.getMilliseconds()).padStart(3, '0')}`;

            let colorClass = 'text-gray-300';
            if (category === 'spot') colorClass = 'text-cyan-400';
            else if (category === 'futures') colorClass = 'text-yellow-400';
            else if (category === 'render') colorClass = 'text-emerald-400';
            else if (category === 'error') colorClass = 'text-rose-400';

            const logLine = document.createElement('div');
            logLine.className = 'py-0.5 border-b border-gray-900 flex items-start space-x-1.5';
            logLine.innerHTML = `<span class="text-gray-500 font-mono">[${timeStr}]</span> <span class="font-bold uppercase text-[10px] px-1 rounded bg-gray-800 text-gray-400">${category}</span> <span class="${colorClass}">${message}</span>`;

            this.el.dbgLogContainer.appendChild(logLine);

            // Keep max 100 lines
            while (this.el.dbgLogContainer.children.length > 100) {
                this.el.dbgLogContainer.removeChild(this.el.dbgLogContainer.firstChild);
            }

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
                    btn.classList.add('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.remove('bg-gray-800/40', 'text-gray-300');
                } else {
                    btn.classList.remove('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.add('bg-gray-800/40', 'text-gray-300');
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
                    btn.classList.add('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.remove('bg-gray-800/40', 'text-gray-300');
                } else {
                    btn.classList.remove('bg-blue-600', 'text-white', 'border-blue-500');
                    btn.classList.add('bg-gray-800/40', 'text-gray-300');
                }
            });
            this.addDebugLog('control', `Switched timeframe to: ${this.state.timeframe}`);
            this.fetchInitialSnapshot();
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

                // Process initial dataset
                const processed = AnalyticsEngine.processAndAlignData(
                    this.state.rawSpotKlines,
                    this.state.rawFutKlines,
                    this.state.rawOiHistory,
                    this.state.rawLsHistory,
                    timeframe
                );
                processed.tickers = this.state.latestTickers;
                this.state.currentData = processed;

                // Render UI & Charts (preserves any toggles)
                this.state.renderCount++;
                this.updateKpiDisplays(processed);
                this.chartManager.renderAll(processed);
                this.renderTable(processed.items);
                this.updateDebugStats();

                // Start Live WebSocket Stream
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
                if (!this.state.autoRefresh) return;
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
                    this.scheduleThrottledUpdate();
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
                return;
            }

            const last = arr[arr.length - 1];
            const lastOpenTime = parseInt(last[0], 10);

            if (lastOpenTime === openTime) {
                // In-place update current candle
                arr[arr.length - 1] = formatted;
            } else if (openTime > lastOpenTime) {
                // New candle
                arr.push(formatted);
                if (arr.length > this.state.limit * 1.5) arr.shift();
            } else {
                const idx = arr.findIndex(item => parseInt(item[0], 10) === openTime);
                if (idx !== -1) {
                    arr[idx] = formatted;
                }
            }
        }

        handleSpotKlineTick(k) {
            this.updateKlineArray(this.state.rawSpotKlines, k);
            if (this.el.dbgSpotLast) {
                this.el.dbgSpotLast.textContent = `$${formatPrice(k.c)} (vol: ${formatNumber(k.q)})`;
            }
            this.scheduleThrottledUpdate();
        }

        handleFuturesKlineTick(k) {
            this.updateKlineArray(this.state.rawFutKlines, k);
            if (this.el.dbgFutLast) {
                this.el.dbgFutLast.textContent = `$${formatPrice(k.c)} (vol: ${formatNumber(k.q)})`;
            }
            this.scheduleThrottledUpdate();
        }

        handleSpotTickerTick(data) {
            if (!this.state.latestTickers.spot) this.state.latestTickers.spot = {};
            if (data.q) this.state.latestTickers.spot.quoteVolume = data.q;
            if (data.c) this.state.latestTickers.spot.lastPrice = data.c;
            this.scheduleThrottledUpdate();
        }

        handleFuturesTickerTick(data) {
            if (!this.state.latestTickers.futures) this.state.latestTickers.futures = {};
            if (data.q) this.state.latestTickers.futures.quoteVolume = data.q;
            if (data.c) this.state.latestTickers.futures.lastPrice = data.c;
            this.scheduleThrottledUpdate();
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

        scheduleThrottledUpdate() {
            const now = performance.now();
            const elapsed = now - this.lastRenderTime;

            if (elapsed >= this.minRenderIntervalMs) {
                this.lastRenderTime = now;
                this.executeProcessAndRender();
            } else {
                if (this.throttleTimer) return;
                this.throttleTimer = setTimeout(() => {
                    this.throttleTimer = null;
                    this.lastRenderTime = performance.now();
                    this.executeProcessAndRender();
                }, this.minRenderIntervalMs - elapsed);
            }
        }

        executeProcessAndRender() {
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
                this.el.dbgLastRenderTime.textContent = `${now.toLocaleTimeString()} (${duration}ms)`;
            }
            this.updateDebugStats();
        }

        updateKpiDisplays(data) {
            const tickers = data.tickers || {};
            const items = data.items;
            const lastItem = items.length > 0 ? items[items.length - 1] : null;

            // Updated at timestamp
            if (this.el.analyticsUpdatedAt) {
                const now = new Date();
                const pad = n => String(n).padStart(2, '0');
                this.el.analyticsUpdatedAt.textContent = `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())} - ${pad(now.getDate())}/${pad(now.getMonth() + 1)}/${now.getFullYear()}`;
            }

            // Spot 24h Vol & Buy/Sell
            let spot24hVol = 0;
            let spotBuy24h = 0;
            let spotSell24h = 0;
            if (this.el.kpiSpotVol) {
                spot24hVol = (tickers.spot && tickers.spot.quoteVolume) ? parseFloat(tickers.spot.quoteVolume) : (lastItem ? lastItem.spotVolumeUsdt * 24 : 0);
                this.el.kpiSpotVol.textContent = '$' + formatNumber(spot24hVol);

                const spotBuyPct = lastItem ? (lastItem.spotBuyPct / 100) : 0.5;
                spotBuy24h = spot24hVol * spotBuyPct;
                spotSell24h = Math.max(0, spot24hVol - spotBuy24h);

                if (this.el.kpiSpotBuySell) {
                    this.el.kpiSpotBuySell.innerHTML = `Mua: <span class="font-bold text-cyan-400">$${formatNumber(spotBuy24h)}</span> | Bán: <span class="font-bold text-blue-400">$${formatNumber(spotSell24h)}</span>`;
                }
            }

            // Futures 24h Vol & Buy/Sell
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
                    this.el.kpiFutBuySell.innerHTML = `Mua: <span class="font-bold text-emerald-400">$${formatNumber(futBuy24h)}</span> | Bán: <span class="font-bold text-rose-400">$${formatNumber(futSell24h)}</span>`;
                }
            }

            // F/S Ratio & Buy/Sell Ratio
            if (this.el.kpiFsRatio) {
                const spotVol = spot24hVol > 0 ? spot24hVol : 1;
                const futVol = fut24hVol > 0 ? fut24hVol : 1;
                const ratio = spotVol > 0 ? (futVol / spotVol) : 0;
                this.el.kpiFsRatio.textContent = ratio.toFixed(2) + 'x';
            }
            if (this.el.kpiBsRatio) {
                const futBsRatio = futSell24h > 0 ? (futBuy24h / futSell24h).toFixed(2) : '--';
                const spotBsRatio = spotSell24h > 0 ? (spotBuy24h / spotSell24h).toFixed(2) : '--';
                this.el.kpiBsRatio.textContent = `Tỷ lệ Mua/Bán: ${futBsRatio} (Fut) | ${spotBsRatio} (Spot)`;
            }

            // Open Interest
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

            // Long Position Card
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

            // Short Position Card
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

            // Correlation
            if (this.el.kpiCorrelation && this.el.kpiCorrelationStatus) {
                const corr = data.overallCorrelation;
                this.el.kpiCorrelation.textContent = corr.toFixed(3);

                const isVi = (document.documentElement.lang || 'vi') === 'vi';
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

            // Market State Badge
            if (this.el.kpiMarketStateBadge && this.el.kpiMarketStateDesc && lastItem) {
                const state = lastItem.marketState;
                const isVi = (document.documentElement.lang || 'vi') === 'vi';

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

        renderTable(items) {
            if (!this.el.dataTableBody) return;

            // Show latest 30 candles in reverse order
            const displayItems = items.slice().reverse().slice(0, 30);

            let html = '';
            displayItems.forEach(row => {
                const priceColor = row.priceChangePct >= 0 ? 'text-emerald-500' : 'text-rose-500';
                const priceSign = row.priceChangePct >= 0 ? '+' : '';

                let stateTag = '';
                switch (row.marketState) {
                    case 'LONG_BUILDUP':
                        stateTag = '<span class="px-2 py-0.5 rounded text-[11px] font-semibold bg-emerald-500/20 text-emerald-500 border border-emerald-500/30">Long Build-up</span>';
                        break;
                    case 'SHORT_SQUEEZE':
                        stateTag = '<span class="px-2 py-0.5 rounded text-[11px] font-semibold bg-yellow-500/20 text-yellow-500 border border-yellow-500/30">Short Squeeze</span>';
                        break;
                    case 'SHORT_BUILDUP':
                        stateTag = '<span class="px-2 py-0.5 rounded text-[11px] font-semibold bg-rose-500/20 text-rose-500 border border-rose-500/30">Short Build-up</span>';
                        break;
                    case 'LONG_LIQUIDATION':
                        stateTag = '<span class="px-2 py-0.5 rounded text-[11px] font-semibold bg-indigo-500/20 text-indigo-500 border border-indigo-500/30">Long Liquidation</span>';
                        break;
                    default:
                        stateTag = '<span class="px-2 py-0.5 rounded text-[11px] font-semibold bg-gray-500/20 text-gray-500 border border-gray-500/30">Neutral</span>';
                }

                html += `
                    <tr class="hover:bg-white/5 dark:hover:bg-black/20 transition-all duration-200" style="border-bottom: 1px solid var(--border-color);">
                        <td class="px-3.5 py-3 text-xs font-medium" style="color: var(--text-primary);">${row.label}</td>
                        <td class="px-3.5 py-3 text-xs font-bold" style="color: var(--text-primary);">$${formatPrice(row.price)} <span class="${priceColor} ml-1 font-semibold">(${priceSign}${row.priceChangePct.toFixed(2)}%)</span></td>
                        <td class="px-3.5 py-3 text-xs text-blue-400 font-semibold">$${formatNumber(row.spotVolumeUsdt)}</td>
                        <td class="px-3.5 py-3 text-xs">
                            <span class="text-cyan-400 font-semibold">$${formatNumber(row.spotBuyVolumeUsdt)}</span> / <span class="text-blue-400 font-semibold">$${formatNumber(row.spotSellVolumeUsdt)}</span>
                            <span class="text-[10px] text-gray-400 ml-1">(${row.spotBuyPct.toFixed(0)}%)</span>
                        </td>
                        <td class="px-3.5 py-3 text-xs text-yellow-400 font-semibold">$${formatNumber(row.futuresVolumeUsdt)}</td>
                        <td class="px-3.5 py-3 text-xs">
                            <span class="text-emerald-400 font-semibold">$${formatNumber(row.futuresBuyVolumeUsdt)}</span> / <span class="text-rose-400 font-semibold">$${formatNumber(row.futuresSellVolumeUsdt)}</span>
                            <span class="text-[10px] text-gray-400 ml-1">(${row.futuresBuyPct.toFixed(0)}%)</span>
                        </td>
                        <td class="px-3.5 py-3 text-xs text-emerald-500 font-semibold">${row.longPositionUsdt !== null ? '$' + formatNumber(row.longPositionUsdt) : '--'}</td>
                        <td class="px-3.5 py-3 text-xs text-rose-500 font-semibold">${row.shortPositionUsdt !== null ? '$' + formatNumber(row.shortPositionUsdt) : '--'}</td>
                        <td class="px-3.5 py-3 text-xs font-bold text-emerald-400">${row.longSpotRatio ? row.longSpotRatio.toFixed(2) + 'x' : '--'}</td>
                        <td class="px-3.5 py-3 text-xs font-bold text-rose-400">${row.shortSpotRatio ? row.shortSpotRatio.toFixed(2) + 'x' : '--'}</td>
                        <td class="px-3.5 py-3 text-xs font-semibold text-indigo-400">${row.longShortRatio ? row.longShortRatio.toFixed(2) : '--'}</td>
                        <td class="px-3.5 py-3 text-xs font-bold" style="color: var(--text-primary);">${row.volumeRatio.toFixed(2)}x</td>
                        <td class="px-3.5 py-3 text-xs text-pink-400 font-semibold">${row.openInterestUsdt ? '$' + formatNumber(row.openInterestUsdt) : '--'}</td>
                        <td class="px-3.5 py-3 text-xs text-center">${stateTag}</td>
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
            const headers = [
                'Timestamp',
                'Date',
                'Price',
                'PriceChangePct',
                'SpotVolumeUSDT',
                'SpotBuyVolumeUSDT',
                'SpotSellVolumeUSDT',
                'SpotBuyRatio',
                'SpotNetDeltaUSDT',
                'FuturesVolumeUSDT',
                'FuturesBuyVolumeUSDT',
                'FuturesSellVolumeUSDT',
                'FuturesBuyRatio',
                'FuturesNetDeltaUSDT',
                'LongPositionUSDT',
                'ShortPositionUSDT',
                'LongRatio',
                'ShortRatio',
                'LongShortRatio',
                'LongSpotRatio',
                'ShortSpotRatio',
                'FuturesSpotRatio',
                'OpenInterestUSDT',
                'OpenInterestCoins',
                'SpotCVD',
                'FuturesCVD',
                'Correlation14',
                'MarketState'
            ];
            const rows = items.map(d => [
                d.timestamp,
                `"${d.label}"`,
                d.price,
                d.priceChangePct.toFixed(2),
                d.spotVolumeUsdt.toFixed(2),
                d.spotBuyVolumeUsdt.toFixed(2),
                d.spotSellVolumeUsdt.toFixed(2),
                d.spotBuyRatio.toFixed(2),
                d.spotNetDelta.toFixed(2),
                d.futuresVolumeUsdt.toFixed(2),
                d.futuresBuyVolumeUsdt.toFixed(2),
                d.futuresSellVolumeUsdt.toFixed(2),
                d.futuresBuyRatio.toFixed(2),
                d.futuresNetDelta.toFixed(2),
                d.longPositionUsdt !== null ? d.longPositionUsdt.toFixed(2) : '',
                d.shortPositionUsdt !== null ? d.shortPositionUsdt.toFixed(2) : '',
                d.longRatio.toFixed(4),
                d.shortRatio.toFixed(4),
                d.longShortRatio ? d.longShortRatio.toFixed(2) : '',
                d.longSpotRatio ? d.longSpotRatio.toFixed(2) : '',
                d.shortSpotRatio ? d.shortSpotRatio.toFixed(2) : '',
                d.volumeRatio.toFixed(2),
                d.openInterestUsdt ? d.openInterestUsdt.toFixed(2) : '',
                d.openInterestCoins ? d.openInterestCoins.toFixed(2) : '',
                d.spotCvd.toFixed(2),
                d.futuresCvd.toFixed(2),
                d.correlation !== null ? d.correlation.toFixed(3) : '',
                `"${d.marketState}"`
            ]);

            const csvContent = 'data:text/csv;charset=utf-8,' + [headers.join(','), ...rows.map(e => e.join(','))].join('\n');
            const encodedUri = encodeURI(csvContent);
            const link = document.createElement('a');
            link.setAttribute('href', encodedUri);
            link.setAttribute('download', `${this.state.symbol}_${this.state.timeframe}_market_analytics.csv`);
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
