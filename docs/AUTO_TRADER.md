# Auto-Trader System Documentation

## Overview

The auto-trader system is a comprehensive solution for automating cryptocurrency trading between Gate.io and Bybit P2P platforms. It monitors pending transactions on Gate.io, creates corresponding advertisements on Bybit, and handles the entire trade lifecycle automatically.

## Architecture

### Core Components

1. **Account Manager** (`src/core/accounts.rs`)
   - Manages Gate.io and Bybit accounts
   - Stores account credentials and status in JSON format
   - Tracks active advertisements per Bybit account
   - Provides account availability and statistics

2. **Auto-Trader** (`src/core/auto_trader.rs`)
   - Main trading loop that runs every 5 minutes
   - Processes pending transactions from Gate.io
   - Creates virtual transactions linking Gate and Bybit
   - Monitors active orders and handles chat communication
   - Processes receipt verification
   - Auto-confirms transactions when enabled

3. **Virtual Transaction System**
   - Links Gate.io transactions with Bybit P2P orders
   - Maintains state across both platforms
   - Handles order lifecycle from creation to completion

## Features

### 1. Automatic Balance Management
- Sets Gate.io account balance to 10M RUB every 4 hours
- Configurable target balance and check intervals
- Automatic balance restoration on startup

### 2. Smart Rate Calculation
The system calculates optimal rates based on:
- **Time of day** (Moscow timezone)
  - Night (0-6): +3%
  - Morning rush (7-9): +1.5%
  - Day (10-16): +2%
  - Evening rush (17-19): +1.5%
  - Evening (20-23): +2.5%
  
- **Order amount**
  - Small (≤10K RUB): +2.5%
  - Medium (10K-50K RUB): +2%
  - Large (50K-200K RUB): +1.5%
  - Very large (>200K RUB): +1%
  
- **Currency pair adjustments**
  - USDT/RUB: Standard rate
  - BTC/RUB, ETH/RUB: +0.5%
  - Other pairs: +1%

### 3. Order Management
- Maximum 4 active ads per Bybit account
- Automatic ad rotation when limits are reached
- Configurable order amount limits (min: 1K, max: 1M RUB)
- Maximum concurrent orders limit (default: 10)

### 4. Chat Automation
- Sends initial greeting in Russian and English
- Requests payment receipt when buyer marks as paid
- AI-powered chat responses (when enabled)
- Automatic language detection

### 5. Receipt Verification
- Email monitoring for payment receipts
- OCR validation of receipt details
- Automatic order completion on valid receipt
- Manual confirmation mode available

### 6. Error Handling
- Automatic retry on transient errors
- Order cancellation on buyer timeout
- Appeal handling and admin notifications
- Graceful shutdown with balance reset

## Configuration

Add to `config/default.toml`:

```toml
[auto_trader]
enabled = true
check_interval_secs = 300        # 5 minutes
balance_check_interval_hours = 4
target_balance_rub = 10000000    # 10M RUB
min_order_amount = 1000
max_order_amount = 1000000
auto_confirm = false             # Manual mode by default
max_concurrent_orders = 10
```

## Running the Auto-Trader

### Starting the System

```bash
# Start in MANUAL mode (default - safer)
# Requires confirmation for each transaction
cargo run

# Start in AUTOMATIC mode
# Auto-confirms all transactions - USE WITH CAUTION!
cargo run -- --auto

# Production builds:
cargo run --release            # Manual mode
cargo run --release -- --auto  # Automatic mode
```

### Mode Differences

**Manual Mode (Default)**
- Interactive confirmation prompts for each major action
- Displays detailed information before proceeding:
  - Transaction details (amount, phone, bank)
  - Bybit ad parameters (rate, USDT amount, payment method)
  - Balance updates (current vs new)
  - Receipt validation results
  - Order completion actions
- User must type "yes/да" or "no/нет" to confirm each action
- Any action can be cancelled by typing "no/нет"
- Safer for testing and initial setup
- Perfect for learning the system behavior

