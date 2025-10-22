# Open Graph Images

## Hướng dẫn tạo ảnh OG (Open Graph Images)

### Yêu cầu kích thước:
- **Kích thước khuyến nghị**: 1200x630 pixels (tỷ lệ 1.91:1)
- **Kích thước tối thiểu**: 600x315 pixels
- **Định dạng**: JPG, PNG (PNG được khuyến nghị cho chất lượng tốt hơn)
- **Dung lượng**: Dưới 8MB (khuyến nghị dưới 1MB)

### Ảnh OG dùng chung cho tất cả trang

Chúng ta đang dùng một file ảnh chung cho tất cả các trang: **image.jpg**

- Vị trí (relative): `shared_assets/images/image.jpg`
- URL đầy đủ (ví dụ production): `https://web-server-report-production.up.railway.app/shared_assets/images/image.jpg`

Nội dung gợi ý cho `image.jpg`:
- Kích thước: 1200x630px
- Nội dung: Logo / preview dashboard / tên "Crypto Dashboard"
- Text overlay (tùy chọn): "Crypto Dashboard - Phân Tích Thị Trường Tiền Mã Hóa"

### Tools để tạo ảnh OG:

1. **Online Tools (Miễn phí)**:
   - [Canva](https://www.canva.com) - Template "Facebook Post" (1200x630)
   - [Figma](https://www.figma.com) - Professional design tool
   - [Crello/VistaCreate](https://create.vista.com)
   - [Meta Business Suite](https://business.facebook.com) - Preview tool

2. **AI Tools**:
   - [DALL-E](https://openai.com/dall-e-2)
   - [Midjourney](https://www.midjourney.com)
   - [Stable Diffusion](https://stability.ai)

3. **Quick Screenshot Method**:
   - Chụp screenshot dashboard của bạn
   - Resize về 1200x630px
   - Thêm overlay text bằng tool online

### Sau khi tạo ảnh:

1. Lưu các file ảnh vào thư mục này (`shared_assets/images/`)
2. Cập nhật URL trong các file HTML (đã thực hiện):
   - Chúng ta đã thay tất cả meta tags để trỏ tới:
     `https://web-server-report-production.up.railway.app/shared_assets/images/image.jpg`
   - Nếu bạn muốn test local, dùng đường dẫn tương đối: `/shared_assets/images/image.jpg`

   ### Facebook App ID (fb:app_id)

   Các trang hiện có một placeholder meta tag cho Facebook App ID:

         <meta property="fb:app_id" content="2216301062195294" />

   Bạn nên thay `YOUR_FB_APP_ID` bằng App ID thực tế từ Facebook for Developers để xóa cảnh báo "Missing Properties: fb:app_id" trong Facebook Debugger.

   Bạn có thể lấy App ID từ:

   - https://developers.facebook.com/apps/ (tạo/app hoặc dùng app hiện có)

   Sau khi có App ID, cập nhật các file:

   - `dashboards/home.html`
   - `dashboards/crypto_dashboard/routes/reports/list.html`
   - `dashboards/crypto_dashboard/routes/reports/view.html`

   Ví dụ:

      <meta property="fb:app_id" content="123456789012345" />


### Test Open Graph Tags:

1. **Facebook Sharing Debugger**: 
   - https://developers.facebook.com/tools/debug/
   - Nhập URL và click "Debug" để xem preview

2. **LinkedIn Post Inspector**:
   - https://www.linkedin.com/post-inspector/

3. **Twitter Card Validator**:
   - https://cards-dev.twitter.com/validator

### Lưu ý:
- Facebook cache ảnh OG, cần "Scrape Again" trong Debugger để refresh
- Domain phải public để Facebook có thể truy cập và validate
- Nếu test local, có thể dùng ngrok để tạo public URL tạm thời
