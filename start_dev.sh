#!/bin/bash

# Development startup script for iTrader Backend

echo "==================================="
echo "iTrader Backend Development Server"
echo "==================================="
echo

# Check environment
echo "Checking environment..."

# Check if .env exists
if [ ! -f .env ]; then
    echo "⚠️  No .env file found. Creating from example..."
    cat > .env << EOF
# Database Configuration
DATABASE_URL=postgresql://postgres:password@localhost/itrader
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=1

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=10

# API Keys
OPENROUTER_API_KEY=your-openrouter-api-key
JWT_SECRET=your-jwt-secret-key

# Email Configuration (optional)
EMAIL_ADDRESS=your-email@gmail.com
EMAIL_PASSWORD=your-app-password

# Admin Token
ADMIN_TOKEN=dev-token-123
EOF
    echo "✅ Created .env file - please update with your credentials"
fi

# Create directory structure
echo "Setting up directories..."
mkdir -p db/gate db/bybit db/gmail db/transactions db/checks data logs

# Create example account files
if [ ! -f db/settings_example.json ]; then
    echo "Creating example account files..."
    cat > db/settings_example.json << 'EOF'
{
  "gate_accounts": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "login": "user@example.com",
      "password": "your_password_here",
      "cookies": null,
      "last_auth": null,
      "balance": 0.0,
      "created_at": "2025-01-04T12:00:00Z",
      "updated_at": "2025-01-04T12:00:00Z"
    }
  ],
  "bybit_accounts": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "api_key": "your_api_key_here",
      "api_secret": "your_api_secret_here",
      "active_ads": 0,
      "last_used": null,
      "created_at": "2025-01-04T12:00:00Z",
      "updated_at": "2025-01-04T12:00:00Z"
    }
  ]
}
EOF
fi

# Try to find Python library
echo "Checking Python environment..."
PYTHON_VERSION=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")' 2>/dev/null || echo "none")

if [ "$PYTHON_VERSION" != "none" ]; then
    echo "✅ Found Python $PYTHON_VERSION"
    
    # Try to find the Python library
    PYTHON_LIB=$(python3 -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))' 2>/dev/null)
    if [ -n "$PYTHON_LIB" ]; then
        export LD_LIBRARY_PATH="$PYTHON_LIB:$LD_LIBRARY_PATH"
    fi
    
    # Also try common locations
    export LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu:/usr/local/lib:$LD_LIBRARY_PATH"
    
    # Add local libs directory for Python compatibility
    export LD_LIBRARY_PATH="$(pwd)/libs:$LD_LIBRARY_PATH"
else
    echo "⚠️  Python not found - Bybit P2P features will be limited"
fi

echo
echo "Starting server..."
echo "=================="

# Display usage instructions
cat << EOF

📌 Quick Start Guide:
   
1. WebSocket API: ws://localhost:8080/ws
2. Admin API: http://localhost:8080/admin (requires admin token)
3. Health Check: http://localhost:8080/health

📁 Account Management:
   - Gate accounts: db/gate/*.json
   - Bybit accounts: db/bybit/*.json
   - See ACCOUNT_SETUP.md for details

🔧 Configuration:
   - Edit .env for API keys and settings
   - Edit config/default.toml for advanced settings

EOF

# Check command line arguments
if [ "$1" == "--auto" ]; then
    echo "🤖 AUTOMATIC MODE - All actions will be auto-confirmed!"
    echo
    export RUST_LOG=info,itrader_backend=debug
    cargo run --bin itrader-backend -- --auto
else
    echo "👤 MANUAL MODE - Actions require confirmation"
    echo "💡 Use './start_dev.sh --auto' for automatic mode"
    echo
    export RUST_LOG=info,itrader_backend=debug
    cargo run --bin itrader-backend
fi