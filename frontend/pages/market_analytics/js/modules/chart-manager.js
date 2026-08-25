/**
 * Chart Manager (In-Place Updates with Absolute Legend State Preservation)
 */

import { formatNumber, formatPrice, formatDate, getI18nText } from './utils.js';

// 4. CHART MANAGER (In-Place Updates with Absolute Legend State Preservation)
export class ChartManager {
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
                        grace: '5%',
                        grid: { color: colors.grid },
                        ticks: { color: colors.text, callback: val => '$' + formatPrice(val) }
                    },
                    yVolume: {
                        type: 'linear',
                        position: 'right',
                        beginAtZero: true,
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
                        grace: '5%',
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
                        borderWidth: 2,
                        pointRadius: 0,
                        pointHoverRadius: 5,
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
                    yPrice: {
                        type: 'linear',
                        position: 'left',
                        grace: '5%',
                        grid: { color: colors.grid },
                        ticks: { color: colors.text, callback: val => '$' + formatPrice(val) }
                    },
                    yDelta: {
                        type: 'linear',
                        position: 'right',
                        grid: { drawOnChartArea: false },
                        ticks: { color: colors.mutedText, callback: val => '$' + formatNumber(val) }
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

