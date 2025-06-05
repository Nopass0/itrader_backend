# Manual Mode Examples

This document shows examples of interactive confirmations in manual mode.

## Starting in Manual Mode

```bash
$ ./start.sh
==================================
    iTrader Auto-Trader System    
==================================

👤 MANUAL MODE (default)
✅ All transactions will require manual confirmation
💡 This is the recommended mode for testing and initial setup

To run in automatic mode, use: ./start.sh --auto
```

## Example Confirmations

### 1. Balance Update

When the system needs to update Gate.io account balance:

```
================================================================================
⚠️  ACTION REQUIRED: Update Gate.io Balance
================================================================================

📋 Details:
  Account: user@example.com
  Current Balance: 2500000.00 RUB
  New Balance: 10000000.00 RUB
  Change: +7500000.00 RUB

❓ Do you want to proceed with this action?
   Enter your choice (yes/no): yes
✅ Action confirmed!
```

### 2. New Transaction Processing

When a new transaction is detected on Gate.io:

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
   Enter your choice (yes/no): да
✅ Action confirmed!
```

### 3. Bybit Advertisement Creation

After accepting the transaction, creating the Bybit ad:

```
================================================================================
⚠️  ACTION REQUIRED: Create Bybit P2P Advertisement
================================================================================

📋 Details:
  Bybit Account: bybit_account_1
  Amount RUB: 75000.00 RUB
  Amount USDT: 932.84 USDT
  Rate: 80.45 RUB/USDT
  Payment Method: SBP
  Ad Type: SELL USDT
  Duration: 15 minutes

❓ Do you want to proceed with this action?
   Enter your choice (yes/no): y
✅ Action confirmed!
```

### 4. Receipt Validation

When a receipt is received and validated:

```
================================================================================
⚠️  ACTION REQUIRED: Receipt Validation Result
================================================================================

📋 Details:
  Expected Amount: 75000.00 RUB
  Extracted Amount: 75000.00 RUB
  Amount Match: ✅ YES
  Expected Phone (last 4): 4567
  Extracted Phone: +79001234567
  Phone Match: ✅ YES
  Bank Match: ✅ YES

❓ Do you want to proceed with this action?
   Enter your choice (yes/no): yes
✅ Action confirmed!
```

### 5. Order Completion

Final confirmation before releasing funds:

```
================================================================================
⚠️  ACTION REQUIRED: Complete Order
================================================================================

📋 Details:
  Order ID: 42
  Gate Transaction: 2518352
  Bybit Order: ORD123456
  Amount: 75000.00 RUB
  Receipt Validation: ✅ PASSED
  Actions: 1. Release funds on Bybit
               2. Approve transaction on Gate.io

❓ Do you want to proceed with this action?
   Enter your choice (yes/no): да
✅ Action confirmed!
```

## Cancelling Actions

You can cancel any action by typing "no" or "нет":

```
❓ Do you want to proceed with this action?
   Enter your choice (yes/no): no
❌ Action cancelled!
```

## Supported Responses

The system accepts responses in both English and Russian:

**To Confirm:**
- `yes`
- `y` 
- `да`

**To Cancel:**
- `no`
- `n`
- `нет`

## Error Handling

If an error occurs, you'll be asked if you want to retry:

```
❌ ERROR OCCURRED
Action: Create Bybit Advertisement
Error: Network timeout

Retry this action? (yes/no): yes
```

## Tips for Manual Mode

1. **Read Carefully**: Always review the details before confirming
2. **Check Amounts**: Verify that amounts and rates are correct
3. **Verify Phone/Bank**: Ensure phone numbers and banks match
4. **Use for Testing**: Perfect for testing configurations before going automatic
5. **Learn the Flow**: Understand what the system does at each step

## Switching to Automatic Mode

Once you're confident with the system behavior, you can switch to automatic mode:

```bash
./start.sh --auto
```

This will bypass all confirmations and run fully automated.