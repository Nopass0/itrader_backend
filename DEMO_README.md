# iTrader Auto-Trader Demo

## Quick Start

Since the main Rust system has compilation issues, we've created Python demos to showcase the confirmation system.

### 🎭 Non-Interactive Showcase
```bash
python3 demo_showcase.py
```
This shows what the system looks like without requiring any input.

### 🎮 Interactive Demo
```bash
# Option 1: Use the interactive menu
./run_demo.sh

# Option 2: Run directly
python3 demo.py        # Manual mode (default)
python3 demo.py --auto # Automatic mode
```

### 📦 Using start.sh
```bash
./start.sh --demo        # Manual mode demo
./start.sh --auto --demo # Auto mode demo
```

## What the Demos Show

### Manual Mode (Default)
- Requires confirmation for each action
- Shows detailed information about transactions
- Accepts yes/no in English and Russian (yes/y/да, no/n/нет)
- Color-coded output for clarity

### Automatic Mode
- Processes all actions automatically
- Requires explicit confirmation to start
- Shows what actions are being taken

## Example Confirmation Dialog

```
================================================================================
⚠️  ACTION REQUIRED: Create Virtual Transaction
================================================================================

📋 Details:
  Transaction ID: GATE-TX-12345
  Amount: 75000.00 RUB
  Phone: +7900******67
  Bank: Tinkoff
  Action: Accept transaction and create Bybit P2P ad

❓ Do you want to proceed with this action?
   Enter your choice (yes/no): 
```

## Features Demonstrated

- **Multi-language support**: Russian and English confirmations
- **Colored output**: Green for success, red for errors, yellow for warnings
- **Detailed information**: All relevant data shown before confirmation
- **Two-step confirmation**: Separate confirmations for transaction and ad creation
- **Safety first**: Default is manual mode, auto mode requires explicit opt-in

## Full System

Once the compilation issues are resolved, the full system will provide:
- Gate.io transaction monitoring
- Bybit P2P ad creation with dynamic rates
- Email monitoring for PDF receipts
- OCR validation of receipt details
- WebSocket API for real-time updates
- Database persistence
- Multi-account support

For now, use the demos to understand how the confirmation system works!