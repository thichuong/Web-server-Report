// static/chart_modules/bar.js

/**
 * TẠO BIỂU ĐỒ CỘT (BAR CHART) HỖ TRỢ GIÁ TRỊ ÂM
 * @param {HTMLElement} container - Element DOM.
 * @param {Array<object>} data - Dữ liệu, vd: [{value: 10, label: 'A', color: 'red'}].
 * @param {object} [options] - Các tùy chọn, vd: { valuePrefix: '$', valueSuffix: 'M', yAxisLabel: 'Triệu USD' }.
 */
function createBarChart(container, data, options = {}) {
    if (!container || !data || data.length === 0) return;

    const { valuePrefix = '', valueSuffix = '', yAxisLabel = '' } = options;

    // Lấy kích thước động từ container để biểu đồ co giãn theo card
    const availableWidth = container.clientWidth || 450; // Lấy chiều rộng thực tế, có fallback
    const width = availableWidth;
    const height = availableWidth * 0.5; // Giữ tỷ lệ khung hình ~2:1, có thể điều chỉnh

    const pTop = 25, pBottom = 20;
    const pLeft = yAxisLabel ? 45 : 20;
    const pRight = 20;

    const allValues = data.map(d => d.value);
    const minValue = Math.min(0, ...allValues);
    const maxValue = Math.max(0, ...allValues);
    const totalRange = maxValue - minValue;

    if (totalRange === 0) return;

    const chartAreaHeight = height - pTop - pBottom;
    const chartWidth = width - pLeft - pRight;
    const zeroY = pTop + (maxValue / totalRange) * chartAreaHeight;
    const barWidth = chartWidth / data.length * 0.65;
    const gap = chartWidth / data.length * 0.35;
    const scale = chartAreaHeight / totalRange;

    let bars = '';
    let legendItems = '';
    let yAxisUnit = '';

    if (yAxisLabel) {
        yAxisUnit = `
            <text x="${-(pTop + chartAreaHeight / 2)}" y="15"
                  transform="rotate(-90)" text-anchor="middle" font-size="12px"
                  font-weight="500" fill="var(--text-secondary)">
                ${yAxisLabel}
            </text>
        `;
    }

    data.forEach((d, i) => {
        const barHeight = Math.abs(d.value) * scale;
        const x = pLeft + i * (barWidth + gap) + gap / 2;
        let y, labelY;

        if (d.value >= 0) {
            y = zeroY - barHeight;
            labelY = -8;
        } else {
            y = zeroY;
            labelY = barHeight + 15;
        }

        bars += `
            <g transform="translate(${x}, ${y})" class="bar-group" data-index="${i}">
                <rect width="${barWidth}" height="${barHeight}"
                      fill="${d.color || 'var(--accent-color)'}" rx="3" class="bar-rect"
                      style="animation-delay: ${i * 100}ms;" />
            <text x="${barWidth / 2}" y="${labelY}" text-anchor="middle" font-weight="600"
                fill="var(--text-primary)" class="bar-value-label h3">${valuePrefix}${d.value}${valueSuffix}</text>
            </g>
        `;

        legendItems += `
            <span class="legend-item">
                <span class="legend-color-box" style="background-color: ${d.color || 'var(--accent-color)'};"></span>
                <span>${d.label}</span>
            </span>
        `;
    });

    container.innerHTML = `
        <div class="bar-chart-layout">
            <div class="bar-chart-svg-wrapper">
                <svg viewBox="0 0 ${width} ${height}" style="width:100%; height:auto; overflow: visible;">
                    ${yAxisUnit}
                    <line x1="${pLeft}" y1="${zeroY}" x2="${width - pRight}" y2="${zeroY}" stroke="var(--border-color)" />
                    ${bars}
                    <!-- Custom Tooltip -->
                    <g class="bar-chart-tooltip" style="visibility: hidden;">
                        <rect class="tooltip-bg" rx="4" ry="4" height="28" />
                        <text class="tooltip-text" x="10" y="18"></text>
                    </g>
                </svg>
            </div>
            <div class="bar-chart-legend doughnut-legend">
                ${legendItems}
            </div>
        </div>
    `;
    container.classList.add('bar-chart-container');

    // --- THÊM TƯƠNG TÁC CHO TOOLTIP ---
    const svg = container.querySelector('svg');
    const tooltip = svg.querySelector('.bar-chart-tooltip');
    const tooltipBg = tooltip.querySelector('.tooltip-bg');
    const tooltipText = tooltip.querySelector('.tooltip-text');
    const barGroups = svg.querySelectorAll('.bar-group');
    const svgPoint = svg.createSVGPoint();

    function getMousePosition(event) {
        svgPoint.x = event.clientX;
        svgPoint.y = event.clientY;
        // Chuyển đổi tọa độ chuột sang hệ tọa độ của SVG
        return svgPoint.matrixTransform(svg.getScreenCTM().inverse());
    }

    barGroups.forEach(bar => {
        bar.addEventListener('mouseover', (e) => {
            const index = parseInt(bar.dataset.index);
            const d = data[index];
            const content = `${d.label}: ${valuePrefix}${d.value}${valueSuffix}`;
            
            tooltipText.textContent = content;
            tooltip.style.visibility = 'visible';

            // Tự động điều chỉnh chiều rộng tooltip
            const textBBox = tooltipText.getBBox();
            tooltipBg.setAttribute('width', textBBox.width + 20); // 10px padding mỗi bên
        });

        bar.addEventListener('mousemove', (e) => {
            const pos = getMousePosition(e);
            tooltip.setAttribute('transform', `translate(${pos.x + 12}, ${pos.y - 30})`);
        });

        bar.addEventListener('mouseout', () => {
            tooltip.style.visibility = 'hidden';
        });
    });
}