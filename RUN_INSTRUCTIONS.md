# iTrader Backend - Running Instructions

## Prerequisites

1. PostgreSQL database running locally or accessible
2. Redis server running locally or accessible  
3. Python 3.11 with PyO3 support (already configured in the virtual environment)

## Quick Start

### 1. Set up environment variables

Create a `.env` file in the project root:

```bash
# Required
APP__admin_token=your-secure-admin-token

# Database (adjust to your setup)
DATABASE_URL=postgres://postgres:password@localhost/itrader

# Redis (adjust to your setup)
REDIS_URL=redis://localhost:6379

# Optional - Email settings
EMAIL_ADDRESS=your-email@gmail.com
EMAIL_PASSWORD=your-app-password

# Optional - AI settings
OPENROUTER_API_KEY=your-openrouter-api-key
```

### 2. Run the application

```bash
# Load environment variables and run
source .env && ./run.sh

# Or run with command line arguments
./run.sh --help

# Run in automatic mode (no confirmations)
./run.sh --auto
```

### 3. Without external services (for testing compilation only)

If you just want to verify the binary runs without connecting to services:

```bash
APP__admin_token=test ./run.sh --help
```

This will still fail when trying to connect to the database, but confirms the binary and configuration are working.

## Configuration

The application uses a layered configuration system:

1. Base configuration: `config/default.toml`
2. Environment-specific: `config/development.toml` (optional)
3. Environment variables: `APP__<section>__<key>` format
4. Command line arguments

## Troubleshooting

### "missing field admin_token" error
- Set the `APP__admin_token` environment variable
- Or add `admin_token = "your-token"` to your config file

### Database connection errors
- Ensure PostgreSQL is running
- Check DATABASE_URL is correct
- Verify database exists: `createdb itrader`

### Python library errors
- The `run.sh` script sets up the correct Python library path
- Always use `./run.sh` instead of running the binary directly

## Development Setup

For local development with Docker:

```bash
# Start PostgreSQL and Redis
docker run -d --name postgres -e POSTGRES_PASSWORD=password -p 5432:5432 postgres
docker run -d --name redis -p 6379:6379 redis

# Create database
docker exec -it postgres createdb -U postgres itrader

# Run migrations (when available)
# sqlx migrate run
```

## Binary Location

The compiled binary is located at: `target/debug/itrader-backend`

Always use the `run.sh` wrapper script to ensure proper environment setup.
EOF < /dev/null