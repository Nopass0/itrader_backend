# iTrader Auto-Trader Demo Quick Start

Due to compilation issues with the full system, a Python demo is available to showcase the confirmation system.

## Running the Demo

### Manual Mode (Default)
```bash
# Using the start script
./start.sh --demo

# Or directly with Python
python3 demo.py
```

### Automatic Mode
```bash
# Using the start script
./start.sh --auto --demo

# Or directly with Python
python3 demo.py --auto
```

## What the Demo Shows

The demo simulates the auto-trader's confirmation system:

1. **Transaction Detection**: Every 3 cycles, a new transaction is "found"
2. **Manual Confirmations**: In manual mode, you'll see confirmation prompts
3. **Auto Processing**: In auto mode, transactions are processed automatically

### Example Manual Mode Interaction

```
================================================================================
⚠️  ACTION REQUIRED: Create Virtual Transaction
================================================================================

📋 Details:
  Transaction ID: DEMO-1003
  Amount: 75000.00 RUB
  Phone: +7900******67
  Bank: Tinkoff
  Action: Accept and create Bybit ad

❓ Do you want to proceed with this action?
   Enter your choice (yes/no): yes
✅ Action confirmed!
```

## Confirmation Responses

The system accepts:
- **To Confirm**: `yes`, `y`, `да`
- **To Cancel**: `no`, `n`, `нет`

## Features Demonstrated

1. **Color-coded Output**: 
   - 🟢 Green: Success/Manual mode
   - 🔴 Red: Errors/Auto mode warnings
   - 🟡 Yellow: Warnings/Actions required
   - 🔵 Blue: Info/System messages

2. **Detailed Information**: Each action shows all relevant data

3. **Multi-step Confirmations**: After accepting a transaction, you'll also confirm the Bybit ad creation

4. **Safety First**: Auto mode requires explicit confirmation before starting

## Full System

Once dependencies are resolved, the full system can be run with:
```bash
cargo run --bin itrader-backend        # Manual mode
cargo run --bin itrader-backend --auto # Auto mode
```

The demo provides the same user experience as the full system's confirmation dialogs.