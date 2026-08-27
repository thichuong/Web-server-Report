/**
 * Table Column Metadata Registry & View Presets
 */

import { formatNumber, formatPrice } from './utils.js';

// Table Column Metadata Registry & Formatters
const TABLE_COLUMNS = [
    // Group 1: Price & Candlestick
    {
        id: 'time',
        groupId: 'price',
        groupI18n: 'column-group-price',
        labelI18n: 'th-time',
        labelDefault: 'Thời gian',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-gray-400 whitespace-nowrap',
        defaultVisible: true,
        fixed: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs font-medium whitespace-nowrap text-gray-900 dark:text-gray-100">${row.label}</td>`
    },
    {
        id: 'price',
        groupId: 'price',
        groupI18n: 'column-group-price',
        labelI18n: 'th-close-price',
        labelDefault: 'Giá Đóng Cửa',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-slate-800 dark:text-gray-300 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => {
            const priceColor = row.priceChangePct >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400';
            const priceSign = row.priceChangePct >= 0 ? '+' : '';
            return `<td class="px-3.5 py-2.5 text-xs font-bold whitespace-nowrap text-gray-900 dark:text-gray-100">$${formatPrice(row.price)} <span class="${priceColor} ml-1 font-bold text-[11px]">(${priceSign}${row.priceChangePct.toFixed(2)}%)</span></td>`;
        }
    },
    {
        id: 'ohlc',
        groupId: 'price',
        groupI18n: 'column-group-price',
        labelI18n: 'th-ohlc',
        labelDefault: 'Nến OHLC ($)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-indigo-700 dark:text-indigo-400 whitespace-nowrap',
        defaultVisible: false,
        render: (row) => {
            return `<td class="px-3.5 py-2.5 text-[11px] whitespace-nowrap text-gray-600 dark:text-gray-400">
                O: <span class="text-gray-900 dark:text-gray-200 font-semibold">$${formatPrice(row.open)}</span>
                H: <span class="text-emerald-700 dark:text-emerald-400 font-semibold">$${formatPrice(row.high)}</span>
                L: <span class="text-rose-700 dark:text-rose-400 font-semibold">$${formatPrice(row.low)}</span>
                C: <span class="text-blue-700 dark:text-blue-400 font-semibold">$${formatPrice(row.close)}</span>
            </td>`;
        }
    },
    // Group 2: Volume & Taker Flow
    {
        id: 'spotVol',
        groupId: 'flow',
        groupI18n: 'column-group-flow',
        labelI18n: 'th-spot-vol',
        labelDefault: 'Spot Vol ($)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-blue-700 dark:text-blue-400 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs text-blue-700 dark:text-blue-400 font-bold whitespace-nowrap">$${formatNumber(row.spotVolumeUsdt)}</td>`
    },
    {
        id: 'spotBuySell',
        groupId: 'flow',
        groupI18n: 'column-group-flow',
        labelI18n: 'th-spot-buy-sell',
        labelDefault: 'Spot Mua / Bán',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-teal-700 dark:text-cyan-400 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs whitespace-nowrap">
            <span class="text-teal-700 dark:text-cyan-400 font-semibold">$${formatNumber(row.spotBuyVolumeUsdt)}</span> <span class="text-gray-400">/</span> <span class="text-blue-700 dark:text-blue-400 font-semibold">$${formatNumber(row.spotSellVolumeUsdt)}</span>
            <span class="text-[10px] text-gray-600 dark:text-gray-400 ml-1 font-medium">(${row.spotBuyPct.toFixed(0)}%)</span>
        </td>`
    },
    {
        id: 'spotDelta',
        groupId: 'flow',
        groupI18n: 'column-group-flow',
        labelI18n: 'th-spot-delta',
        labelDefault: 'Spot Net Delta',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-teal-700 dark:text-cyan-300 whitespace-nowrap',
        defaultVisible: false,
        render: (row) => {
            const isPos = row.spotNetDelta >= 0;
            const col = isPos ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400';
            const sign = isPos ? '+' : '';
            return `<td class="px-3.5 py-2.5 text-xs font-bold whitespace-nowrap ${col}">${sign}$${formatNumber(row.spotNetDelta)}</td>`;
        }
    },
    {
        id: 'futVol',
        groupId: 'flow',
        groupI18n: 'column-group-flow',
        labelI18n: 'th-fut-vol',
        labelDefault: 'Futures Vol ($)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-amber-700 dark:text-yellow-400 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs text-amber-700 dark:text-yellow-400 font-bold whitespace-nowrap">$${formatNumber(row.futuresVolumeUsdt)}</td>`
    },
    {
        id: 'futBuySell',
        groupId: 'flow',
        groupI18n: 'column-group-flow',
        labelI18n: 'th-fut-buy-sell',
        labelDefault: 'Fut Mua / Bán',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-amber-700 dark:text-amber-400 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs whitespace-nowrap">
            <span class="text-emerald-700 dark:text-emerald-400 font-semibold">$${formatNumber(row.futuresBuyVolumeUsdt)}</span> <span class="text-gray-400">/</span> <span class="text-rose-700 dark:text-rose-400 font-semibold">$${formatNumber(row.futuresSellVolumeUsdt)}</span>
            <span class="text-[10px] text-gray-600 dark:text-gray-400 ml-1 font-medium">(${row.futuresBuyPct.toFixed(0)}%)</span>
        </td>`
    },
    {
        id: 'futDelta',
        groupId: 'flow',
        groupI18n: 'column-group-flow',
        labelI18n: 'th-fut-delta',
        labelDefault: 'Fut Net Delta',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-amber-700 dark:text-amber-300 whitespace-nowrap',
        defaultVisible: false,
        render: (row) => {
            const isPos = row.futuresNetDelta >= 0;
            const col = isPos ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400';
            const sign = isPos ? '+' : '';
            return `<td class="px-3.5 py-2.5 text-xs font-bold whitespace-nowrap ${col}">${sign}$${formatNumber(row.futuresNetDelta)}</td>`;
        }
    },
    {
        id: 'cvd',
        groupId: 'flow',
        groupI18n: 'column-group-flow',
        labelI18n: 'th-cvd',
        labelDefault: 'Spot & Fut CVD',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-orange-700 dark:text-orange-400 whitespace-nowrap',
        defaultVisible: false,
        render: (row) => {
            const sCol = row.spotCvd >= 0 ? 'text-teal-700 dark:text-cyan-400' : 'text-rose-700 dark:text-cyan-600';
            const fCol = row.futuresCvd >= 0 ? 'text-amber-700 dark:text-amber-400' : 'text-rose-700 dark:text-rose-400';
            return `<td class="px-3.5 py-2.5 text-xs whitespace-nowrap">
                <span class="${sCol} font-semibold">S: $${formatNumber(row.spotCvd)}</span> | <span class="${fCol} font-semibold">F: $${formatNumber(row.futuresCvd)}</span>
            </td>`;
        }
    },
    // Group 3: Positions & Leverage
    {
        id: 'longPos',
        groupId: 'positions',
        groupI18n: 'column-group-positions',
        labelI18n: 'th-long-pos',
        labelDefault: 'Vị thế Long ($)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-emerald-700 dark:text-emerald-500 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs text-emerald-700 dark:text-emerald-400 font-bold whitespace-nowrap">${row.longPositionUsdt !== null ? '$' + formatNumber(row.longPositionUsdt) : '--'}</td>`
    },
    {
        id: 'shortPos',
        groupId: 'positions',
        groupI18n: 'column-group-positions',
        labelI18n: 'th-short-pos',
        labelDefault: 'Vị thế Short ($)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-rose-700 dark:text-rose-500 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs text-rose-700 dark:text-rose-400 font-bold whitespace-nowrap">${row.shortPositionUsdt !== null ? '$' + formatNumber(row.shortPositionUsdt) : '--'}</td>`
    },
    {
        id: 'netPos',
        groupId: 'positions',
        groupI18n: 'column-group-positions',
        labelI18n: 'th-net-pos',
        labelDefault: 'Vị thế Net ($)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-teal-700 dark:text-teal-400 whitespace-nowrap',
        defaultVisible: false,
        render: (row) => {
            if (row.netPositionUsdt === null || row.netPositionUsdt === undefined) {
                return `<td class="px-3.5 py-2.5 text-xs text-gray-500 whitespace-nowrap">--</td>`;
            }
            const isPos = row.netPositionUsdt >= 0;
            const col = isPos ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400';
            const sign = isPos ? '+' : '';
            return `<td class="px-3.5 py-2.5 text-xs font-bold whitespace-nowrap ${col}">${sign}$${formatNumber(row.netPositionUsdt)}</td>`;
        }
    },
    {
        id: 'longSpot',
        groupId: 'positions',
        groupI18n: 'column-group-positions',
        labelI18n: 'th-long-spot',
        labelDefault: 'Long / Spot',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-emerald-700 dark:text-emerald-400 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs font-bold text-emerald-700 dark:text-emerald-400 whitespace-nowrap">${row.longSpotRatio ? row.longSpotRatio.toFixed(2) + 'x' : '--'}</td>`
    },
    {
        id: 'shortSpot',
        groupId: 'positions',
        groupI18n: 'column-group-positions',
        labelI18n: 'th-short-spot',
        labelDefault: 'Short / Spot',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-rose-700 dark:text-rose-400 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs font-bold text-rose-700 dark:text-rose-400 whitespace-nowrap">${row.shortSpotRatio ? row.shortSpotRatio.toFixed(2) + 'x' : '--'}</td>`
    },
    {
        id: 'lsRatio',
        groupId: 'positions',
        groupI18n: 'column-group-positions',
        labelI18n: 'th-ls-ratio',
        labelDefault: 'Tỷ lệ L/S',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-indigo-700 dark:text-indigo-400 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs font-bold text-indigo-700 dark:text-indigo-400 whitespace-nowrap">${row.longShortRatio ? row.longShortRatio.toFixed(2) : '--'}</td>`
    },
    {
        id: 'fsRatio',
        groupId: 'positions',
        groupI18n: 'column-group-positions',
        labelI18n: 'th-fs-ratio',
        labelDefault: 'Tỷ lệ F/S',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-slate-800 dark:text-gray-300 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs font-bold whitespace-nowrap text-gray-900 dark:text-gray-100">${row.volumeRatio.toFixed(2)}x</td>`
    },
    // Group 4: OI & Market State
    {
        id: 'oi',
        groupId: 'market',
        groupI18n: 'column-group-market',
        labelI18n: 'open-interest',
        labelDefault: 'Vị thế mở (OI)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-fuchsia-700 dark:text-pink-400 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => `<td class="px-3.5 py-2.5 text-xs text-fuchsia-700 dark:text-pink-400 font-extrabold whitespace-nowrap">${row.openInterestUsdt ? '$' + formatNumber(row.openInterestUsdt) : '--'}</td>`
    },
    {
        id: 'oiChange',
        groupId: 'market',
        groupI18n: 'column-group-market',
        labelI18n: 'th-oi-change',
        labelDefault: 'Biến động OI (%)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-purple-700 dark:text-purple-400 whitespace-nowrap',
        defaultVisible: false,
        render: (row) => {
            if (row.oiChangePct === undefined || row.oiChangePct === null) {
                return `<td class="px-3.5 py-2.5 text-xs text-gray-500 whitespace-nowrap">--</td>`;
            }
            const isPos = row.oiChangePct >= 0;
            const col = isPos ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400';
            const sign = isPos ? '+' : '';
            return `<td class="px-3.5 py-2.5 text-xs font-bold whitespace-nowrap ${col}">${sign}${row.oiChangePct.toFixed(2)}%</td>`;
        }
    },
    {
        id: 'correlation',
        groupId: 'market',
        groupI18n: 'column-group-market',
        labelI18n: 'th-correlation',
        labelDefault: 'Tương quan (14)',
        thClass: 'px-3.5 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-purple-700 dark:text-purple-400 whitespace-nowrap',
        defaultVisible: false,
        render: (row) => {
            if (row.correlation === undefined || row.correlation === null) {
                return `<td class="px-3.5 py-2.5 text-xs text-gray-500 whitespace-nowrap">--</td>`;
            }
            let col = 'text-purple-700 dark:text-purple-400';
            if (row.correlation > 0.7) col = 'text-emerald-700 dark:text-emerald-400';
            else if (row.correlation < 0.3) col = 'text-rose-700 dark:text-rose-400';
            return `<td class="px-3.5 py-2.5 text-xs font-bold whitespace-nowrap ${col}">${row.correlation.toFixed(3)}</td>`;
        }
    },
    {
        id: 'marketState',
        groupId: 'market',
        groupI18n: 'column-group-market',
        labelI18n: 'market-state',
        labelDefault: 'Trạng thái',
        thClass: 'px-3.5 py-3.5 text-center text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-gray-300 whitespace-nowrap',
        defaultVisible: true,
        render: (row) => {
            let stateTag = '';
            switch (row.marketState) {
                case 'LONG_BUILDUP':
                    stateTag = '<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-100 text-emerald-800 border border-emerald-300 dark:bg-emerald-500/20 dark:text-emerald-400 dark:border-emerald-500/30">🟢 Long Build-up</span>';
                    break;
                case 'SHORT_SQUEEZE':
                    stateTag = '<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-amber-100 text-amber-800 border border-amber-300 dark:bg-yellow-500/20 dark:text-yellow-400 dark:border-yellow-500/30">🟡 Short Squeeze</span>';
                    break;
                case 'SHORT_BUILDUP':
                    stateTag = '<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-rose-100 text-rose-800 border border-rose-300 dark:bg-rose-500/20 dark:text-rose-400 dark:border-rose-500/30">🔴 Short Build-up</span>';
                    break;
                case 'LONG_LIQUIDATION':
                    stateTag = '<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-indigo-100 text-indigo-800 border border-indigo-300 dark:bg-indigo-500/20 dark:text-indigo-400 dark:border-indigo-500/30">⚪ Long Liquidation</span>';
                    break;
                default:
                    stateTag = '<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-gray-100 text-gray-700 border border-gray-300 dark:bg-gray-500/20 dark:text-gray-400 dark:border-gray-500/30">⚪ Neutral</span>';
            }
            return `<td class="px-3.5 py-2.5 text-xs text-center whitespace-nowrap">${stateTag}</td>`;
        }
    }
];

