// static/chart_modules/line.js

/**
 * TẠO BIỂU ĐỒ ĐƯỜNG VỚI CÁC TÙY CHỌN NÂNG CAO
 * Hàm này sẽ vẽ một biểu đồ đường SVG bên trong một element container được chỉ định.
 *
 * @param {HTMLElement} container - Element DOM để chứa biểu đồ (ví dụ: document.getElementById('myChart')).
 * @param {Array<number>} data - Mảng các giá trị số để vẽ biểu đồ.
 * @param {object} [options={}] - Một đối tượng chứa các tùy chọn cấu hình cho biểu đồ.
 * @param {string} [options.color='var(--accent-color)'] - Màu chính của đường, vùng tô và các điểm. Có thể dùng bất kỳ biến màu nào từ colors.css.
 * @param {string} [options.valuePrefix=''] - Tiền tố để thêm vào trước nhãn giá trị (ví dụ: '$').
 * @param {string} [options.valueSuffix=''] - Hậu tố để thêm vào sau nhãn giá trị (ví dụ: '%').
 * @param {number} [options.strokeWidth=2.5] - Độ dày của đường biểu đồ.
 */
function createLineChart(container, data, options = {}) {
    // Kiểm tra đầu vào để đảm bảo hàm hoạt động chính xác
    if (!container || !Array.isArray(data) || data.length === 0) {
        console.error("Container hoặc dữ liệu không hợp lệ để vẽ biểu đồ.");
        return;
    }

    // Thiết lập các giá trị mặc định cho options
    const {
        color = 'var(--accent-color)',
        valuePrefix = '',
        valueSuffix = '',
        strokeWidth = 2.5
    } = options;

    // Kích thước và padding của SVG (tăng kích thước)
    const width = 480;
    const height = 340;
    const p = 60; // Padding tăng để tránh nhãn vẽ ra ngoài

    // Tính toán các giá trị cần thiết để xác định tỷ lệ
    const maxValue = Math.max(...data);
    const minValue = Math.min(...data);
    // Thêm một chút khoảng đệm (10% trên và 10% dưới) để biểu đồ không bị cắt
    const valueRange = (maxValue - minValue) * 1.2 || 1;
    const rangeMin = minValue - (maxValue - minValue) * 0.1;

    // Hàm chuyển đổi tọa độ: từ chỉ số mảng và giá trị dữ liệu sang tọa độ (x, y) trên SVG
    const toX = i => (i / (data.length - 1)) * (width - 2 * p) + p;
    const toY = val => height - p - ((val - rangeMin) / valueRange) * (height - 2 * p);

    // Tạo chuỗi tọa độ cho đường và vùng tô
    const points = data.map((d, i) => `${toX(i)},${toY(d)}`).join(' ');
    const areaPoints = `${p},${height - p} ${points} ${width - p},${height - p}`;

    // Xác định các chỉ số cần hiển thị: đầu, cuối, cao nhất, thấp nhất
    const firstIdx = 0;
    const lastIdx = data.length - 1;
    const maxIdx = data.indexOf(maxValue);
    const minIdx = data.indexOf(minValue);
    // Tạo mảng các chỉ số cần hiển thị, loại bỏ trùng lặp
    const showIndices = Array.from(new Set([firstIdx, lastIdx, maxIdx, minIdx]));

    // Tạo mã SVG động
    const svg = `
        <svg viewBox="0 0 ${width} ${height}" style="width: 100%; height: auto; overflow: visible;" class="line-chart-interactive">
            <defs>
                <linearGradient id="areaGradient-${container.id}" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0%" stop-color="${color}" stop-opacity="0.25" />
                    <stop offset="100%" stop-color="${color}" stop-opacity="0" />
                </linearGradient>
            </defs>

            <polygon fill="url(#areaGradient-${container.id})" points="${areaPoints}" class="line-area" />

            <polyline fill="none" stroke="${color}" stroke-width="${strokeWidth}" stroke-linecap="round"
                      stroke-linejoin="round" points="${points}" class="line-path" />

            ${data.map((d, i) => {
                let label = '';
                if (i === firstIdx) {
                    // Nhãn đầu ở bên trái
                    label = `<text x="${toX(i) - 12}" y="${toY(d) + 3}" text-anchor="end"
                        fill="var(--text-primary)" font-weight="600" class="value-label h3">
                        ${valuePrefix}${d.toFixed(1)}${valueSuffix}
                    </text>`;
                } else if (i === lastIdx) {
                    // Nhãn cuối ở bên phải
                    label = `<text x="${toX(i) + 12}" y="${toY(d) + 3}" text-anchor="start"
                        fill="var(--text-primary)" font-weight="600" class="value-label h3">
                        ${valuePrefix}${d.toFixed(1)}${valueSuffix}
                    </text>`;
                } else if (i === maxIdx) {
                    // Nhãn max ở phía trên
                    label = `<text x="${toX(i)}" y="${toY(d) - 12}" text-anchor="middle"
                        fill="var(--text-primary)" font-weight="600" class="value-label h3">
                        ${valuePrefix}${d.toFixed(1)}${valueSuffix}
                    </text>`;
                } else if (i === minIdx) {
                    // Nhãn min ở phía dưới
                    label = `<text x="${toX(i)}" y="${toY(d) + 18}" text-anchor="middle"
                        fill="var(--text-primary)" font-weight="600" class="value-label h3">
                        ${valuePrefix}${d.toFixed(1)}${valueSuffix}
                    </text>`;
                }
                return `
                    <g class="line-point-group">
                        <circle cx="${toX(i)}" cy="${toY(d)}" r="9" fill="transparent" />
                        <circle cx="${toX(i)}" cy="${toY(d)}" r="4" fill="${color}"
                                stroke="var(--bg-secondary)" stroke-width="2" class="line-dot"
                                style="animation-delay: ${i * 80}ms"/>
                        ${showIndices.includes(i) ? label : ''}
                    </g>
                `;
            }).join('')}
        </svg>
    `;

    // Gán SVG vào container và thêm class
    container.innerHTML = svg;
    container.classList.add('line-chart-container');

    // Tạo tooltip
    let tooltip = container.querySelector('.line-chart-tooltip');
    if (!tooltip) {
        tooltip = document.createElement('div');
        tooltip.className = 'line-chart-tooltip';
        container.appendChild(tooltip);
    }

    // Xử lý tooltip cho các điểm và giữa các điểm (interpolated values)
    const svgEl = container.querySelector('svg');
    const pointGroups = svgEl.querySelectorAll('.line-point-group');

    // Create SVG hover indicator: vertical line + focus dot
    (function createHoverIndicator() {
        const svgns = 'http://www.w3.org/2000/svg';
        const hoverGroup = document.createElementNS(svgns, 'g');
        hoverGroup.setAttribute('class', 'hover-indicator');

        const hoverLine = document.createElementNS(svgns, 'line');
        hoverLine.setAttribute('x1', 0);
        hoverLine.setAttribute('x2', 0);
        hoverLine.setAttribute('y1', p);
        hoverLine.setAttribute('y2', height - p);
        hoverLine.setAttribute('stroke', color);
        hoverLine.setAttribute('stroke-width', 1);
        hoverLine.setAttribute('stroke-dasharray', '4 3');
        hoverLine.setAttribute('visibility', 'hidden');

        const hoverDot = document.createElementNS(svgns, 'circle');
        hoverDot.setAttribute('cx', 0);
        hoverDot.setAttribute('cy', 0);
        hoverDot.setAttribute('r', 5);
        hoverDot.setAttribute('fill', color);
        hoverDot.setAttribute('stroke', 'var(--bg-secondary)');
        hoverDot.setAttribute('stroke-width', 2);
        hoverDot.setAttribute('visibility', 'hidden');

        hoverGroup.appendChild(hoverLine);
        hoverGroup.appendChild(hoverDot);
        svgEl.appendChild(hoverGroup);

        // Expose references for later updates
        svgEl.__hoverLine = hoverLine;
        svgEl.__hoverDot = hoverDot;
    })();

    // Helper: show tooltip at client coordinates with formatted value
    function showTooltipAt(clientX, clientY, value) {
        tooltip.textContent = `${valuePrefix}${value.toFixed(2)}${valueSuffix}`;
        tooltip.classList.add('active');
        const rect = container.getBoundingClientRect();
        tooltip.style.left = `${clientX - rect.left + 12}px`;
        tooltip.style.top = `${clientY - rect.top - 32}px`;
    }

    function hideTooltip() {
        tooltip.classList.remove('active');
    }

    // Point-specific handlers still trigger exact values but reuse helper
    pointGroups.forEach((group, i) => {
        const dot = group.querySelector('.line-dot');
        dot.addEventListener('mouseenter', (e) => {
            const value = data[i];
            showTooltipAt(e.clientX, e.clientY, value);
        });
        dot.addEventListener('mousemove', (e) => {
            // Keep tooltip following the cursor when on a point
            const value = data[i];
            showTooltipAt(e.clientX, e.clientY, value);
        });
        dot.addEventListener('mouseleave', () => {
            // Don't immediately hide: let svg mousemove take over if still inside svg.
            // Small timeout prevents flicker when moving between point and nearby area.
            setTimeout(() => {
                // If tooltip was re-activated by svg mousemove, don't hide it.
                if (!tooltip.classList.contains('active')) return;
                // If mouse is outside svg, hide
                // Use document.elementFromPoint to check current element under cursor
                // If it's not inside our container, hide tooltip.
                const rect = container.getBoundingClientRect();
                const cx = rect.left + rect.width / 2; // fallback
                const el = document.elementFromPoint(cx, rect.top + rect.height / 2);
                if (!container.contains(el)) hideTooltip();
            }, 10);
        });
    });

    // SVG-level mousemove to compute interpolated value between points
    svgEl.addEventListener('mousemove', (e) => {
        // Compute mouse X relative to the displayed SVG width, then map to viewBox width
        const rect = svgEl.getBoundingClientRect();
        if (rect.width === 0) return;
        // Normalize to SVG viewBox coordinates (the viewBox width is the `width` constant)
        const mouseX = (e.clientX - rect.left) * (width / rect.width);

        // If mouse is outside the start/end area (padding), hide tooltip/indicator and exit
        if (mouseX < p || mouseX > (width - p)) {
            hideTooltip();
            const hoverLine = svgEl.__hoverLine;
            const hoverDot = svgEl.__hoverDot;
            if (hoverLine) hoverLine.setAttribute('visibility', 'hidden');
            if (hoverDot) hoverDot.setAttribute('visibility', 'hidden');
            return;
        }

        // Map normalized X to fractional data index
        const usableWidth = width - 2 * p;
        const idxFloat = ((mouseX - p) / usableWidth) * (data.length - 1);
        const clamped = Math.max(0, Math.min(data.length - 1, idxFloat));
        const prevIdx = Math.floor(clamped);
        const nextIdx = Math.min(data.length - 1, Math.ceil(clamped));

        let value;
        if (prevIdx === nextIdx) {
            value = data[prevIdx];
        } else {
            const t = clamped - prevIdx;
            value = data[prevIdx] + t * (data[nextIdx] - data[prevIdx]);
        }

        showTooltipAt(e.clientX, e.clientY, value);

        // Update hover indicator (use viewBox coordinates: mouseX is in viewBox space)
        const hoverLine = svgEl.__hoverLine;
        const hoverDot = svgEl.__hoverDot;
        if (hoverLine && hoverDot) {
            // Set vertical line x position and make visible
            hoverLine.setAttribute('x1', mouseX);
            hoverLine.setAttribute('x2', mouseX);
            hoverLine.setAttribute('visibility', 'visible');

            // Set dot at interpolated point on line
            const dotY = toY(value);
            hoverDot.setAttribute('cx', mouseX);
            hoverDot.setAttribute('cy', dotY);
            hoverDot.setAttribute('visibility', 'visible');
        }
    });

    svgEl.addEventListener('mouseleave', () => {
        hideTooltip();
        const hoverLine = svgEl.__hoverLine;
        const hoverDot = svgEl.__hoverDot;
        if (hoverLine) hoverLine.setAttribute('visibility', 'hidden');
        if (hoverDot) hoverDot.setAttribute('visibility', 'hidden');
    });
}