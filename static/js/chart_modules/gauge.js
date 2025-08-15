/**
 * =============================================================================
 * TẠO BIỂU ĐỒ ĐỒNG HỒ (GAUGE) ĐƯỢC CẢI TIẾN
 * =============================================================================
 * Kiểu dáng được điều khiển bởi 'gauge-chart.css'.
 *
 * @param {HTMLElement} container - Element DOM để chứa biểu đồ.
 * @param {number} value - Giá trị hiện tại để hiển thị trên biểu đồ.
 * @param {object} config - Đối tượng cấu hình cho biểu đồ.
 * @param {number} [config.min=0] - Giá trị tối thiểu của gauge.
 * @param {number} [config.max=100] - Giá trị tối đa của gauge.
 * @param {Array<object>} [config.segments=[]] - Mảng các đoạn màu.
 */

/**
 * Chuyển đổi tọa độ cực sang Descartes cho gauge chart.
 * @param {number} centerX - Tọa độ X của tâm.
 * @param {number} centerY - Tọa độ Y của tâm.
 * @param {number} radius - Bán kính.
 * @param {number} angleInDegrees - Góc (tính bằng độ).
 * @returns {{x: number, y: number}} Tọa độ Descartes.
 */
function polarToCartesian_gauge(centerX, centerY, radius, angleInDegrees) {
    const angleInRadians = ((angleInDegrees - 90) * Math.PI) / 180.0;
    return {
        x: centerX + radius * Math.cos(angleInRadians),
        y: centerY + radius * Math.sin(angleInRadians),
    };
}

/**
 * Tạo chuỗi path data 'd' cho một cung tròn SVG trong gauge chart.
 * @param {number} x - Tọa độ X của tâm.
 * @param {number} y - Tọa độ Y của tâm.
 * @param {number} radius - Bán kính của cung tròn.
 * @param {number} startAngle - Góc bắt đầu (độ).
 * @param {number} endAngle - Góc kết thúc (độ).
 * @returns {string} Chuỗi data cho thuộc tính 'd' của thẻ <path>.
 */
function describeArc_gauge(x, y, radius, startAngle, endAngle) {
    const startPoint = polarToCartesian_gauge(x, y, radius, startAngle);
    const endPoint = polarToCartesian_gauge(x, y, radius, endAngle);

    const largeArcFlag = endAngle - startAngle <= 180 ? '0' : '1';
    const sweepFlag = '1'; // Vẽ cung theo chiều dương (cùng chiều kim đồng hồ)

    const d = [
        'M', startPoint.x, startPoint.y,
        'A', radius, radius, 0, largeArcFlag, sweepFlag, endPoint.x, endPoint.y
    ].join(' ');

    return d;
}

