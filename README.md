# Web-server-Report (Client-facing Rust server)

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

