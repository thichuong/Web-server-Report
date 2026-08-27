/**
 * Mathematical & Analytical Computation Engine (Optimized Incremental & Binary Search)
 */

import { formatDate } from './utils.js';

export class AnalyticsEngine {
    /**
     * Pearson correlation coefficient between two arrays
     */
    static calculatePearsonCorrelation(x, y) {
        const n = x.length;
        if (n < 2) return 0;

        let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0, sumY2 = 0;
        for (let i = 0; i < n; i++) {
            const xi = x[i];
            const yi = y[i];
            sumX += xi;
            sumY += yi;
            sumXY += xi * yi;
            sumX2 += xi * xi;
            sumY2 += yi * yi;
        }

        const numerator = n * sumXY - sumX * sumY;
        const denominator = Math.sqrt((n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY));

        if (denominator === 0) return 0;
        const corr = numerator / denominator;
        return Math.max(-1, Math.min(1, corr));
    }

    /**
     * Rolling Pearson correlation across window
     */
    static calculateRollingCorrelation(spotVol, futVol, window = 14) {
        const len = spotVol.length;
        const rolling = new Array(len);
        for (let i = 0; i < len; i++) {
            if (i < window - 1) {
                rolling[i] = null;
            } else {
                const sliceX = spotVol.slice(i - window + 1, i + 1);
                const sliceY = futVol.slice(i - window + 1, i + 1);
                rolling[i] = this.calculatePearsonCorrelation(sliceX, sliceY);
            }
        }
        return rolling;
    }

