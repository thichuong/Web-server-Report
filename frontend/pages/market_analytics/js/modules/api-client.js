/**
 * Binance REST API Client for Market Analytics
 */

// 1. BINANCE REST API CLIENT (For initial snapshots and background sync)
export class BinanceApiClient {
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

