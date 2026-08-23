/**
 * ChartRenderer - Renders gauge and dominance charts
 * 
 * Responsibilities:
 * - Render gauge charts for Fear & Greed Index and RSI
 * - Render dominance charts for BTC and ETH
 */

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class ChartRenderer {
    constructor() {
        this.GAUGE_START_ANGLE = -120;
        this.GAUGE_END_ANGLE = 120;
    }
    
    /**
     * Convert polar coordinates to Cartesian
     */
    polarToCartesian(centerX, centerY, radius, angleInDegrees) {
        const angleInRadians = ((angleInDegrees - 90) * Math.PI) / 180.0;
        return {
            x: centerX + radius * Math.cos(angleInRadians),
            y: centerY + radius * Math.sin(angleInRadians)
        };
    }
    
    /**
     * Create SVG arc path data
     */
    describeArc(x, y, radius, startAngle, endAngle) {
        const startPoint = this.polarToCartesian(x, y, radius, startAngle);
        const endPoint = this.polarToCartesian(x, y, radius, endAngle);
        const largeArcFlag = endAngle - startAngle <= 180 ? '0' : '1';
        
        return [
            'M', startPoint.x, startPoint.y,
            'A', radius, radius, 0, largeArcFlag, '1', endPoint.x, endPoint.y
        ].join(' ');
    }
    
    /**
     * Get color stops for gauge gradient
     */
    getColorStops(type) {
        if (type === 'fear-greed') {
            return [
                { value: 0, color: '#ef4444' },    // Red (Extreme Fear)
                { value: 25, color: '#f97316' },   // Orange
                { value: 40, color: '#fb923c' },   // Light Orange
                { value: 50, color: '#fbbf24' },   // Yellow (Neutral)
                { value: 60, color: '#a3e635' },   // Yellow-Green
                { value: 75, color: '#84cc16' },   // Lime
                { value: 100, color: '#22c55e' }   // Green (Extreme Greed)
            ];
        } else {
            // RSI
            return [
                { value: 0, color: '#22c55e' },    // Green (Oversold)
                { value: 25, color: '#84cc16' },   // Lime
                { value: 40, color: '#a3e635' },   // Yellow-Green
                { value: 50, color: '#fbbf24' },   // Yellow (Neutral)
                { value: 60, color: '#fb923c' },   // Light Orange
                { value: 75, color: '#f97316' },   // Orange
                { value: 100, color: '#ef4444' }   // Red (Overbought)
            ];
        }
    }
    
    /**
     * Render gauge chart for Fear & Greed Index or RSI
     * @param {string} type - 'fear-greed' or 'btc-rsi'
     * @param {number} value - Value (0-100)
     */
    renderGaugeChart(type, value) {
        const svgId = type === 'fear-greed' ? 'fear-greed-gauge-svg' : 'btc-rsi-gauge-svg';
        const svg = document.getElementById(svgId);
        if (!svg) {
            debugLog(`⚠️ Gauge SVG not found for ${type}`);
            return;
        }
        
        const ANGLE_SPAN = this.GAUGE_END_ANGLE - this.GAUGE_START_ANGLE;
        const centerX = 60;
        const centerY = 60;
        const radius = 45;
        const strokeWidth = 10;
        
        const clampedValue = Math.max(0, Math.min(100, value));
        const percentage = clampedValue / 100;
        const valueAngle = this.GAUGE_START_ANGLE + (percentage * ANGLE_SPAN);
        
        const colorStops = this.getColorStops(type);
        
        // Clear SVG
        svg.innerHTML = '';
        
        // Create gradient
        const gradientId = `gauge-gradient-${type}`;
        const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
        const gradient = document.createElementNS('http://www.w3.org/2000/svg', 'linearGradient');
        gradient.setAttribute('id', gradientId);
        gradient.setAttribute('x1', '0%');
        gradient.setAttribute('y1', '0%');
        gradient.setAttribute('x2', '100%');
        gradient.setAttribute('y2', '0%');
        
        colorStops.forEach(stop => {
            const stopElement = document.createElementNS('http://www.w3.org/2000/svg', 'stop');
            stopElement.setAttribute('offset', `${stop.value}%`);
            stopElement.setAttribute('stop-color', stop.color);
            gradient.appendChild(stopElement);
        });
        
        defs.appendChild(gradient);
        svg.appendChild(defs);
        
        // Background arc
        const backgroundPath = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        backgroundPath.setAttribute('d', this.describeArc(centerX, centerY, radius, this.GAUGE_START_ANGLE, this.GAUGE_END_ANGLE));
        backgroundPath.setAttribute('stroke', '#e5e7eb');
        backgroundPath.setAttribute('stroke-width', strokeWidth);
        backgroundPath.setAttribute('fill', 'none');
        svg.appendChild(backgroundPath);
        
        // Value arc
        if (clampedValue > 0) {
            const valuePath = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            valuePath.setAttribute('d', this.describeArc(centerX, centerY, radius, this.GAUGE_START_ANGLE, valueAngle));
            valuePath.setAttribute('stroke', `url(#${gradientId})`);
            valuePath.setAttribute('stroke-width', strokeWidth);
            valuePath.setAttribute('stroke-linecap', 'round');
            valuePath.setAttribute('fill', 'none');
            svg.appendChild(valuePath);
        }
        
        // Needle
        const needlePoint = this.polarToCartesian(centerX, centerY, radius - 5, valueAngle);
        const needleLine = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        needleLine.setAttribute('x1', centerX);
        needleLine.setAttribute('y1', centerY);
        needleLine.setAttribute('x2', needlePoint.x);
        needleLine.setAttribute('y2', needlePoint.y);
        needleLine.setAttribute('stroke', '#374151');
        needleLine.setAttribute('stroke-width', '2');
        svg.appendChild(needleLine);
        
        // Center dot
        const centerDot = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        centerDot.setAttribute('cx', centerX);
        centerDot.setAttribute('cy', centerY);
        centerDot.setAttribute('r', '4');
        centerDot.setAttribute('fill', '#374151');
        svg.appendChild(centerDot);
        
        debugLog(`✅ Rendered ${type} gauge chart: ${clampedValue}`);
    }
    
    /**
     * Render dominance pie chart
     * @param {string} type - 'btc' or 'eth'
     * @param {number} value - Dominance percentage
     */
    renderDominanceChart(type, value) {
        const svgId = `${type}-dominance-svg`;
        const svg = document.getElementById(svgId);
        if (!svg) {
            debugLog(`⚠️ Dominance SVG not found for ${type}`);
            return;
        }
        
        const width = 120;
        const height = 80;
        const centerX = width / 2;
        const centerY = height / 2;
        const radius = 28;
        
        const color = type === 'btc' ? '#f7931a' : '#627eea';
        const othersColor = 'rgba(128, 128, 128, 0.3)';
        
        // Clear SVG
        svg.innerHTML = '';
        svg.setAttribute('viewBox', `0 0 ${width} ${height}`);
        svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
        
        // Background circle (Others)
        const othersCircle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        othersCircle.setAttribute('cx', centerX);
        othersCircle.setAttribute('cy', centerY);
        othersCircle.setAttribute('r', radius);
        othersCircle.setAttribute('fill', othersColor);
        svg.appendChild(othersCircle);
        
        const clampedValue = Math.max(0, Math.min(100, value));
        
        // Dominance slice
        if (clampedValue >= 0.5 && clampedValue < 100) {
            const angle = (clampedValue / 100) * 2 * Math.PI;
            const startAngle = -Math.PI / 2; // Start from top
            const endAngle = startAngle + angle;
            
            const x1 = centerX + radius * Math.cos(startAngle);
            const y1 = centerY + radius * Math.sin(startAngle);
            const x2 = centerX + radius * Math.cos(endAngle);
            const y2 = centerY + radius * Math.sin(endAngle);
            
            const largeArc = angle > Math.PI ? 1 : 0;
            
            const pathData = [
                `M ${centerX} ${centerY}`,
                `L ${x1} ${y1}`,
                `A ${radius} ${radius} 0 ${largeArc} 1 ${x2} ${y2}`,
                'Z'
            ].join(' ');
            
            const slice = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            slice.setAttribute('d', pathData);
            slice.setAttribute('fill', color);
            slice.setAttribute('opacity', '0.8');
            svg.appendChild(slice);
        } else if (clampedValue >= 100) {
            // Full circle
            const fullCircle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            fullCircle.setAttribute('cx', centerX);
            fullCircle.setAttribute('cy', centerY);
            fullCircle.setAttribute('r', radius);
            fullCircle.setAttribute('fill', color);
            fullCircle.setAttribute('opacity', '0.8');
            svg.appendChild(fullCircle);
        }
        
        debugLog(`📊 Rendered ${type.toUpperCase()} dominance chart: ${clampedValue.toFixed(1)}%`);
    }
}
