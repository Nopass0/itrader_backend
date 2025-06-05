# Bybit Python Bridge Architecture

## Overview

All Bybit operations are now handled through Python scripts that act as a bridge between Rust and the Bybit API. This architecture ensures:

1. **Rust manages all database operations** - Only Rust interacts with PostgreSQL
2. **Python handles Bybit API calls** - Using the official pybit SDK
3. **Simple JSON communication** - Rust sends JSON to Python stdin, receives JSON from stdout

## Architecture

```
Rust Application
    ↓
[JSON Request via stdin]
    ↓
Python Script (pybit SDK)
    ↓
Bybit API
    ↓
[JSON Response via stdout]
    ↓
Rust Application
    ↓
PostgreSQL Database
```

## Python Scripts

### 1. `scripts/bybit_get_rates.py`
Gets current P2P buy/sell rates.

**Input:**
```json
{
  "amount_rub": 10000,
  "testnet": true
}
```

**Output:**
```json
{
  "success": true,
  "data": {
    "buy_rate": 98.51,
    "sell_rate": 97.49,
    "amount_rub": 10000,
    "spread": 1.02,
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### 2. `scripts/bybit_create_ad.py`
Creates a P2P advertisement.

**Input:**
```json
{
  "api_key": "your_api_key",
  "api_secret": "your_api_secret",
  "testnet": true,
  "ad_params": {
    "side": "0",
    "currency": "RUB",
    "price": "98.50",
    "quantity": "100",
    "min_amount": "1000",
    "max_amount": "50000",
    "payment_methods": ["582"],
    "remarks": "Quick trade"
  }
}
```

**Output:**
```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "adId": "AD_123456",
    "status": "online",
    "createdTime": "2024-01-01T00:00:00Z"
  }
}
```

## Testing

Run Bybit tests through Python bridge:

```bash
# Run simple Python bridge tests
./test.sh bybit-simple

# Run specific Bybit tests
./test.sh bybit-python-rates
./test.sh bybit-python-create-ad
```

## Database Integration

After successful API calls, Rust updates the database:

1. **Account Management**: Updates `bybit_accounts` table
   - Increments `active_ads` counter
   - Updates `last_used` timestamp

2. **Order Tracking**: Creates records in `orders` table
   - Links to Bybit account
   - Stores ad ID and status

## Adding New Bybit Operations

To add a new Bybit operation:

1. Create a new Python script in `scripts/`
2. Define input/output JSON format
3. Implement the API call using pybit SDK
4. Create a Rust test that calls the script
5. Update database as needed in Rust

## Security Notes

- API credentials are stored encrypted in the database
- Python scripts receive credentials via stdin (not command line args)
- All database operations remain in Rust for consistency