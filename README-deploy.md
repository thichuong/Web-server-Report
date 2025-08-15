Triển khai lên Railway (không dùng Docker)

Hướng dẫn ngắn gọn để deploy ứng dụng Rust này lên Railway bằng buildpacks (không dùng Docker).

1. Đảm bảo repo đã được đẩy lên Git (GitHub/GitLab).

2. Tạo file `Procfile` ở thư mục gốc (đã cung cấp sẵn):

```
web: ./target/release/web-server-report
```

3. Trên Railway:
- Chọn "New Project" → "Deploy from Git Repo" và kết nối repo của bạn.
- Railway sẽ phát hiện Rust và sử dụng buildpack để chạy `cargo build --release`.

4. Dịch vụ PostgreSQL & biến môi trường:
- Thêm một PostgreSQL service trong Railway (New → Database → PostgreSQL). Railway sẽ tạo các biến và thường sinh `DATABASE_URL` tự động.
- Thêm các biến môi trường cho project/app:
  - `DATABASE_URL` (từ service PostgreSQL)
  - `PORT` (Railway tự cấp PORT runtime; không cần set tĩnh, nhưng bạn có thể thêm `0.0.0.0` vào `HOST` nếu cần)
  - `HOST` = `0.0.0.0` (tùy chọn)
  - `AUTO_UPDATE_SECRET_KEY` nếu bạn dùng endpoint auto-update.

5. Build & Start command (nếu Railway không tự detect):
- Build command:

```
cargo build --release
```

- Start command:

```
./target/release/web-server-report
```

6. Lưu ý và debug:
- Railway cung cấp logs trong tab "Deployments" và "Logs".
- Nếu ứng dụng không lắng nghe đúng cổng, đảm bảo dùng biến `PORT` trong code hoặc cho phép bind tới `0.0.0.0` (app hiện đang dùng `HOST` và `PORT` env vars).
- Nếu cần tắt caching cho `chart_modules`, set `DEBUG=1`.

7. Thử nghiệm cục bộ trước khi đẩy:
- Build release cục bộ để kiểm tra:

```bash
cargo build --release
./target/release/web-server-report
```

Hoàn tất — sau khi deploy, kiểm tra endpoint `GET /health` để đảm bảo service chạy.
