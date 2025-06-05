# Auto-Trader Workflow Automation

This system automates the complete trading workflow between Gate.io and Bybit based on the test.sh functionality.

## Overview

The auto-trader workflow performs the following steps:

1. **Authenticate all Gate accounts** - Logs into each Gate.io account and saves cookies
2. **Get pending transactions** - Retrieves all transactions with status 4 and 5
3. **Accept status 4 transactions** - Marks them as "in progress" (status 5)
4. **Process status 5 transactions** - For each:
   - Get current Bybit P2P rate for the transaction amount
   - Find an available Bybit account (with < 2 active ads)
   - Create a new P2P advertisement on Bybit
   - Track the transaction-ad mapping

## Prerequisites

- Rust application compiled and working (`test.sh` must be functional)
- Python 3.7+ with virtual environment
- `db/settings.json` with Gate.io and Bybit accounts configured
- Valid credentials for all accounts

## Setup

1. First, ensure the Rust application is compiled:
   ```bash
   cargo build
   ```

2. Make sure `test.sh` is working:
   ```bash
   ./test.sh gate-auth  # Test Gate authentication
   ./test.sh bybit-rates-python 50000  # Test rate fetching
   ```

3. Configure accounts in `db/settings.json`:
   ```json
   {
     "gate_accounts": [
       {
         "id": "gate_1",
         "email": "user@example.com",
         "password": "password",
         "nickname": "Account 1"
       }
     ],
     "bybit_accounts": [
       {
         "id": "bybit_1",
         "nickname": "Trader 1",
         "api_key": "your-api-key",
         "api_secret": "your-api-secret"
       }
     ]
   }
   ```

## Usage

### Run Once

Execute the workflow once:

```bash
./start_auto_trader.sh
```

Or directly:

```bash
python3 auto_trader_workflow.py
```

### Daemon Mode

Run continuously with automatic retries:

```bash
# Run every 5 minutes (default)
./start_auto_trader.sh --daemon

# Run every 10 minutes
./start_auto_trader.sh --daemon --interval 600

# With custom config
./start_auto_trader.sh --daemon --config custom_settings.json
```

### Command Line Options

- `--daemon` - Run in continuous mode
- `--interval <seconds>` - Time between runs in daemon mode (default: 300)
- `--config <file>` - Custom config file (default: db/settings.json)
- `--help` - Show usage information

## How It Works

### 1. Authentication Phase

For each Gate.io account:
- Checks if valid cookies exist (`.gate_cookies_<id>.json`)
- If cookies are valid, uses them
- Otherwise, performs login with credentials
- Saves cookies for future use

### 2. Transaction Discovery

For each authenticated Gate account:
- Runs `./test.sh gate-pending` to get pending transactions
- Parses output to extract transaction details
- Filters for status 4 (new) and status 5 (in progress)

### 3. Transaction Processing

**Status 4 transactions:**
- Marks as accepted (would transition to status 5)
- Currently a placeholder - needs Gate API implementation

**Status 5 transactions:**
- Gets current Bybit rate using `./test.sh bybit-rates-python <amount>`
- Finds available Bybit account using `./test.sh bybit-active-ads`
- Creates P2P advertisement (placeholder - needs Bybit API implementation)
- Records transaction mapping in `active_transactions.json`

### 4. Rate Caching

- Rates are cached for 5 minutes to reduce API calls
- Cache key: `rate_<amount>`
- Automatically refreshes when expired

## Output Files

### Cookie Files
- `.gate_cookies_<account_id>.json` - Saved authentication cookies for each Gate account

### Transaction Tracking
- `active_transactions.json` - Maps Gate transactions to Bybit advertisements:
  ```json
  [
    {
      "gate_transaction_id": "123456",
      "gate_account": "user@example.com",
      "bybit_account": "Trader 1",
      "amount": 50000,
      "rate": 95.5,
      "created_at": "2024-06-04T10:30:00"
    }
  ]
  ```

## Logging

The system provides detailed logging:
- `INFO` - Normal operations
- `WARNING` - Non-critical issues (e.g., no available Bybit accounts)
- `ERROR` - Critical failures

Example output:
```
2024-06-04 10:30:00 - INFO - === Starting Auto-Trader Workflow ===
2024-06-04 10:30:00 - INFO - Loaded 2 Gate accounts and 3 Bybit accounts
2024-06-04 10:30:01 - INFO - ✅ Account user@example.com already authenticated via cookies
2024-06-04 10:30:02 - INFO - Found 3 pending transactions for user@example.com
2024-06-04 10:30:03 - INFO - Processing status 5 transaction 123456
2024-06-04 10:30:04 - INFO - Current rate: 95.5
2024-06-04 10:30:05 - INFO - ✅ Found available account: Trader 1 (1 active ads)
```

## Integration Points

The workflow uses test.sh commands:
- `gate-login` - Authenticate with credentials
- `gate-auth` - Verify authentication with cookies
- `gate-pending` - List pending transactions
- `bybit-rates-python` - Get current P2P rates
- `bybit-active-ads` - Check active advertisements

## Extending the System

To implement actual transaction acceptance and ad creation:

1. **Gate Transaction Accept**: Implement the actual API call in `accept_transaction()`
2. **Bybit Ad Creation**: Implement the actual API call in `create_bybit_ad()`
3. **Status Monitoring**: Add transaction status checking and completion handling

## Troubleshooting

### "Test script not found"
Ensure `test.sh` exists and is executable:
```bash
chmod +x test.sh
```

### "Failed to authenticate"
- Check Gate.io credentials in `db/settings.json`
- Verify network connectivity
- Try manual login: `./test.sh gate-login`

### "No available Bybit account"
- All Bybit accounts have 2+ active ads
- Add more Bybit accounts or wait for ads to complete

### Rate Limiting
The system includes delays between operations to avoid rate limits:
- 2 seconds between account authentications
- 1 second between transaction accepts
- 2 seconds between ad creations