**Automatic Mode**
- All actions are automatically confirmed without prompts
- No manual intervention required
- Suitable for production after thorough testing
- ⚠️ Higher risk - ensure all configurations are correct before using

### Interactive Confirmations

In manual mode, you'll see prompts like:

```
================================================================================
⚠️  ACTION REQUIRED: Create Virtual Transaction
================================================================================

📋 Details:
  Gate Transaction ID: 2518352
  Amount: 75000.00 RUB
  Phone Number: +79001234567
  Bank: Tinkoff
  Action: Accept transaction and create Bybit ad

❓ Do you want to proceed with this action?
   Enter your choice (yes/no): _
```

The system supports both English and Russian responses:
- Confirm: `yes`, `y`, `да`
- Cancel: `no`, `n`, `нет`

## Account Management

### Using the CLI Tool

```bash
# List all accounts
cargo run --bin manage_accounts list

# Add Gate.io account
cargo run --bin manage_accounts add-gate user@example.com password123

# Add Bybit account
cargo run --bin manage_accounts add-bybit account_name API_KEY API_SECRET

# Show statistics
cargo run --bin manage_accounts stats

# Update Gate balance
cargo run --bin manage_accounts update-balance 1 5000000
```

### Account Storage

Accounts are stored in `data/accounts.json`:

```json
{
  "gate_accounts": [
    {
      "id": 1,
      "email": "user@example.com",
      "cookies": {...},
      "last_auth": "2025-03-06T10:00:00Z",
      "balance": 10000000.0,
      "status": "active"
    }
  ],
  "bybit_accounts": [
    {
      "id": 1,
      "account_name": "bybit1",
      "api_key": "...",
      "active_ads": 2,
      "status": "available"
    }
  ]
}
```

## Order Lifecycle

1. **New Transaction Detection**
   - Gate.io pending transaction detected
   - Checked against order limits
   - Available Bybit account selected

2. **Virtual Transaction Creation**
   - Transaction accepted on Gate.io
   - Rate calculated using smart pricing
   - Advertisement created on Bybit
   - Order recorded in database

3. **Buyer Matching**
   - Monitor for new P2P orders on advertisement
   - Send initial greeting message
   - Move to chat phase

4. **Payment Processing**
   - Wait for buyer to mark as paid
   - Request payment receipt
   - Monitor email for receipt

5. **Verification**
   - OCR validation of receipt
   - Amount and details verification
   - Auto-confirm if enabled

6. **Order Completion**
   - Release funds on Bybit
   - Confirm transaction on Gate.io
   - Delete advertisement
   - Update account statistics

## Monitoring

The system provides comprehensive logging:
- Account statistics on each cycle
- Transaction processing details
- Rate calculations
- Order state transitions
- Error conditions

## Security Considerations

1. **Credentials**
   - API keys stored in account manager
   - Passwords never logged or serialized
   - Environment-based encryption key

2. **Rate Limiting**
   - Respects platform rate limits
   - Built-in request throttling
   - Automatic backoff on errors

3. **Balance Protection**
   - Minimum balance thresholds
   - Automatic balance restoration
   - Shutdown balance reset to 0

## Troubleshooting

### Common Issues

1. **No available Bybit accounts**
   - Check if all accounts have reached 4 active ads
   - Use `manage_accounts stats` to view availability
   - Wait for orders to complete or add more accounts

2. **Authentication failures**
   - Verify cookies are up to date
   - Check API key validity
   - Monitor session refresh logs

3. **Rate calculation issues**
   - Verify timezone settings
   - Check rate multiplier configuration
   - Review calculation logs

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

## Future Enhancements

1. **Advanced Features**
   - Machine learning for rate optimization
   - Multi-currency support
   - Advanced fraud detection
   - Telegram/Discord notifications

2. **Scalability**
   - Redis-based state management
   - Horizontal scaling support
   - Load balancing across accounts

3. **Analytics**
   - Profitability tracking
   - Performance metrics
   - Historical data analysis
   - Real-time dashboards