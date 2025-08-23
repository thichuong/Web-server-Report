# Crypto Reports Architecture - Layer 5 Business Logic

## 📋 Tổng quan

Module `crypto_reports` là một Service Island thuộc Layer 5 (Business Logic) trong kiến trúc Service Islands. Module này chịu trách nhiệm xử lý tất cả các hoạt động liên quan đến báo cáo cryptocurrency, từ tạo báo cáo đến xuất PDF và phân tích dữ liệu thị trường.

## 🏗️ Kiến trúc Service Islands

### Vị trí trong kiến trúc
```
Layer 5: Business Logic (crypto_reports) 
   ↓
Layer 3: Communication Services
   ↓
Layer 2: External Services
   ↓
Layer 1: Infrastructure (ChartModulesIsland)
```

### Nguyên tắc kiến trúc nghiêm ngặt
- ✅ **Layer 5 → Layer 3 → Layer 2**: Luồng phụ thuộc chính xác
- ✅ **Layer 5 → Layer 1**: Sử dụng infrastructure services (ChartModulesIsland)
- ❌ **Layer 5 → Layer 2**: Không được phép truy cập trực tiếp
- ✅ **Tách biệt trách nhiệm**: Mỗi component có chức năng riêng biệt
- ✅ **Dependency Injection**: Sử dụng WebSocketServiceIsland từ Layer 3 và ChartModulesIsland từ Layer 1

## 📁 Cấu trúc Module

### Tệp tin chính
```
src/service_islands/layer5_business_logic/crypto_reports/
├── mod.rs                    # Main island coordinator
├── handlers.rs               # HTTP request handlers
├── pdf_generator.rs          # PDF generation component
├── report_creator.rs         # Report creation business logic
├── data_manager.rs          # Data processing and analytics
├── template_orchestrator.rs  # Template rendering management
└── tests.rs                 # Unit and integration tests
```

## 🧩 Components Chi tiết

### 1. CryptoReportsIsland (mod.rs)
**Chức năng chính:**
- Điều phối tất cả các component con
- Quản lý dependency với Layer 3 (WebSocketServiceIsland)
- Fetch dữ liệu thị trường real-time theo kiến trúc nghiêm ngặt
- Health check tổng thể cho toàn bộ island

**Tính năng key:**
```rust
pub struct CryptoReportsIsland {
    pub handlers: handlers::CryptoHandlers,
    pub pdf_generator: pdf_generator::PdfGenerator,
    pub report_creator: report_creator::ReportCreator,
    pub data_manager: data_manager::DataManager,
    pub template_orchestrator: template_orchestrator::TemplateOrchestrator,
    pub websocket_service: Option<Arc<WebSocketServiceIsland>>,
}
```

**Luồng dữ liệu:**
1. `fetch_realtime_market_data()`: Layer 5 → Layer 3 → Layer 2
2. Chuẩn hóa dữ liệu cho JavaScript client
3. Xử lý lỗi và fallback scenarios

### 2. CryptoHandlers (handlers.rs)
**Chức năng chính:**
- Xử lý tất cả HTTP requests liên quan đến crypto reports
- Tích hợp với Template Engine (không tạo HTML thủ công)
- Quản lý routing và response handling

**Tính năng key:**
- Report listing và detail handlers
- PDF generation endpoints
- API integration với front-end
- Error handling và status codes

### 3. PdfGenerator (pdf_generator.rs)
**Chức năng chính:**
- Tạo PDF từ crypto reports
- Tối ưu hóa cho định dạng A4
- Template rendering cho PDF output

**Tính năng key:**
```rust
pub async fn crypto_report_pdf_with_tera(
    &self, 
    app_state: &Arc<AppState>, 
    report_id: i32
) -> Result<String, Box<dyn StdError + Send + Sync>>
```

### 4. ReportCreator (report_creator.rs)
**Chức năng chính:**
- Tạo và quản lý crypto reports
- Xử lý business logic cho report generation
- Integration với CryptoDataService (Layer 3)
- Sử dụng ChartModulesIsland (Layer 1) cho chart JavaScript modules

**Architecture Dependencies:**
```rust
pub struct ReportCreator {
    pub data_service: CryptoDataService,           // Layer 3
    pub chart_modules_island: ChartModulesIsland, // Layer 1
}
```