function createGauge(container, value, config) {
    // --- 1. KIỂM TRA ĐẦU VÀO ---
    if (!container) {
        console.error("Lỗi: Container element không được cung cấp cho createGauge.");
        return;
    }

    // --- 2. ĐẢM BẢO CONTAINER CÓ THUỘC TÍNH CĂN GIỮA ---
    // Thêm styles inline để đảm bảo căn giữa hoàn hảo
    container.style.display = 'flex';
    container.style.justifyContent = 'center';
    container.style.alignItems = 'center';
    container.style.margin = '0 auto';
    container.style.textAlign = 'center';

    // --- 3. CẤU HÌNH & HẰNG SỐ ---
    const cfg = {
        min: 0,
        max: 100,
        segments: [],
        ...config,
    };
    const GAUGE_START_ANGLE = -120;
    const GAUGE_END_ANGLE = 120;
    const ANGLE_SPAN = GAUGE_END_ANGLE - GAUGE_START_ANGLE;
    const SVG_NS = "http://www.w3.org/2000/svg"; // Namespace cho SVG

    // --- 4. TÍNH TOÁN LOGIC ---
    const clampedValue = Math.max(cfg.min, Math.min(cfg.max, value));
    const percentage = (clampedValue - cfg.min) / (cfg.max - cfg.min);
    const valueAngle = GAUGE_START_ANGLE + (percentage * ANGLE_SPAN);

    const currentSegment = cfg.segments.find(s => clampedValue <= s.limit) ||
                           cfg.segments[cfg.segments.length - 1] ||
                           { color: '#ccc', label: '' };
    const valueColor = currentSegment.color;
    const classification = currentSegment.label || '';

    // --- 5. TẠO CÁC THÀNH PHẦN SVG ---

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
    container.classList.add('gauge-container');

    // Tính toán viewBox với padding để đảm bảo căn giữa hoàn hảo
    const SVG_WIDTH = 300; // Tăng từ 200 lên 300
    const SVG_HEIGHT = 210; // Tăng từ 140 lên 210
    const PADDING = 15; // Tăng padding từ 10 lên 15
    
    // Tạo SVG element chính với viewBox tối ưu cho responsive và căn giữa
    const svg = createSvgElement('svg', {
        viewBox: `${-PADDING} ${-PADDING} ${SVG_WIDTH + 2*PADDING} ${SVG_HEIGHT + 2*PADDING}`,
        preserveAspectRatio: "xMidYMid meet",
        class: 'gauge-svg'
    });

    // a. Tạo đường track nền
    const trackPath = createSvgElement('path', {
        d: describeArc_gauge(150, 150, 110, GAUGE_START_ANGLE, GAUGE_END_ANGLE), // Tăng center từ (100,100) lên (150,150) và radius từ 75 lên 110
        class: 'gauge-track'
    });
    svg.appendChild(trackPath);

    // b. Tạo các đoạn màu
    const segmentsGroup = createSvgElement('g', { class: 'gauge-segments-group' });
    let lastPercentage = 0;
    cfg.segments.forEach(segment => {
        const segmentEndPercentage = (segment.limit - cfg.min) / (cfg.max - cfg.min);
        const start = GAUGE_START_ANGLE + (lastPercentage * ANGLE_SPAN);
        const end = GAUGE_START_ANGLE + (Math.min(segmentEndPercentage, 1) * ANGLE_SPAN);
        
        const segmentPath = createSvgElement('path', {
            d: describeArc_gauge(150, 150, 110, start, end), // Cập nhật center và radius
            stroke: segment.color, // Color được đặt trực tiếp vì nó là dữ liệu động
            class: 'gauge-segment'
        });
        segmentsGroup.appendChild(segmentPath);
        lastPercentage = segmentEndPercentage;
    });
    svg.appendChild(segmentsGroup);

    // c. Tạo kim chỉ
    const needleGroup = createSvgElement('g', {
        class: 'gauge-needle-group',
        transform: `rotate(${valueAngle} 150 150)` // Cập nhật center rotation
    });
    const needlePointer = createSvgElement('path', {
        d: 'M 150 30 L 145 150 L 155 150 Z', // Cập nhật needle path cho kích thước lớn hơn
        class: 'gauge-needle-pointer'
    });
    const needlePivot = createSvgElement('circle', {
        cx: 150, // Cập nhật center X
        cy: 150, // Cập nhật center Y
        r: 8, // Tăng radius từ 6 lên 8
        class: 'gauge-needle-pivot'
    });
    needleGroup.append(needlePointer, needlePivot);
    svg.appendChild(needleGroup);

    // d. Tạo các nhãn văn bản với vị trí căn giữa tuyệt đối
    const centerX = 150; // Cập nhật tâm X của gauge
    const centerY = 150; // Cập nhật tâm Y của gauge
    
    // Tạo group để chứa text và đảm bảo căn giữa
    const textGroup = createSvgElement('g', {
        class: 'gauge-text-group'
    });
    
    const valueText = createSvgElement('text', {
        x: centerX,
        y: centerY + 30, // Điều chỉnh vị trí text theo tỷ lệ mới
        class: 'gauge-text gauge-value-text',
        'text-anchor': 'middle',
        'dominant-baseline': 'central'
    });
    valueText.textContent = value.toFixed(1);

    const labelText = createSvgElement('text', {
        x: centerX,
        y: centerY + 70, // Điều chỉnh vị trí label theo tỷ lệ mới
        fill: valueColor, // Color được đặt trực tiếp vì nó là dữ liệu động
        class: 'gauge-text gauge-label-text',
        'text-anchor': 'middle',
        'dominant-baseline': 'central'
    });
    labelText.textContent = classification;

    textGroup.append(valueText, labelText);
    svg.appendChild(textGroup);

    // --- 6. RENDER BIỂU ĐỒ ---
    container.appendChild(svg);
    
    // --- 7. ĐẢM BẢO CĂN GIỮA SAU KHI RENDER ---
    // Force center alignment sau khi append
    setTimeout(() => {
        if (container.offsetWidth > 0) {
            container.style.textAlign = 'center';
        }
    }, 0);
}