# P2P Trading System - Setup Guide

## 🚀 Quick Start Options

### Option 1: Demo Mode (No Database Required)
```bash
# Run demo without PostgreSQL
python demo_mode.py
```
This mode lets you explore the system without setting up a database.

### Option 2: Setup PostgreSQL First
```bash
# 1. Install PostgreSQL
sudo apt-get install postgresql postgresql-client

# 2. Run database setup
./setup_database.sh

# 3. Start the system
./p2p_system.sh
```

### Option 3: Manual Database Setup
```bash
# 1. Create database as postgres user
sudo -u postgres psql -c "CREATE DATABASE p2p_trading;"

# 2. Run migrations
for migration in migrations/*.sql; do
    psql -U postgres -d p2p_trading -f "$migration"
done

# 3. Start system
./p2p_system.sh
```

## 🔧 Troubleshooting

### "Failed to create database"
This usually means PostgreSQL is not running or needs different credentials.

**Fix on Ubuntu/Debian:**
```bash
# Start PostgreSQL
sudo systemctl start postgresql

# Create database as postgres user
sudo -u postgres createdb p2p_trading

# Or create user for your username
sudo -u postgres createuser -s $USER
createdb p2p_trading
```

**Fix on MacOS:**
```bash
# Install PostgreSQL
brew install postgresql

# Start service
brew services start postgresql

# Create database
createdb p2p_trading
```

### "Password authentication failed"
Create a `.pgpass` file for passwordless access:
```bash
echo "localhost:5432:p2p_trading:postgres:postgres" > ~/.pgpass
chmod 600 ~/.pgpass
```

Or set environment variable:
```bash
export PGPASSWORD="your_postgres_password"
./p2p_system.sh
```

### "PostgreSQL not found"
Install PostgreSQL first:
- **Ubuntu/Debian**: `sudo apt-get install postgresql postgresql-client`
- **MacOS**: `brew install postgresql`
- **Windows**: Download from https://www.postgresql.org/download/windows/

## 📊 System Architecture

```
P2P Trading System
├── Gate.io Monitor (checks every 5 minutes)
├── Transaction Processor (queue-based)
├── Bybit P2P Ad Creator (smart alternation)
├── Chat Bot (automated responses)
├── Gmail Monitor (receipt scanning)
├── OCR Processor (PDF validation)
└── Database (PostgreSQL)
```

## 🎯 Usage Modes

### 1. Demo Mode
- No database required
- Simulates complete workflow
- Perfect for understanding the system

### 2. Manual Mode
- Requires confirmation for each action
- Safe for testing with real accounts
- Recommended for first-time users

### 3. Automatic Mode
- Fully automated operation
- No confirmations required
- Use only when confident

### 4. Test Mode
- Processes single transaction
- Shows complete workflow
- Ideal for verification

## 🛠️ Required Credentials

### Gate.io Account
- Email/Phone login
- Password
- Optional: Cookies file

### Bybit Account
- API Key with P2P permissions
- API Secret
- Must have P2P trading enabled

### Gmail Account
- OAuth2 credentials.json
- Gmail API enabled
- Access to receipt emails

## 📝 First Steps After Setup

1. **Add Gate.io Account**
   - Select option 2 → Add Gate account
   - Provide login credentials

2. **Add Bybit Account**
   - Select option 2 → Add Bybit account
   - Provide API credentials

3. **Setup Gmail**
   - Select option 1
   - Upload credentials.json
   - Complete OAuth flow

4. **Start Monitoring**
   - Option 3: Manual mode (recommended)
   - Option 4: Automatic mode
   - Option 5: Test single transaction

## 🔐 Security Notes

- API keys are stored as plain text (per request)
- Use dedicated accounts for trading
- Monitor transactions regularly
- Keep credentials secure

## 📞 Support

For issues:
1. Check `logs/` directory
2. Run in demo mode first
3. Verify all credentials
4. Check database connection