**Key Methods:**
- `fetch_and_cache_latest_report()`: Lấy report mới nhất qua Layer 3
- `fetch_and_cache_report_by_id()`: Lấy report theo ID qua Layer 3  
- `get_chart_modules_content()`: Lấy chart modules qua Layer 1 Infrastructure
- `get_available_chart_modules()`: Danh sách modules có sẵn qua Layer 1

**Chart Modules Integration:**
ReportCreator không còn tự xử lý file I/O cho chart modules mà delegate xuống ChartModulesIsland trong Layer 1, tuân thủ nguyên tắc architectural separation.

**Data Models:**
```rust
pub struct Report {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

**Supporting Models:**
- `ReportSummary`: Cho report listing
- `ReportListItem`: Với formatted dates

## 🏛️ Layer 1 Infrastructure Dependencies

### ChartModulesIsland Integration
**Vị trí:** `service_islands/layer1_infrastructure/chart_modules_island/`

**Chức năng chính:**
- Quản lý JavaScript chart modules từ `shared_assets/js/chart_modules/`
- Concurrent loading của multiple chart files
- Priority-based loading order (gauge.js, bar.js, line.js, doughnut.js)
- Error handling và module wrapping
- Health checking cho chart modules availability

**Integration trong Crypto Reports:**
```rust
// ReportCreator sử dụng ChartModulesIsland
let chart_content = self.chart_modules_island.get_cached_chart_modules_content().await;
```

**Key Features:**
- ✅ **Concurrent Loading**: Tải multiple JS files song song
- ✅ **Error Wrapping**: Mỗi module được wrap trong try-catch
- ✅ **Priority Order**: Configurable loading sequence
- ✅ **Health Check**: Verify chart modules accessibility
- ✅ **Caching Ready**: Sẵn sàng tích hợp với L1 cache system
- ✅ **Performance Optimized**: Spawn blocking cho concatenation

**Output Format:**
```javascript
// ==================== gauge.js ====================
try {
    // Chart module code here
} catch (error) {
    console.error('Error loading chart module gauge.js:', error);
}
// ==================== End gauge.js ====================
```

**Architecture Benefits:**
- **Separation of Concerns**: File I/O logic tách khỏi business logic
- **Reusability**: ChartModulesIsland có thể được dùng bởi các services khác
- **Maintainability**: Centralized chart modules management
- **Testability**: Infrastructure và business logic test riêng biệt

### 5. DataManager (data_manager.rs)
**Chức năng chính:**
- Xử lý và phân tích dữ liệu crypto
- Tạo insights từ market data
- Data transformation và aggregation

**Trạng thái hiện tại:**
- Component cơ bản đã được khởi tạo
- Sẵn sàng cho việc mở rộng tính năng analytics

### 6. TemplateOrchestrator (template_orchestrator.rs)
**Chức năng chính:**
- Quản lý tất cả template rendering operations
- Chuẩn bị context cho templates
- Injection chart modules và assets

**Tính năng key:**
```rust
pub struct TemplateContext {
    pub report: Report,
    pub chart_modules_content: String,
    pub current_route: String,
    pub current_lang: String,
    pub current_time: String,
    pub pdf_url: String,
    pub additional_context: Option<HashMap<String, serde_json::Value>>,
}
```

## 🔄 Data Flow

### Market Data Fetch Flow
```
1. Client Request → Layer 5 (CryptoReportsIsland)
2. Layer 5 → Layer 3 (WebSocketServiceIsland)
3. Layer 3 → Layer 2 (External APIs: CoinGecko, Fear&Greed)
4. Layer 2 → Layer 3 (Raw data)
5. Layer 3 → Layer 5 (Processed data)
6. Layer 5 → Client (Normalized JSON)
```

### Report Generation Flow
```
1. HTTP Request → CryptoHandlers
2. CryptoHandlers → ReportCreator (business logic)
3. ReportCreator → DataManager (data processing)
4. ReportCreator → ChartModulesIsland (Layer 1 - get chart modules)
5. DataManager → TemplateOrchestrator (context preparation)
6. TemplateOrchestrator → Tera Engine (rendering with chart modules)
7. Response → Client (HTML/PDF)
```

### Chart Modules Loading Flow
```
1. ReportCreator → ChartModulesIsland.get_chart_modules_content()
2. ChartModulesIsland → File System (shared_assets/js/chart_modules/)
3. Concurrent loading of: gauge.js, bar.js, line.js, doughnut.js
4. Error wrapping và concatenation
5. Return → ReportCreator (complete chart modules string)
6. Integration → Template context for rendering
```

## 🧪 Testing Strategy

### Test Coverage
- **Unit Tests**: Mỗi component có tests riêng
- **Integration Tests**: Test các interactions giữa components
- **Health Checks**: Automated health monitoring

### Test Files
- `tests.rs`: Tập trung tất cả test cases
- Coverage cho tất cả public methods
- Mock dependencies cho isolated testing

## 🔧 Configuration

### Dependencies
- **Layer 3 Dependency**: WebSocketServiceIsland (required)
- **Layer 1 Dependency**: ChartModulesIsland (required)
- **Internal Dependencies**: AppState, Tera, SQLx
- **External Crates**: serde, tokio, anyhow, chrono, futures

### Environment Setup
- Database connection để lưu trữ reports
- Template directory cho Tera engine
- Assets directory cho chart modules

## 📊 Performance Considerations

### Optimization Strategies
1. **Caching**: L1/L2 cache cho reports và market data
2. **Async Operations**: Non-blocking I/O operations
3. **Connection Pooling**: Database và HTTP connections
4. **Template Caching**: Pre-compiled Tera templates

### Resource Management
- Memory-efficient data structures
- Proper cleanup của temporary resources
- Connection lifecycle management

## 🚀 Future Roadmap

### Planned Features
1. **Advanced Analytics**: Technical indicators, trend analysis
2. **Enhanced PDF Generation**: Charts, graphs, visual analytics
3. **Real-time Updates**: WebSocket integration cho live data
4. **Multi-language Support**: I18n cho templates
5. **Custom Report Templates**: User-defined report formats

### Architecture Improvements
1. **Performance Monitoring**: Metrics và tracing
2. **Error Recovery**: Robust error handling và retry logic
3. **Scalability**: Horizontal scaling support
4. **Security**: Authentication và authorization
5. **Chart Modules Caching**: Tích hợp ChartModulesIsland với L1 Cache System
6. **Dynamic Chart Loading**: Runtime chart modules configuration
7. **Chart Modules Versioning**: Support multiple chart library versions

## 🔍 Monitoring & Debugging

### Logging Strategy
- Structured logging với tracing
- Performance metrics per operation
- Error tracking và alerting

### Health Checks
```rust
pub async fn health_check(&self) -> bool {
    let handlers_ok = self.handlers.health_check().await;
    let pdf_ok = self.pdf_generator.health_check().await;
    let creator_ok = self.report_creator.health_check().await; // Includes chart modules check
    let manager_ok = self.data_manager.health_check().await;
    let orchestrator_ok = self.template_orchestrator.health_check().await;
    
    handlers_ok && pdf_ok && creator_ok && manager_ok && orchestrator_ok
}

// ReportCreator health check now includes ChartModulesIsland
pub async fn health_check(&self) -> bool {
    self.chart_modules_island.health_check().await // Checks shared_assets/js/chart_modules/
}
```

## 📚 API Documentation

### Main Endpoints (handled by CryptoHandlers)
- `GET /crypto/reports` - List all crypto reports
- `GET /crypto/reports/{id}` - Get specific report
- `GET /crypto/reports/{id}/pdf` - Generate PDF version
- `GET /api/crypto/market-data` - Real-time market data

### Response Formats
- **JSON**: Cho API endpoints
- **HTML**: Cho web interface  
- **PDF**: Cho downloadable reports

---

## 🔗 Liên kết

- [Service Islands Architecture](./SERVICE_ISLANDS_ARCHITECTURE.md)
- [WebSocket Real-time Implementation](./WEBSOCKET_REALTIME_IMPLEMENTATION.md)
- [Cache Architecture Analysis](./CACHE_ARCHITECTURE_ANALYSIS.md)
- [Chart Modules Island README](../src/service_islands/layer1_infrastructure/chart_modules_island/README.md)