// Stacked Multi-line Grouped Columns Definition (Compact View)
const STACKED_COLUMNS = [
    {
        id: 'time_state',
        labelI18n: 'th-group-time-state',
        labelDefault: 'Thời Gian & Trạng Thái',
        thClass: 'px-4 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-gray-400 whitespace-nowrap',
        render: (row) => {
            let stateTag = '';
            switch (row.marketState) {
                case 'LONG_BUILDUP':
                    stateTag = '<span class="px-2.5 py-0.5 rounded text-xs font-bold bg-emerald-100 text-emerald-800 border border-emerald-300 dark:bg-emerald-500/20 dark:text-emerald-400 dark:border-emerald-500/30">🟢 Long Build-up</span>';
                    break;
                case 'SHORT_SQUEEZE':
                    stateTag = '<span class="px-2.5 py-0.5 rounded text-xs font-bold bg-amber-100 text-amber-800 border border-amber-300 dark:bg-yellow-500/20 dark:text-yellow-400 dark:border-yellow-500/30">🟡 Short Squeeze</span>';
                    break;
                case 'SHORT_BUILDUP':
                    stateTag = '<span class="px-2.5 py-0.5 rounded text-xs font-bold bg-rose-100 text-rose-800 border border-rose-300 dark:bg-rose-500/20 dark:text-rose-400 dark:border-rose-500/30">🔴 Short Build-up</span>';
                    break;
                case 'LONG_LIQUIDATION':
                    stateTag = '<span class="px-2.5 py-0.5 rounded text-xs font-bold bg-indigo-100 text-indigo-800 border border-indigo-300 dark:bg-indigo-500/20 dark:text-indigo-400 dark:border-indigo-500/30">⚪ Long Liquidation</span>';
                    break;
                default:
                    stateTag = '<span class="px-2.5 py-0.5 rounded text-xs font-bold bg-gray-100 text-gray-700 border border-gray-300 dark:bg-gray-500/20 dark:text-gray-400 dark:border-gray-500/30">⚪ Neutral</span>';
            }
            return `
                <td class="px-4 py-3 whitespace-nowrap text-xs">
                    <div class="font-bold text-xs text-gray-900 dark:text-gray-100">${row.label}</div>
                    <div class="mt-1.5">${stateTag}</div>
                </td>
            `;
        }
    },
    {
        id: 'price_range',
        labelI18n: 'th-group-price-range',
        labelDefault: 'Giá & Dải Nến (OHLC)',
        thClass: 'px-4 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-slate-800 dark:text-gray-200 whitespace-nowrap',
        render: (row) => {
            const priceColor = row.priceChangePct >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400';
            const priceSign = row.priceChangePct >= 0 ? '+' : '';
            return `
                <td class="px-4 py-3 whitespace-nowrap text-xs">
                    <div class="font-extrabold text-sm text-gray-900 dark:text-gray-100 flex items-center space-x-1.5">
                        <span>$${formatPrice(row.price)}</span>
                        <span class="${priceColor} text-xs font-bold">(${priceSign}${row.priceChangePct.toFixed(2)}%)</span>
                    </div>
                    <div class="mt-1.5 text-xs text-gray-600 dark:text-gray-400 font-mono space-x-2">
                        <span>O: <span class="text-gray-900 dark:text-gray-200 font-bold">$${formatPrice(row.open)}</span></span>
                        <span>H: <span class="text-emerald-700 dark:text-emerald-400 font-bold">$${formatPrice(row.high)}</span></span>
                        <span>L: <span class="text-rose-700 dark:text-rose-400 font-bold">$${formatPrice(row.low)}</span></span>
                    </div>
                </td>
            `;
        }
    },
    {
        id: 'spot_flow',
        labelI18n: 'th-group-spot-flow',
        labelDefault: 'Spot Vol & Dòng Tiền Taker',
        thClass: 'px-4 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-blue-700 dark:text-blue-400 whitespace-nowrap',
        render: (row) => {
            const deltaPos = row.spotNetDelta >= 0;
            const deltaCol = deltaPos ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400';
            const deltaSign = deltaPos ? '+' : '';
            const cvdCol = row.spotCvd >= 0 ? 'text-teal-700 dark:text-cyan-400' : 'text-rose-700 dark:text-cyan-600';
            return `
                <td class="px-4 py-3 whitespace-nowrap text-xs">
                    <div class="flex items-center space-x-1.5 text-xs">
                        <span class="text-gray-600 dark:text-gray-400 font-medium">Vol:</span>
                        <span class="font-bold text-blue-700 dark:text-blue-400 text-xs">$${formatNumber(row.spotVolumeUsdt)}</span>
                    </div>
                    <div class="mt-1.5 text-xs flex items-center space-x-1">
                        <span class="text-teal-700 dark:text-cyan-400 font-bold">$${formatNumber(row.spotBuyVolumeUsdt)}</span>
                        <span class="text-gray-400">/</span>
                        <span class="text-blue-700 dark:text-blue-400 font-bold">$${formatNumber(row.spotSellVolumeUsdt)}</span>
                        <span class="text-gray-600 dark:text-gray-400 text-xs font-semibold ml-0.5">(${row.spotBuyPct.toFixed(0)}% Buy)</span>
                    </div>
                    <div class="mt-1.5 text-xs space-x-2 font-mono">
                        <span class="text-gray-600 dark:text-gray-400">Δ: <span class="${deltaCol} font-bold">${deltaSign}$${formatNumber(row.spotNetDelta)}</span></span>
                        <span class="text-gray-600 dark:text-gray-400">CVD: <span class="${cvdCol} font-bold">$${formatNumber(row.spotCvd)}</span></span>
                    </div>
                </td>
            `;
        }
    },
    {
        id: 'fut_flow',
        labelI18n: 'th-group-fut-flow',
        labelDefault: 'Futures Vol & Dòng Tiền Taker',
        thClass: 'px-4 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-amber-700 dark:text-yellow-400 whitespace-nowrap',
        render: (row) => {
            const deltaPos = row.futuresNetDelta >= 0;
            const deltaCol = deltaPos ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400';
            const deltaSign = deltaPos ? '+' : '';
            const cvdCol = row.futuresCvd >= 0 ? 'text-amber-700 dark:text-amber-400' : 'text-rose-700 dark:text-rose-400';
            return `
                <td class="px-4 py-3 whitespace-nowrap text-xs">
                    <div class="flex items-center space-x-2 text-xs">
                        <span class="text-gray-600 dark:text-gray-400 font-medium">Vol:</span>
                        <span class="font-bold text-amber-700 dark:text-yellow-400 text-xs">$${formatNumber(row.futuresVolumeUsdt)}</span>
                        <span class="px-2 py-0.5 text-xs rounded bg-indigo-100 text-indigo-700 border border-indigo-200 dark:bg-indigo-500/20 dark:text-indigo-300 dark:border-indigo-500/30 font-bold">${row.volumeRatio.toFixed(2)}x F/S</span>
                    </div>
                    <div class="mt-1.5 text-xs flex items-center space-x-1">
                        <span class="text-emerald-700 dark:text-emerald-400 font-bold">$${formatNumber(row.futuresBuyVolumeUsdt)}</span>
                        <span class="text-gray-400">/</span>
                        <span class="text-rose-700 dark:text-rose-400 font-bold">$${formatNumber(row.futuresSellVolumeUsdt)}</span>
                        <span class="text-gray-600 dark:text-gray-400 text-xs font-semibold ml-0.5">(${row.futuresBuyPct.toFixed(0)}% Buy)</span>
                    </div>
                    <div class="mt-1.5 text-xs space-x-2 font-mono">
                        <span class="text-gray-600 dark:text-gray-400">Δ: <span class="${deltaCol} font-bold">${deltaSign}$${formatNumber(row.futuresNetDelta)}</span></span>
                        <span class="text-gray-600 dark:text-gray-400">CVD: <span class="${cvdCol} font-bold">$${formatNumber(row.futuresCvd)}</span></span>
                    </div>
                </td>
            `;
        }
    },
    {
        id: 'positions_ratios',
        labelI18n: 'th-group-positions',
        labelDefault: 'Vị Thế Long / Short & Tỷ Lệ',
        thClass: 'px-4 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-emerald-700 dark:text-emerald-400 whitespace-nowrap',
        render: (row) => {
            const isNetPos = row.netPositionUsdt >= 0;
            const netCol = isNetPos ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400';
            const netSign = isNetPos ? '+' : '';
            return `
                <td class="px-4 py-3 whitespace-nowrap text-xs">
                    <div class="flex items-center space-x-1.5 text-xs">
                        <span class="text-emerald-700 dark:text-emerald-400 font-bold">L: ${row.longPositionUsdt !== null ? '$' + formatNumber(row.longPositionUsdt) : '--'}</span>
                        <span class="text-xs text-emerald-800 dark:text-emerald-500 font-bold">(${row.longSpotRatio ? row.longSpotRatio.toFixed(1) + 'x' : '--'})</span>
                        <span class="text-gray-400 dark:text-gray-500">|</span>
                        <span class="text-rose-700 dark:text-rose-400 font-bold">S: ${row.shortPositionUsdt !== null ? '$' + formatNumber(row.shortPositionUsdt) : '--'}</span>
                        <span class="text-xs text-rose-800 dark:text-rose-500 font-bold">(${row.shortSpotRatio ? row.shortSpotRatio.toFixed(1) + 'x' : '--'})</span>
                    </div>
                    <div class="mt-1.5 flex items-center space-x-2 text-xs">
                        <span class="text-gray-600 dark:text-gray-400 font-medium">Tỷ lệ L/S: <span class="text-indigo-700 dark:text-indigo-400 font-bold">${row.longShortRatio ? row.longShortRatio.toFixed(2) : '--'}</span></span>
                        <span class="text-gray-400 dark:text-gray-500">·</span>
                        <span class="text-gray-600 dark:text-gray-400 font-medium">Net: <span class="${netCol} font-bold">${row.netPositionUsdt !== null && row.netPositionUsdt !== undefined ? netSign + '$' + formatNumber(row.netPositionUsdt) : '--'}</span></span>
                    </div>
                </td>
            `;
        }
    },
    {
        id: 'oi_corr',
        labelI18n: 'th-group-oi-corr',
        labelDefault: 'Vị Thế Mở (OI) & Tương Quan',
        thClass: 'px-4 py-3.5 text-left text-xs font-bold uppercase tracking-wider text-fuchsia-700 dark:text-pink-400 whitespace-nowrap',
        render: (row) => {
            const oiChangePos = row.oiChangePct >= 0;
            const oiChangeCol = oiChangePos ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400';
            const oiChangeSign = oiChangePos ? '+' : '';

            let corrCol = 'text-purple-700 dark:text-purple-400';
            if (row.correlation > 0.7) corrCol = 'text-emerald-700 dark:text-emerald-400';
            else if (row.correlation < 0.3) corrCol = 'text-rose-700 dark:text-rose-400';

            return `
                <td class="px-4 py-3 whitespace-nowrap text-xs">
                    <div class="flex items-center space-x-2 text-xs">
                        <span class="font-extrabold text-sm text-fuchsia-700 dark:text-pink-400">${row.openInterestUsdt ? '$' + formatNumber(row.openInterestUsdt) : '--'}</span>
                        <span class="${oiChangeCol} text-xs font-bold">(${oiChangeSign}${row.oiChangePct !== null && row.oiChangePct !== undefined ? row.oiChangePct.toFixed(2) : '0.00'}%)</span>
                    </div>
                    <div class="mt-1.5 text-xs text-gray-700 dark:text-gray-300 flex items-center space-x-2">
                        <span class="font-bold">${row.openInterestCoins ? formatNumber(row.openInterestCoins, 2) + ' Coins' : '--'}</span>
                        <span class="text-gray-400 dark:text-gray-500">·</span>
                        <span class="text-gray-600 dark:text-gray-400 font-medium">Corr: <span class="${corrCol} font-bold">${row.correlation !== null && row.correlation !== undefined ? row.correlation.toFixed(3) : '--'}</span></span>
                    </div>
                </td>
            `;
        }
    }
];

// Quick View Presets
const PRESETS = {
    standard: ['time', 'price', 'spotVol', 'spotBuySell', 'futVol', 'futBuySell', 'longPos', 'shortPos', 'longSpot', 'shortSpot', 'lsRatio', 'fsRatio', 'oi', 'marketState'],
    full: TABLE_COLUMNS.map(c => c.id),
    ohlc: ['time', 'price', 'ohlc', 'spotVol', 'futVol', 'fsRatio', 'marketState'],
    flow: ['time', 'price', 'spotVol', 'spotBuySell', 'spotDelta', 'futVol', 'futBuySell', 'futDelta', 'cvd', 'correlation'],
    positions: ['time', 'price', 'longPos', 'shortPos', 'netPos', 'longSpot', 'shortSpot', 'lsRatio', 'oi', 'oiChange', 'marketState']
};


export { TABLE_COLUMNS, STACKED_COLUMNS, PRESETS };
