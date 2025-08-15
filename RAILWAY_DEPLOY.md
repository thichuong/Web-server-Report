# Railway Deployment Guide

## Prerequisites
- Railway account (sign up at https://railway.app)
- Railway CLI installed (`npm install -g @railway/cli`)

## Deployment Steps

### 1. Install Railway CLI
```bash
npm install -g @railway/cli
```

### 2. Login to Railway
```bash
railway login
```

### 3. Initialize Railway Project
```bash
railway init
```

### 4. Set Up PostgreSQL Database
```bash
# Add PostgreSQL service to your project
railway add postgresql
```

### 5. Set Environment Variables
After adding PostgreSQL, Railway will automatically provide a `DATABASE_URL`. You need to set additional variables:

```bash
# Set the auto-update secret key
railway variables set AUTO_UPDATE_SECRET_KEY=your_secret_key_here

# Set host (Railway handles PORT automatically)
railway variables set HOST=0.0.0.0
```

### 6. Deploy
```bash
railway up
```

## Environment Variables Needed

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string (auto-provided by Railway) | `postgresql://user:pass@host:5432/db` |
| `AUTO_UPDATE_SECRET_KEY` | Secret key for auto-update functionality | `your_secret_key_here` |
| `HOST` | Host address (should be 0.0.0.0 for Railway) | `0.0.0.0` |
| `PORT` | Port number (auto-provided by Railway) | `8000` |

## Notes

- The application will automatically use Railway's provided `PORT` environment variable
- PostgreSQL database is required for the application to work
- Static files are served from the `/static` directory
- Templates are loaded from the `/templates` directory

## Troubleshooting

1. **Database Connection Issues**: Ensure PostgreSQL service is added and `DATABASE_URL` is set
2. **Port Issues**: Railway automatically sets the `PORT` variable, don't override it
3. **Static Files Not Loading**: Ensure `static/` and `templates/` directories are included in the build

## Alternative: One-Click Deploy

You can also deploy directly from GitHub:
1. Connect your GitHub repository to Railway
2. Set the environment variables in Railway dashboard
3. Railway will automatically detect the Dockerfile and deploy
