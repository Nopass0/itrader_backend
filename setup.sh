#!/bin/bash
# Setup script for Auto-Trader System

set -e

echo "🚀 Setting up Auto-Trader System..."

# Check Python version
echo "📌 Checking Python version..."
python_version=$(python3 --version 2>&1 | awk '{print $2}')
required_version="3.8"

if [ "$(printf '%s\n' "$required_version" "$python_version" | sort -V | head -n1)" != "$required_version" ]; then 
    echo "❌ Error: Python $required_version or higher is required (found $python_version)"
    exit 1
fi
echo "✅ Python $python_version is installed"

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    echo "📦 Creating virtual environment..."
    python3 -m venv venv
else
    echo "✅ Virtual environment already exists"
fi

# Activate virtual environment
echo "🔄 Activating virtual environment..."
source venv/bin/activate

# Upgrade pip
echo "📦 Upgrading pip..."
pip install --upgrade pip

# Install dependencies
echo "📦 Installing Python dependencies..."
pip install -r requirements.txt

# Check for Tesseract OCR
echo "🔍 Checking for Tesseract OCR..."
if command -v tesseract &> /dev/null; then
    echo "✅ Tesseract OCR is installed"
else
    echo "⚠️  Tesseract OCR is not installed"
    echo "   To install on Ubuntu/Debian: sudo apt-get install tesseract-ocr tesseract-ocr-rus"
    echo "   To install on macOS: brew install tesseract tesseract-lang"
fi

# Create default configuration if it doesn't exist
if [ ! -f "config/default.toml" ]; then
    echo "📝 Creating default configuration..."
    mkdir -p config
    cat > config/default.toml << EOF
# Default configuration for Auto-Trader System

[gate]
poll_interval = 15  # seconds

[bybit]
api_key = ""
api_secret = ""
testnet = false

[payment]
bank = "Тинькофф"
phone = "+7 XXX XXX-XX-XX"
method_ids = ["75", "382"]  # Тинькофф, СБП
receipt_email = "your-receipt@email.com"

[email]
monitoring_enabled = true
imap_server = "imap.gmail.com"
imap_port = 993
username = ""
password = ""
check_interval = 30  # seconds
required_receipt_sender = "noreply@tinkoff.ru"

[trading]
profit_margin_percent = 2.0
min_order_amount = 1000.0  # RUB
max_order_amount = 50000.0  # RUB
ad_remarks = "Быстрая сделка. Отправьте чек на email после оплаты."

[database]
path = "trader.db"

[logging]
level = "INFO"
file = "trader.log"
EOF
    echo "✅ Default configuration created"
else
    echo "✅ Configuration already exists"
fi

# Create .env.example if it doesn't exist
if [ ! -f ".env.example" ]; then
    echo "📝 Creating .env.example..."
    python3 -c "from config import Config; Config().save_example()"
    echo "✅ .env.example created"
fi

# Check for .gate_cookies.json
if [ ! -f ".gate_cookies.json" ]; then
    echo "⚠️  Warning: .gate_cookies.json not found"
    echo "   Please create this file with your Gate.io cookies"
    echo "   Format: [{\"name\": \"sid\", \"value\": \"...\", ...}]"
else
    echo "✅ Gate.io cookies file found"
fi

# Create logs directory
mkdir -p logs

# Make scripts executable
chmod +x setup.sh run.sh

echo ""
echo "✅ Setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Copy .env.example to .env and fill in your credentials"
echo "2. Ensure .gate_cookies.json exists with your Gate.io cookies"
echo "3. Run the trader: ./run.sh"
echo ""
echo "For manual mode: ./run.sh --mode manual"
echo "For automatic mode: ./run.sh --mode automatic"