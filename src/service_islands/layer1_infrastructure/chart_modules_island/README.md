# Chart Modules Island - Layer 1 Infrastructure

## Tổng quan

Chart Modules Island là một thành phần cơ sở hạ tầng trong Layer 1 của kiến trúc Service Islands, chịu trách nhiệm quản lý và phục vụ các JavaScript chart modules từ thư mục `shared_assets/js/chart_modules/`.

## Tính năng chính

- **Tải song song**: Sử dụng concurrent futures để tải nhiều file cùng lúc
- **Thứ tự ưu tiên**: Hỗ trợ định nghĩa thứ tự tải modules theo độ ưu tiên
- **Xử lý lỗi**: Wrap từng module trong try-catch để tránh lỗi runtime
- **Cấu hình linh hoạt**: Có thể tùy chỉnh thư mục gốc và thứ tự ưu tiên
- **Health check**: Kiểm tra tình trạng hoạt động của service
- **Logging**: Ghi log chi tiết cho debugging và monitoring

## Kiến trúc

```
Layer 5 (Business Logic)
    ↓ sử dụng
Layer 1 (Infrastructure) - ChartModulesIsland
    ↓ đọc từ
File System - shared_assets/js/chart_modules/
```

## Cách sử dụng

### Sử dụng cơ bản

```rust
use crate::service_islands::layer1_infrastructure::ChartModulesIsland;

// Tạo instance mới
let chart_modules = ChartModulesIsland::new();

// Kiểm tra health
let is_healthy = chart_modules.health_check().await;

// Tải tất cả chart modules
let content = chart_modules.get_chart_modules_content().await;

// Lấy danh sách modules có sẵn
let available = chart_modules.get_available_modules().await;
```

### Cấu hình tùy chỉnh

```rust
let custom_priority = vec![
    "line.js".to_string(),
    "bar.js".to_string(),
    "gauge.js".to_string(),
    "doughnut.js".to_string(),
];

let chart_modules = ChartModulesIsland::with_config(
    "custom/path/to/modules".to_string(),
    custom_priority,
);
```

### Thông qua Business Logic Layer

```rust
use crate::service_islands::layer5_business_logic::crypto_reports::report_creator::ReportCreator;

let report_creator = ReportCreator::new();
let content = report_creator.get_chart_modules_content().await;
```

## Thứ tự ưu tiên mặc định

1. `gauge.js` - Gauge charts
2. `bar.js` - Bar charts  
3. `line.js` - Line charts
4. `doughnut.js` - Doughnut charts
5. Các file khác theo thứ tự alphabet

## Output format

Mỗi module được wrap trong format sau:

```javascript
// ==================== filename.js ====================
try {
    // Nội dung module
} catch (error) {
    console.error('Error loading chart module filename.js:', error);
}
// ==================== End filename.js ====================
```

## Performance

- **Concurrent loading**: Tải nhiều file đồng thời
- **Spawn blocking**: Sử dụng thread pool cho việc concatenate để tránh block async runtime
- **Caching ready**: Sẵn sàng tích hợp với cache system khi Layer 1 cache được implement

## Integration points

- **Layer 5 Business Logic**: ReportCreator sử dụng để lấy chart modules
- **Layer 1 Cache System**: Sẵn sàng tích hợp khi cache system được implement
- **File System**: Đọc trực tiếp từ shared_assets directory

## Error handling

- Graceful degradation khi không tìm thấy thư mục
- Individual module error wrapping
- Detailed error logging
- Fallback messages cho missing modules

## Examples

Xem `examples.rs` để biết các ví dụ sử dụng chi tiết.

## Dependencies

- `tokio` - Async runtime và file operations
- `futures` - Concurrent futures utilities
- `std::path` - Path operations
- `std::env` - Environment variables

## Development

### Testing

```bash
cargo test chart_modules
```

### Running examples

```bash
cargo run --example chart_modules
```

### Debugging

Set environment variable `DEBUG=1` để enable debug logging.
