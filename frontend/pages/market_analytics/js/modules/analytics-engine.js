/**
 * Mathematical & Analytical Computation Engine
 */

import { formatDate } from './utils.js';

// 3. MATHEMATICAL & ANALYTICAL COMPUTATION ENGINE
export class AnalyticsEngine {
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
                open: fut ? fut.open : (spot ? spot.open : 0),
                high: fut ? fut.high : (spot ? spot.high : 0),
                low: fut ? fut.low : (spot ? spot.low : 0),
                close: fut ? fut.close : (spot ? spot.close : 0),
                futuresBaseVolume: fut ? fut.volume : 0,
                spotBaseVolume: spot ? spot.volume : 0,
                spotOpen: spot ? spot.open : (fut ? fut.open : 0),
                spotHigh: spot ? spot.high : (fut ? fut.high : 0),
                spotLow: spot ? spot.low : (fut ? fut.low : 0),
                spotClose: spot ? spot.close : (fut ? fut.close : 0),
                spotVolumeUsdt: spotQuoteVol,
                spotBuyVolumeUsdt: spotBuyQuoteVol,
                spotSellVolumeUsdt: spotSellQuoteVol,
                spotNetDelta: spotNetDelta,
                spotBuyRatio: spotBuyRatio,
                spotBuyPct: spotBuyPct,
                spotTrades: spot ? spot.trades : 0,
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
                tradesCount: fut ? fut.trades : (spot ? spot.trades : 0)
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

