#!/bin/bash

echo "Starting iTrader Backend in development mode with hot reload..."

# Check if OpenSSL is installed
if ! pkg-config --exists openssl; then
    echo "❌ OpenSSL development libraries not found!"
    echo "Please run: ./install-deps.sh"
    exit 1
fi

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch for hot reload..."
    cargo install cargo-watch
fi

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "Installing sqlx-cli for database migrations..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Check if .env file exists
if [ ! -f .env ]; then
    echo "Creating .env file with default values..."
    cat > .env << EOF
# Database
DATABASE_URL=postgresql://postgres:root@localhost/itrader_db
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2

# Redis
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=10

# Encryption (32-byte key)
ENCRYPTION_KEY=development-key-do-not-use-in-prod

# API Keys (you need to set these)
OPENROUTER_API_KEY=your-openrouter-api-key
JWT_SECRET=development-jwt-secret

# Email Configuration (optional for development)
EMAIL_ADDRESS=test@example.com
EMAIL_PASSWORD=test-password
IMAP_SERVER=imap.gmail.com
IMAP_PORT=993

# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Logging
RUST_LOG=info,itrader_backend=debug

# Development
HOT_RELOAD=true
EOF
    echo "Created .env file with default values"
    echo "⚠️  Please update OPENROUTER_API_KEY with your actual key"
fi

# Check if PostgreSQL is running
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo "PostgreSQL is not running. Please start PostgreSQL first."
    echo "On Ubuntu/Debian: sudo systemctl start postgresql"
    echo "On macOS: brew services start postgresql"
    exit 1
fi

# Create database if it doesn't exist
echo "Checking database..."
PGPASSWORD=root psql -h localhost -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'itrader_db'" | grep -q 1 || {
    echo "Creating database itrader_db..."
    PGPASSWORD=root createdb -h localhost -U postgres itrader_db
    if [ $? -eq 0 ]; then
        echo "Database created successfully"
    else
        echo "Failed to create database. Make sure PostgreSQL user 'postgres' with password 'root' exists"
        echo "You can create it with: sudo -u postgres psql -c \"ALTER USER postgres PASSWORD 'root';\""
        exit 1
    fi
}

# Run migrations
echo "Running database migrations..."
DATABASE_URL="postgresql://postgres:root@localhost/itrader_db" sqlx migrate run
if [ $? -eq 0 ]; then
    echo "Migrations completed successfully"
else
    echo "Failed to run migrations"
    exit 1
fi

# Check if Redis is running
if ! redis-cli ping > /dev/null 2>&1; then
    echo "Redis is not running. Please start Redis first."
    echo "On Ubuntu/Debian: sudo systemctl start redis"
    echo "On macOS: brew services start redis"
    exit 1
fi

# Export development environment
export RUST_LOG=debug,itrader_backend=trace
export RUN_MODE=development

# Run with hot reload
echo "Starting with hot reload..."
echo "Press Ctrl+C to stop"
cargo watch -x run