/**
 * =============================================================================
 * TẠO BIỂU ĐỒ TRÒN (DOUGHNUT CHART) ĐƯỢC CẢI TIẾN
 * =============================================================================
 * Kiểu dáng được điều khiển bởi 'doughnut-chart.css'.
 *
 * @param {HTMLElement} container - Element DOM để chứa biểu đồ.
 * @param {Array<object>} data - Dữ liệu. Vd: [{ value: 30, label: 'A', color: '#FF6384' }]
 * @param {object|string} config - Đối tượng cấu hình cho biểu đồ hoặc title string (backward compatibility).
 * @param {string} [config.title=''] - Tiêu đề hiển thị ở giữa biểu đồ.
 * @param {number} [config.outerRadius=80] - Bán kính ngoài của biểu đồ.
 * @param {number} [config.innerRadius=50] - Bán kính trong của biểu đồ.
 * @param {boolean} [config.showLegend=true] - Có hiển thị chú thích hay không.
 */

/**
 * Chuyển đổi tọa độ cực sang Descartes cho doughnut chart.
 * @param {number} centerX - Tọa độ X của tâm.
 * @param {number} centerY - Tọa độ Y của tâm.
 * @param {number} radius - Bán kính.
 * @param {number} angleInDegrees - Góc (tính bằng độ).
 * @returns {{x: number, y: number}} Tọa độ Descartes.
 */
function polarToCartesian_doughnut(centerX, centerY, radius, angleInDegrees) {
    const angleInRadians = ((angleInDegrees - 90) * Math.PI) / 180.0;
    return {
        x: centerX + radius * Math.cos(angleInRadians),
        y: centerY + radius * Math.sin(angleInRadians),
    };
}

/**
 * Tạo chuỗi path data 'd' cho một cung của biểu đồ Doughnut.
 * @returns {string}
 */
function describeDoughnutArc_doughnut(x, y, outerRadius, innerRadius, startAngle, endAngle) {
    const startOuter = polarToCartesian_doughnut(x, y, outerRadius, endAngle);
    const endOuter = polarToCartesian_doughnut(x, y, outerRadius, startAngle);
    const startInner = polarToCartesian_doughnut(x, y, innerRadius, endAngle);
    const endInner = polarToCartesian_doughnut(x, y, innerRadius, startAngle);

    const largeArcFlag = endAngle - startAngle <= 180 ? '0' : '1';

    const d = [
        'M', startOuter.x, startOuter.y,
        'A', outerRadius, outerRadius, 0, largeArcFlag, 0, endOuter.x, endOuter.y,
        'L', endInner.x, endInner.y,
        'A', innerRadius, innerRadius, 0, largeArcFlag, 1, startInner.x, startInner.y,
        'Z'
    ].join(' ');

    return d;
}

