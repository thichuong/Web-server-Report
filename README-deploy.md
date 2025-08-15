Railway deployment notes

Quick steps to deploy on Railway using the provided Dockerfile:

1. Create a new Railway project and connect your GitHub repo (or link via CLI).
2. In Railway, add a new service and choose 'Deploy from Dockerfile' (or use the provided `railway.json`).
3. Set environment variables in the Railway service settings:
   - DATABASE_URL: postgresql://<user>:<pass>@<host>:<port>/<db>
   - AUTO_UPDATE_SECRET_KEY: a secret string
   - PORT: 8000 (Railway usually provides this automatically)
4. Deploy. Railway will build the Docker image using the `Dockerfile` and run the `web` service.

Notes & troubleshooting
- The Dockerfile uses `SQLX_OFFLINE=1` to avoid runtime DB checks during build; keep it if you don't want the build to require DB access.
- If you want to use sqlx's offline compile-time verification, remove `SQLX_OFFLINE` and set up a `DATABASE_URL` build-time secret in Railway.
- Ensure the PostgreSQL add-on is provisioned and `DATABASE_URL` is set correctly.
- The server binds to `HOST` and `PORT` environment variables; Railway provides `PORT` automatically.

Environment
- The app reads `.env` keys: `DATABASE_URL`, `AUTO_UPDATE_SECRET_KEY`, `HOST`, `PORT`.
