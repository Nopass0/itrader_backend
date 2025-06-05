# Gate.io Integration Usage Guide

## Current Status (June 2025)

⚠️ **Important**: The Gate.io API endpoints for fetching transactions are currently returning `410 Gone`. Only the approval endpoint is working.

## Working Features

### 1. Login and Authentication
```bash
./test.sh gate-login
```
- Authenticates with Gate.io using credentials
- Saves cookies to `.gate_cookies.json`
- Required before using other features

### 2. Direct Transaction Approval
```bash
./approve_direct.sh <TRANSACTION_ID> <RECEIPT_PDF>
# Example:
./approve_direct.sh 2518352 test_data/receipt.pdf
```
- Approves a transaction directly without fetching its details
- Parses and displays PDF receipt information
- Asks for confirmation before approval
- Uploads the PDF receipt with the approval request

### 3. Transaction Approval with Verification (Limited)
```bash
./approve_transaction.sh <TRANSACTION_ID> <RECEIPT_PDF>
# Example:
./approve_transaction.sh 2518352 test_data/receipt.pdf
```
- Attempts to fetch transaction details first (currently fails due to 410)
- Falls back to direct approval if needed

## Non-Working Features (API Returns 410)

### 1. List Pending Transactions
```bash
./test.sh gate-pending
```
- Should list transactions with status 5 (pending approval)
- Currently fails with 410 Gone error

### 2. Search Transaction by ID
```bash
./test.sh gate-search <TRANSACTION_ID>
```
- Should search for a specific transaction
- Currently fails with 410 Gone error

### 3. Get Available Transactions
```bash
./test.sh gate-tx
```
- Should list available transactions
- Currently fails with 410 Gone error

## Workflow

Since the API is not working for fetching transactions, the current workflow is:

1. **Get Transaction IDs Manually**
   - Log into Gate.io web interface
   - Navigate to pending transactions
   - Copy the transaction ID

2. **Prepare PDF Receipt**
   - Make sure you have the PDF receipt for the transaction
   - The receipt should contain matching:
     - Amount
     - Bank name
     - Phone number or card number

3. **Approve Transaction**
   ```bash
   ./approve_direct.sh <TRANSACTION_ID> <RECEIPT_PDF>
   ```

4. **Verify Approval**
   - Check the web interface to confirm the transaction was approved
   - The tool will show the new status after approval

## Technical Details

### Bank Normalization
The system includes comprehensive bank name normalization supporting:
- Official bank names
- Common abbreviations
- English transliterations
- Over 100 Russian banks

### Phone Number Normalization
- Handles +7 and 8 prefixes for Russian numbers
- Removes formatting characters
- Ensures consistent format for comparison

### PDF Parsing
The receipt parser extracts:
- Transaction amount
- Date and time
- Bank name
- Phone number
- Card number (if available)
- Recipient information
- Transaction status

## Troubleshooting

### "Cookie file not found" Error
Run the login command first:
```bash
./test.sh gate-login
```

### "410 Gone" Errors
This indicates the API endpoint has been removed. Use the direct approval method instead of trying to fetch transaction details.

### PDF Parsing Issues
Ensure the PDF:
- Is not corrupted
- Contains text (not just images)
- Is a valid bank receipt

## Future Improvements

When/if the API becomes available again:
- Automatic transaction fetching
- Bulk approval functionality
- Transaction status monitoring
- Automated receipt matching