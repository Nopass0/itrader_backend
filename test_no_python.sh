#!/bin/bash

# Test running without Python dependencies
echo "Testing iTrader Backend without Python dependencies..."
echo

# Set environment variables
export DATABASE_URL="postgresql://postgres:password@localhost/itrader"
export REDIS_URL="redis://localhost:6379"
export ADMIN_TOKEN="test-token-123"
export GATE_API_URL="https://panel.gate.cx/api/v1"

# Create minimal .env if needed
if [ ! -f .env ]; then
    cat > .env << EOF
DATABASE_URL=postgresql://postgres:password@localhost/itrader
REDIS_URL=redis://localhost:6379
ADMIN_TOKEN=test-token-123
EOF
fi

# Create db directory structure
mkdir -p db/gate db/bybit db/gmail db/transactions db/checks

# Run without Python
echo "Starting application..."
cargo run --bin itrader-backend 2>&1 | head -100