#!/bin/bash

echo "=== iTrader Auto-Trader Demo ==="
echo

# Set up environment
export DATABASE_URL="postgresql://postgres:password@localhost/itrader_db"
export REDIS_URL="redis://localhost:6379"
export ENCRYPTION_KEY="your-32-byte-encryption-key-here!"
export OPENROUTER_API_KEY="your-api-key"
export EMAIL_ADDRESS="your-email@gmail.com"
export EMAIL_PASSWORD="your-app-password"

# Create data directory if it doesn't exist
mkdir -p data

# 1. Initialize database
echo "1. Setting up database..."
sqlx database create
sqlx migrate run

# 2. Add accounts
echo -e "\n2. Adding accounts..."

# Add Gate.io account
echo "Adding Gate.io account..."
cargo run --bin manage_accounts -- add-gate trader1@example.com password123

# Add Bybit accounts
echo "Adding Bybit accounts..."
cargo run --bin manage_accounts -- add-bybit bybit_account_1 "API_KEY_1" "API_SECRET_1"
cargo run --bin manage_accounts -- add-bybit bybit_account_2 "API_KEY_2" "API_SECRET_2"

# 3. Show account stats
echo -e "\n3. Account Statistics:"
cargo run --bin manage_accounts -- stats

# 4. Start the auto-trader
echo -e "\n4. Starting auto-trader..."
echo "Press Ctrl+C to stop"
echo

# Run with debug logging
RUST_LOG=info,itrader_backend=debug cargo run

echo -e "\nAuto-trader stopped."