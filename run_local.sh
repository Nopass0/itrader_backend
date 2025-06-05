#!/bin/bash

# Run iTrader Backend locally without Python SDK

echo "Starting iTrader Backend (without Python SDK)..."
echo

# Check if .env exists
if [ ! -f .env ]; then
    echo "Creating default .env file..."
    cat > .env << EOF
DATABASE_URL=postgresql://postgres:password@localhost/itrader
REDIS_URL=redis://localhost:6379
ADMIN_TOKEN=your-admin-token
OPENROUTER_API_KEY=your-openrouter-api-key
JWT_SECRET=your-jwt-secret-key
EOF
    echo "Please update .env with your actual credentials"
    echo
fi

# Create db directory structure
mkdir -p db/gate db/bybit db/gmail db/transactions db/checks

# Create a dummy Python lib to satisfy the linker
if [ ! -f /tmp/libpython3.11.so.1.0 ]; then
    echo "Creating dummy Python library..."
    touch /tmp/libpython3.11.so.1.0
fi

export LD_LIBRARY_PATH="/tmp:$LD_LIBRARY_PATH"

# Disable Python features
export ITRADER_DISABLE_PYTHON=1

echo "Note: Bybit P2P features requiring Python SDK will be disabled"
echo

# Check if running in auto mode
if [ "$1" == "--auto" ]; then
    echo "🤖 Running in AUTOMATIC mode - all actions will be auto-confirmed!"
    echo "⚠️  Transactions will be processed WITHOUT manual confirmation!"
    cargo run --bin itrader-backend -- --auto
else
    echo "👤 Running in MANUAL mode - actions require confirmation"
    echo "💡 To run in automatic mode, use: ./run_local.sh --auto"
    cargo run --bin itrader-backend
fi