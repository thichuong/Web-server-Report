# Web Server Report

Một ứng dụng web server được viết bằng Rust sử dụng Axum framework để hiển thị các báo cáo đầu tư.

## Tính năng

- Hiển thị báo cáo đầu tư với biểu đồ tương tác
- Hỗ trợ đa ngôn ngữ (Tiếng Việt/English)
- Giao diện responsive
- API RESTful
- Kết nối PostgreSQL database

## Công nghệ sử dụng

- **Backend**: Rust + Axum
- **Database**: PostgreSQL
- **Frontend**: HTML, CSS, JavaScript với Chart.js
- **Template Engine**: Tera

## Cài đặt Local

1. Clone repository:
```bash
git clone <repository-url>
cd Web-server-Report
```

2. Cài đặt Rust (nếu chưa có):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3. Copy file environment:
```bash
cp .env.example .env
```

4. Cập nhật thông tin database trong file `.env`:
```
DATABASE_URL=postgresql://username:password@localhost:5432/database_name
AUTO_UPDATE_SECRET_KEY=your_secret_key
HOST=0.0.0.0
PORT=8000
```

5. Chạy ứng dụng:
```bash
cargo run
```

## Deploy lên Railway

### Bước 1: Chuẩn bị

1. Đăng ký tài khoản tại [Railway.app](https://railway.app)
2. Cài đặt Railway CLI (tùy chọn):
```bash
npm install -g @railway/cli
```

### Bước 2: Tạo PostgreSQL Database

1. Đăng nhập vào Railway dashboard
2. Tạo project mới
3. Add PostgreSQL service từ Templates
4. Copy DATABASE_URL từ Variables tab

### Bước 3: Deploy Application

**Cách 1: Qua GitHub (Khuyến nghị)**

1. Push code lên GitHub repository
2. Trong Railway dashboard, tạo service mới
3. Connect GitHub repository
4. Railway sẽ tự động detect Rust project và build

**Cách 2: Qua Railway CLI**

1. Login Railway CLI:
```bash
railway login
```

2. Link project:
```bash
railway link
```

3. Deploy:
```bash
railway up
```

### Bước 4: Cấu hình Environment Variables

Trong Railway dashboard, vào Variables tab và thêm:

```
DATABASE_URL=<postgresql-url-from-railway>
AUTO_UPDATE_SECRET_KEY=<your-secret-key>
HOST=0.0.0.0
PORT=8000
```

### Bước 5: Custom Domain (Tùy chọn)

1. Vào Settings tab trong Railway dashboard
2. Thêm custom domain nếu cần

## Cấu trúc Project

```
├── src/
│   └── main.rs          # Main application logic
├── static/              # Static files (CSS, JS, HTML)
├── templates/           # Tera templates
├── Cargo.toml          # Rust dependencies
├── railway.json        # Railway deployment config
├── nixpacks.toml       # Build configuration
├── Dockerfile          # Docker configuration (alternative)
└── .env.example        # Environment variables template
```

## API Endpoints

- `GET /` - Homepage
- `GET /health` - Health check
- `GET /report/:id` - View specific report
- `GET /reports` - List all reports
- `GET /upload` - Upload page
- `GET /auto-update-system-:secret` - Auto update endpoint

## Troubleshooting

### Build Issues
- Đảm bảo Rust version >= 1.70
- Check dependencies trong Cargo.toml

### Database Connection
- Verify DATABASE_URL format
- Ensure PostgreSQL service is running
- Check network connectivity

### Railway Deployment
- Check build logs trong Railway dashboard
- Verify environment variables
- Ensure all required files are committed to git

## Support

Nếu gặp vấn đề, vui lòng tạo issue trong repository. (Client-facing Rust server)

This is a standalone Rust/Axum web server that reads reports from the PostgreSQL
database created by the `Crypto-Dashboard-and-AI-ReportGenerator` project and
serves them to client users. The admin UI and AI-driven report creation remain in
`Crypto-Dashboard-and-AI-ReportGenerator`; this service only reads the `report`
table and serves HTML/CSS/JS stored in the database.

Quick start

1. Copy `.env.example` to `.env` and set `DATABASE_URL` to the same Postgres used by the admin project.
2. Install Rust toolchain (rustup + cargo).
3. Build and run:

```bash
cargo build --release
cargo run --release
```

Default listen address: `0.0.0.0:8000` (configurable by `HOST` and `PORT` env vars).

Routes
- GET /health
- GET / -> latest report HTML
- GET /report/:id -> report HTML
- GET /pdf-template/:id -> same as report HTML
- GET /reports?page=N -> JSON list of reports
- GET /upload -> static upload page
- GET /auto-update-system-:secret -> requires `AUTO_UPDATE_SECRET_KEY`

Notes
- The server expects the `report` table to match the schema in the admin project.
- Static assets referenced in report HTML should be served from this project's `static/` folder or by an external CDN. You can extend the server to serve more static routes if needed.

