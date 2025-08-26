# Market Indicators Dashboard Component

Đây là một shared component hiển thị các chỉ số thị trường tiền mã hóa theo thời gian thực mà không vẽ chart.

## Tính năng

- **Hiển thị các chỉ số chính:**
  - Giá Bitcoin (BTC)
  - Tổng vốn hóa thị trường 
  - Khối lượng giao dịch 24h
  - Chỉ số Fear & Greed
  - Độ thống trị Bitcoin
  - Số lượng coin hoạt động
  - Số lượng sàn giao dịch

- **Cập nhật thời gian thực:**
  - Kết nối WebSocket để nhận dữ liệu live
  - Animation khi cập nhật dữ liệu
  - Hiển thị trạng thái kết nối
  - Tự động reconnect khi mất kết nối

- **Responsive Design:**
  - Tương thích với mobile và desktop
  - Grid layout linh hoạt
  - Dark mode support

- **Đa ngôn ngữ:**
  - Hỗ trợ Tiếng Việt và English
  - Tích hợp với hệ thống translation

## Cách sử dụng

### 1. Include CSS
```html
<link rel="stylesheet" href="/shared_components/market-indicators/market-indicators.css">
```

### 2. Include HTML Component
```javascript
// Load component HTML
fetch('/shared_components/market-indicators/market-indicators.html')
  .then(response => response.text())
  .then(html => {
    document.getElementById('your-container').innerHTML = html;
  });
```

### 3. Include JavaScript
```html
<script src="/shared_components/market-indicators/market-indicators.js" defer></script>
```

### 4. HTML Container
```html
<div id="market-indicators-container">
  <!-- Component will be loaded here -->
</div>
```

## File Structure

```
shared_components/market-indicators/
├── market-indicators.html    # HTML template
├── market-indicators.css     # Styles
├── market-indicators.js      # JavaScript logic
└── README.md                # Documentation
```

## API Dependencies

Component này cần WebSocket endpoint `/ws` để nhận dữ liệu thời gian thực với các message types:

- `dashboard_data` - Dữ liệu tổng quan dashboard
- `btc_price_update` - Cập nhật giá BTC
- `market_update` - Cập nhật dữ liệu thị trường

## Customization

### CSS Variables
Bạn có thể customize màu sắc và spacing bằng cách override CSS variables:

```css
.market-indicators-container {
  --primary-color: #3b82f6;
  --success-color: #059669;
  --danger-color: #dc2626;
  --neutral-color: #6b7280;
}
```

### Animation Settings
Thay đổi animation timing:

```javascript
const dashboard = new MarketIndicatorsDashboard();
dashboard.updateAnimationDuration = 500; // milliseconds
```

## Browser Support

- Chrome/Chromium 60+
- Firefox 60+
- Safari 12+
- Edge 79+

## Performance

- Lightweight: < 50KB total size
- WebSocket connection reuse
- Efficient DOM updates
- CSS animations với GPU acceleration

## Integration Examples

### Trong dashboards/home.html
```html
<!-- Include CSS -->
<link rel="stylesheet" href="/shared_components/market-indicators/market-indicators.css">

<!-- Container -->
<div id="market-indicators-container"></div>

<!-- Scripts -->
<script src="/shared_components/market-indicators/market-indicators.js" defer></script>
<script>
document.addEventListener('DOMContentLoaded', function() {
  fetch('/shared_components/market-indicators/market-indicators.html')
    .then(response => response.text())
    .then(html => {
      document.getElementById('market-indicators-container').innerHTML = html;
    });
});
</script>
```

### Trong crypto report pages
Component tự động khởi tạo khi tìm thấy element với id `market-indicators-dashboard`.

## Troubleshooting

### Component không load
- Kiểm tra WebSocket connection ở Developer Tools
- Đảm bảo route `/shared_components` đã được cấu hình
- Kiểm tra Console errors

### Dữ liệu không cập nhật
- Verify WebSocket server đang gửi message types đúng format
- Check network tab cho WebSocket messages
- Đảm bảo data format matches expected schema

### Styling issues
- Verify CSS file đã load
- Check for CSS conflicts với existing styles
- Use browser dev tools để debug layout

## License

MIT License - Xem file LICENSE để biết thêm chi tiết.