function createDoughnutChart(container, data, config = {}) {
    // --- BACKWARD COMPATIBILITY ---
    // Nếu tham số thứ 3 là string, tạo config object với title
    if (typeof config === 'string') {
        config = { title: config, showLegend: true };
    }

    // --- 1. KIỂM TRA ĐẦU VÀO ---
    if (!container) {
        console.error("Lỗi: Container element không được cung cấp cho createDoughnutChart.");
        return;
    }

    if (!data || data.length === 0) {
        container.innerHTML = '<div class="doughnut-error">Không có dữ liệu để hiển thị.</div>';
        return;
    }

    const total = data.reduce((sum, d) => sum + d.value, 0);
    if (total === 0) {
        container.innerHTML = '<div class="doughnut-error">Tổng giá trị bằng 0, không thể vẽ biểu đồ.</div>';
        return;
    }

    // --- 2. CẤU HÌNH & HẰNG SỐ ---
    const cfg = {
        title: '',
        outerRadius: 80,
        innerRadius: 50,
        showLegend: true,
        ...config,
    };

    const SVG_SIZE = 180;
    const CENTER_X = SVG_SIZE / 2;
    const CENTER_Y = SVG_SIZE / 2;
    const SVG_NS = "http://www.w3.org/2000/svg";

    // --- 3. TÍNH TOÁN LOGIC ---
    const validData = data.filter(d => d.value > 0);
    
    // --- 4. TẠO CÁC THÀNH PHẦN SVG ---

    // Hàm trợ giúp để tạo phần tử SVG và đặt thuộc tính
    function createSvgElement(tag, attributes) {
        const element = document.createElementNS(SVG_NS, tag);
        for (const key in attributes) {
            element.setAttribute(key, attributes[key]);
        }
        return element;
    }

    // Xóa nội dung cũ và thêm class vào container
    container.innerHTML = '';
    container.classList.add('doughnut-container');

    // Tạo wrapper chính
    const wrapper = document.createElement('div');
    wrapper.className = 'doughnut-chart-wrapper';

    // Tạo phần biểu đồ
    const chartSection = document.createElement('div');
    chartSection.className = 'doughnut-chart-section';

    // Tạo SVG element chính
    const svg = createSvgElement('svg', {
        viewBox: `0 0 ${SVG_SIZE} ${SVG_SIZE}`,
        class: 'doughnut-svg'
    });

    // Tạo group cho các segments
    const segmentsGroup = createSvgElement('g', { class: 'doughnut-segments-group' });

    // a. Tạo các segments
    let currentAngle = 0;
    const segments = [];

    validData.forEach((d, i) => {
        const percentage = d.value / total;
        const angleSpan = percentage * 360;
        const endAngle = currentAngle + angleSpan;
        
        const pathData = describeDoughnutArc_doughnut(CENTER_X, CENTER_Y, cfg.outerRadius, cfg.innerRadius, currentAngle, endAngle);
        
        const segmentPath = createSvgElement('path', {
            d: pathData,
            fill: d.color,
            class: 'doughnut-segment',
            'data-index': i,
            'data-value': d.value,
            'data-percentage': (percentage * 100).toFixed(1),
            'data-label': d.label
        });

        segments.push({
            element: segmentPath,
            data: d,
            percentage: percentage * 100,
            index: i
        });

        segmentsGroup.appendChild(segmentPath);
        currentAngle = endAngle;
    });

    svg.appendChild(segmentsGroup);

    // b. Tạo title ở giữa (nếu có)
    if (cfg.title) {
        const titleText = createSvgElement('text', {
            x: CENTER_X,
            y: CENTER_Y,
            class: 'doughnut-title-text'
        });
        titleText.textContent = cfg.title;
        svg.appendChild(titleText);
    }

    chartSection.appendChild(svg);
    wrapper.appendChild(chartSection);

    // c. Tạo legend (nếu được yêu cầu)
    let legendItems = [];
    if (cfg.showLegend) {
        const legendSection = document.createElement('div');
        legendSection.className = 'doughnut-legend-section';

        validData.forEach((d, i) => {
            const percentage = (d.value / total * 100).toFixed(1);
            
            const legendItem = document.createElement('div');
            legendItem.className = 'doughnut-legend-item';
            legendItem.dataset.index = i;

            const colorBox = document.createElement('span');
            colorBox.className = 'doughnut-legend-color';
            colorBox.style.backgroundColor = d.color;

            const labelSpan = document.createElement('span');
            labelSpan.className = 'doughnut-legend-label';
            labelSpan.textContent = `${d.label}: ${percentage}%`;

            legendItem.appendChild(colorBox);
            legendItem.appendChild(labelSpan);
            legendSection.appendChild(legendItem);

            legendItems.push(legendItem);
        });

        wrapper.appendChild(legendSection);
    }

    // --- 5. THIẾT LẬP HIỆU ỨNG HOVER ---
    function handleMouseEnter(event) {
        const target = event.currentTarget;
        const index = parseInt(target.dataset.index);
        
        // Thêm class highlight cho container
        wrapper.classList.add('is-highlighted');
        
        // Highlight segment tương ứng
        segments.forEach((seg, i) => {
            if (i === index) {
                seg.element.classList.add('highlight');
            } else {
                seg.element.classList.add('dimmed');
            }
        });

        // Highlight legend item tương ứng
        if (cfg.showLegend) {
            legendItems.forEach((item, i) => {
                if (i === index) {
                    item.classList.add('highlight');
                } else {
                    item.classList.add('dimmed');
                }
            });
        }
    }

    function handleMouseLeave(event) {
        // Xóa tất cả highlight và dimmed classes
        wrapper.classList.remove('is-highlighted');
        segments.forEach(seg => {
            seg.element.classList.remove('highlight', 'dimmed');
        });
        
        if (cfg.showLegend) {
            legendItems.forEach(item => {
                item.classList.remove('highlight', 'dimmed');
            });
        }
    }

    // Thêm event listeners cho segments
    segments.forEach(seg => {
        seg.element.addEventListener('mouseenter', handleMouseEnter);
        seg.element.addEventListener('mouseleave', handleMouseLeave);
    });

    // Thêm event listeners cho legend items
    if (cfg.showLegend) {
        legendItems.forEach(item => {
            item.addEventListener('mouseenter', handleMouseEnter);
            item.addEventListener('mouseleave', handleMouseLeave);
        });
    }

    // --- 6. RENDER BIỂU ĐỒ ---
    container.appendChild(wrapper);
}