    /**
     * Binary search helper: Find the latest record with timestamp <= targetTs
     */
    static findFloorEntry(sortedKeys, map, targetTs) {
        if (!sortedKeys || sortedKeys.length === 0) return null;
        if (map.has(targetTs)) return map.get(targetTs);

        let low = 0;
        let high = sortedKeys.length - 1;
        let bestIndex = -1;

        while (low <= high) {
            const mid = (low + high) >> 1;
            if (sortedKeys[mid] <= targetTs) {
                bestIndex = mid;
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        return bestIndex !== -1 ? map.get(sortedKeys[bestIndex]) : null;
    }

    /**
     * Format a single raw kline array to structured object
     */
    static parseKline(k) {
        if (!k) return null;
        return {
            openTime: parseInt(k[0], 10),
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
        };
    }

    /**
     * Full dataset processing and alignment (Used on initial load or new candle open)
     */
    static processAndAlignData(spotKlines, futKlines, oiHistory, lsHistory, timeframe) {
        const spotMap = new Map();
        spotKlines.forEach(k => {
            const parsed = this.parseKline(k);
            if (parsed) spotMap.set(parsed.openTime, parsed);
        });

        const futMap = new Map();
        futKlines.forEach(k => {
            const parsed = this.parseKline(k);
            if (parsed) futMap.set(parsed.openTime, parsed);
        });

        // Sorted OI map and index keys
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
        const sortedOiKeys = Array.from(oiMap.keys()).sort((a, b) => a - b);

        // Sorted Long/Short map and index keys
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
        const sortedLsKeys = Array.from(lsMap.keys()).sort((a, b) => a - b);

        // Merge all timestamps
        const allTimestamps = new Set([...spotMap.keys(), ...futMap.keys()]);
        const sortedTimestamps = Array.from(allTimestamps).sort((a, b) => a - b);
        const alignedData = [];

        let spotCumulativeCvd = 0;
        let futCumulativeCvd = 0;

        for (let i = 0; i < sortedTimestamps.length; i++) {
            const ts = sortedTimestamps[i];
            const futCandle = futMap.get(ts);
            const spotCandle = spotMap.get(ts);

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

            spotCumulativeCvd += spotNetDelta;
            futCumulativeCvd += futNetDelta;

            // Fast Binary Search for OI and LS
            const matchedOi = this.findFloorEntry(sortedOiKeys, oiMap, ts);
            const matchedLs = this.findFloorEntry(sortedLsKeys, lsMap, ts);

            const longRatio = matchedLs ? matchedLs.longRatio : 0.5;
            const shortRatio = matchedLs ? matchedLs.shortRatio : 0.5;
            const lsRatio = matchedLs ? matchedLs.longShortRatio : 1.0;

            const openInterestCoins = matchedOi ? matchedOi.sumOpenInterest : null;
            const openInterestUsdt = matchedOi ? matchedOi.sumOpenInterestValue : null;

            const longPositionUsdt = openInterestUsdt !== null ? (openInterestUsdt * longRatio) : null;
            const shortPositionUsdt = openInterestUsdt !== null ? (openInterestUsdt * shortRatio) : null;

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
                        state = 'SHORT_LIQUIDATION';
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

        const overallCorrelation = this.calculatePearsonCorrelation(spotVols, futVols);

        return {
            items: alignedData,
            overallCorrelation,
            sortedOiKeys,
            oiMap,
            sortedLsKeys,
            lsMap
        };
    }

    /**
     * O(1) Fast incremental update for real-time live ticks on the current active candle
     */
    static updateLatestCandle(currentData, rawSpotKlines, rawFutKlines, timeframe) {
        if (!currentData || !currentData.items || !currentData.items.length) {
            return null;
        }

        const items = currentData.items;
        const lastIdx = items.length - 1;
        const lastItem = items[lastIdx];
        const prevItem = lastIdx > 0 ? items[lastIdx - 1] : null;

        const rawSpot = rawSpotKlines.length > 0 ? rawSpotKlines[rawSpotKlines.length - 1] : null;
        const rawFut = rawFutKlines.length > 0 ? rawFutKlines[rawFutKlines.length - 1] : null;

        const spot = this.parseKline(rawSpot) || this.parseKline(rawFut);
        const fut = this.parseKline(rawFut) || this.parseKline(rawSpot);
        if (!spot && !fut) return currentData;

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

        // In-place update latest candle fields
        lastItem.price = futPrice;
        lastItem.spotPrice = spotPrice;
        lastItem.high = fut ? fut.high : (spot ? spot.high : lastItem.high);
        lastItem.low = fut ? fut.low : (spot ? spot.low : lastItem.low);
        lastItem.close = fut ? fut.close : (spot ? spot.close : lastItem.close);
        lastItem.futuresBaseVolume = fut ? fut.volume : lastItem.futuresBaseVolume;
        lastItem.spotBaseVolume = spot ? spot.volume : lastItem.spotBaseVolume;
        lastItem.spotVolumeUsdt = spotQuoteVol;
        lastItem.spotBuyVolumeUsdt = spotBuyQuoteVol;
        lastItem.spotSellVolumeUsdt = spotSellQuoteVol;
        lastItem.spotNetDelta = spotNetDelta;
        lastItem.spotBuyRatio = spotBuyRatio;
        lastItem.spotBuyPct = spotBuyPct;
        lastItem.spotTrades = spot ? spot.trades : lastItem.spotTrades;
        lastItem.futuresVolumeUsdt = futQuoteVol;
        lastItem.futuresBuyVolumeUsdt = futBuyQuoteVol;
        lastItem.futuresSellVolumeUsdt = futSellQuoteVol;
        lastItem.futuresNetDelta = futNetDelta;
        lastItem.futuresBuyRatio = futBuyRatio;
        lastItem.futuresBuyPct = futBuyPct;
        lastItem.volumeDelta = volumeDelta;
        lastItem.volumeRatio = volumeRatio;
        lastItem.tradesCount = fut ? fut.trades : (spot ? spot.trades : lastItem.tradesCount);

        // CVD update
        const prevSpotCvd = prevItem ? prevItem.spotCvd : 0;
        const prevFutCvd = prevItem ? prevItem.futuresCvd : 0;
        lastItem.spotCvd = prevSpotCvd + spotNetDelta;
        lastItem.futuresCvd = prevFutCvd + futNetDelta;

        // Ratio vs Spot
        const longPos = lastItem.longPositionUsdt;
        const shortPos = lastItem.shortPositionUsdt;
        lastItem.longSpotRatio = (spotQuoteVol > 0 && longPos !== null) ? (longPos / spotQuoteVol) : 0;
        lastItem.shortSpotRatio = (spotQuoteVol > 0 && shortPos !== null) ? (shortPos / spotQuoteVol) : 0;

        // Price change & Market State
        if (prevItem) {
            const prevPrice = prevItem.price;
            const priceChange = futPrice - prevPrice;
            lastItem.priceChangePct = prevPrice > 0 ? (priceChange / prevPrice) * 100 : 0;

            const currOi = lastItem.openInterestUsdt;
            const prevOi = prevItem.openInterestUsdt;
            if (currOi !== null && prevOi !== null && prevOi > 0) {
                const oiChange = currOi - prevOi;
                lastItem.oiChangePct = (oiChange / prevOi) * 100;
                if (priceChange >= 0 && oiChange >= 0) {
                    lastItem.marketState = 'LONG_BUILDUP';
                } else if (priceChange >= 0 && oiChange < 0) {
                    lastItem.marketState = 'SHORT_LIQUIDATION';
                } else if (priceChange < 0 && oiChange >= 0) {
                    lastItem.marketState = 'SHORT_BUILDUP';
                } else if (priceChange < 0 && oiChange < 0) {
                    lastItem.marketState = 'LONG_LIQUIDATION';
                }
            }
        }

        // Fast rolling Pearson correlation for the latest point only
        if (items.length >= 14) {
            const recent14 = items.slice(-14);
            const x = recent14.map(d => d.spotVolumeUsdt);
            const y = recent14.map(d => d.futuresVolumeUsdt);
            lastItem.correlation = this.calculatePearsonCorrelation(x, y);
        }

        return currentData;
    }
}
