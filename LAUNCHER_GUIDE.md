# P2P Trading System - Launcher Guide

## 🚀 Quick Start

### Method 1: Using the startup script (Recommended)
```bash
./p2p_system.sh
```

### Method 2: Using the CLI
```bash
./p2p            # Start with menu
./p2p auto       # Start automatic mode
./p2p manual     # Start manual mode
./p2p test       # Test with single transaction
```

### Method 3: Direct Python
```bash
python src/main_launcher.py
```

## 📋 System Requirements

Before starting, ensure you have:
1. ✅ PostgreSQL database running
2. ✅ Python 3.8+ installed
3. ✅ UV package manager (will be installed automatically)

## 🎯 First Time Setup

1. **Run the launcher:**
   ```bash
   ./p2p_system.sh
   ```

2. **Setup Gmail (Option 1):**
   - Download credentials.json from Google Cloud Console
   - Provide path when prompted
   - Authenticate with your Gmail account

3. **Add Accounts (Option 2):**
   - Add at least one Gate.io account
   - Add at least one Bybit account with P2P API access

4. **Start Monitoring:**
   - Option 3: Manual mode (with confirmations)
   - Option 4: Automatic mode (fully automated)
   - Option 5: Test mode (single transaction)

## 🔧 Operating Modes

### Manual Mode
- Every action requires confirmation
- You can review all decisions
- Safe for learning and testing
- Recommended for first-time users

### Automatic Mode
- Fully automated operation
- Creates ads automatically
- Sends chat messages without confirmation
- Processes receipts and releases funds
- Use only when confident with the system

### Test Mode
- Processes only ONE transaction
- Shows complete workflow
- Stops after completion
- Perfect for testing setup

## 🛠️ CLI Commands

```bash
# Main commands
./p2p start      # Start with interactive menu
./p2p auto       # Start automatic monitoring
./p2p manual     # Start manual monitoring
./p2p test       # Run test mode

# Management commands
./p2p accounts   # Manage Gate/Bybit accounts
./p2p gmail      # Setup Gmail authentication
./p2p history    # View transaction history
./p2p help       # Show help message
```

## 📊 System Status Indicators

When you start the launcher, it shows:
- ✅ Green checkmark = Component ready
- ⚠️  Yellow warning = Component needs setup
- ❌ Red X = Component not configured

## 🔍 Monitoring Behavior

The system monitors:
- **Gate.io**: Every 5 minutes for pending transactions
- **Gmail**: Every 30 seconds for new receipts
- **Bybit P2P**: Every 30 seconds for new orders
- **Release scheduler**: Every 30 seconds for approved transactions

## 🛡️ Safety Features

1. **Manual Mode Confirmations:**
   - Confirm before creating ads
   - Review chat messages before sending
   - Approve requisites before sharing

2. **Transaction States:**
   - All states tracked in database
   - Complete audit trail
   - Error handling at each step

3. **Fool Pool Protection:**
   - Non-compliant buyers moved to fool pool
   - Prevents asset loss
   - Tracks problematic transactions

## 🚨 Troubleshooting

### "Gmail not configured"
1. Go to https://console.cloud.google.com
2. Enable Gmail API
3. Create OAuth credentials
4. Download credentials.json
5. Run: `./p2p gmail`

### "No active accounts"
1. Run: `./p2p accounts`
2. Add Gate.io account (option 1)
3. Add Bybit account (option 2)
4. Test connections (option 7)

### "Database not found"
1. Create database: `createdb p2p_trading`
2. Run migrations: `./p2p_system.sh` (will auto-migrate)

### Service Installation (Optional)
```bash
# Install as systemd service
sudo cp p2p-trading.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable p2p-trading
sudo systemctl start p2p-trading
```

## 📞 Support

For issues or questions:
1. Check transaction history: `./p2p history`
2. Review logs in `logs/` directory
3. Check database state
4. Restart in manual mode for debugging