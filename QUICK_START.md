# P2P Trading System - Quick Start Guide

## 🚀 Quick Setup

### 1. Install Dependencies
```bash
# Install UV if not already installed
curl -LsSf https://astral.sh/uv/install.sh | sh

# Create virtual environment and install dependencies
uv venv
source .venv/bin/activate
uv pip install -r requirements.txt
```

### 2. Setup Database
```bash
# Create PostgreSQL database
sudo -u postgres createdb p2p_trading

# Run migrations
for migration in migrations/*.sql; do
    psql -U postgres -d p2p_trading -f "$migration"
done
```

### 3. Start the System

#### Method 1: Using the startup script
```bash
./p2p_system.sh
```

#### Method 2: Direct Python
```bash
python start_p2p.py
```

#### Method 3: Quick CLI
```bash
./p2p auto    # Start automatic mode
./p2p manual  # Start manual mode
./p2p test    # Test mode
```

## 📋 First Time Setup

1. **Setup Gmail Authentication**
   - Select option 1 from main menu
   - Provide credentials.json from Google Cloud Console
   - Follow OAuth2 flow

2. **Add Accounts**
   - Select option 2 from main menu
   - Add at least one Gate.io account
   - Add at least one Bybit account

3. **Start Monitoring**
   - Option 3: Manual mode (recommended for first time)
   - Option 4: Automatic mode (fully automated)
   - Option 5: Test mode (single transaction)

## 🔧 Environment Variables

```bash
# Database connection (optional, defaults to local)
export DATABASE_URL="postgresql://user:password@localhost:5432/p2p_trading"

# PostgreSQL password for migrations
export PGPASSWORD="your_postgres_password"
```

## 🛠️ Troubleshooting

### Database Connection Failed
```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Create database manually
sudo -u postgres psql -c "CREATE DATABASE p2p_trading;"

# Grant permissions
sudo -u postgres psql -c "GRANT ALL ON DATABASE p2p_trading TO postgres;"
```

### Missing Dependencies
```bash
# Install all Python dependencies
uv pip install -r requirements.txt

# Install system dependencies for OCR
sudo apt-get install tesseract-ocr tesseract-ocr-rus
```

### Permission Denied
```bash
# Make scripts executable
chmod +x p2p_system.sh p2p start_p2p.py
```

## 📊 Features

- ✅ Gate.io transaction monitoring
- ✅ Bybit P2P ad creation
- ✅ Smart payment method alternation
- ✅ Automated chat bot
- ✅ Receipt OCR processing
- ✅ Gmail integration
- ✅ Multi-account support
- ✅ Manual and automatic modes
- ✅ Complete transaction tracking

## 🎯 Usage Examples

### Monitor Single Transaction (Test Mode)
```bash
./p2p test
```

### Start Full Automation
```bash
./p2p auto
```

### View Transaction History
```bash
./p2p history
```

### Manage Accounts
```bash
./p2p accounts
```