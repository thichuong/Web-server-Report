# CoinMarketCap API Fallback Setup

## Tổng Quan

Ứng dụng hiện đã hỗ trợ CoinMarketCap làm API dự phòng khi CoinGecko gặp sự cố hoặc bị rate limit. Điều này đảm bảo tính ổn định và độ tin cậy cao hơn cho việc lấy dữ liệu thị trường cryptocurrency.

## Cách Hoạt Động

1. **Primary API**: CoinGecko (miễn phí, được sử dụng trước)
2. **Fallback API**: CoinMarketCap (yêu cầu API key, chỉ sử dụng khi CoinGecko thất bại)

## Cài Đặt CoinMarketCap API

### 1. Lấy API Key

1. Đăng ký tài khoản tại [CoinMarketCap Pro API](https://pro.coinmarketcap.com/)
2. Tạo API key mới
3. Copy API key của bạn

### 2. Cấu Hình Environment Variable

Thêm API key vào file `.env` hoặc environment variables:

```bash
# CoinMarketCap API Key (tùy chọn - cho fallback support)
CMC_API_KEY=your_coinmarketcap_api_key_here

# TAAPI Secret (bắt buộc)
TAAPI_SECRET=your_taapi_secret_here
```

### 3. Khởi Động Lại Ứng Dụng

Sau khi cấu hình API key, khởi động lại ứng dụng:

```bash
cargo run
```

## Kiểm Tra Trạng Thái

Khi khởi động, ứng dụng sẽ hiển thị:

- ✅ **Với CoinMarketCap API key**: `"🔑 CoinMarketCap API key found - enabling fallback support"`
- ⚠️ **Không có API key**: `"⚠️ No CoinMarketCap API key - using CoinGecko only"`

## API Endpoints Được Hỗ Trợ

### CoinGecko (Primary)
- Global market data: `/api/v3/global`
- Bitcoin price: `/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true`

### CoinMarketCap (Fallback)
- Global market data: `/v1/global-metrics/quotes/latest`
- Bitcoin price: `/v1/cryptocurrency/quotes/latest?symbol=BTC`

## Rate Limiting & Error Handling

- **Retry Logic**: Exponential backoff với tối đa 3 lần thử
- **Circuit Breaker**: Tự động chuyển đổi khi API primary thất bại
- **Graceful Degradation**: Ứng dụng vẫn hoạt động ngay cả khi cả 2 API đều thất bại

## Giám Sát

API statistics được theo dõi và có thể xem thông qua:

```rust
let stats = market_data_api.get_api_stats();
```

Bao gồm:
- `total_api_calls`: Tổng số lời gọi API
- `successful_calls`: Số lời gọi thành công
- `failed_calls`: Số lời gọi thất bại
- `success_rate`: Tỷ lệ thành công (%)
- `has_coinmarketcap_key`: Có CoinMarketCap key hay không

## Lưu Ý

1. **Chi Phí**: CoinMarketCap có giới hạn free tier, hãy theo dõi usage
2. **Performance**: CoinMarketCap có thể chậm hơn CoinGecko một chút
3. **Data Format**: Response từ 2 API được chuẩn hóa để đảm bảo tương thích
4. **Logging**: Tất cả API calls và fallbacks đều được log để debug

## Troubleshooting

### API Key Không Hoạt Động
```bash
❌ CoinMarketCap BTC API returned status: 401
```
- Kiểm tra API key có đúng không
- Đảm bảo API key chưa hết hạn
- Kiểm tra permissions của API key

### Rate Limit Issues
```bash
⚠️ CoinMarketCap rate limit (429), retrying...
```
- Đây là hành vi bình thường, ứng dụng sẽ tự retry
- Nếu xảy ra thường xuyên, cân nhắc upgrade plan

### Both APIs Failing
```bash
❌ Both CoinGecko and CoinMarketCap failed
```
- Kiểm tra kết nối internet
- Kiểm tra firewall settings
- Xem logs để biết chi tiết lỗi cụ thể
