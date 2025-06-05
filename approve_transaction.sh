#!/bin/bash

# Gate.io Transaction Approval Script
# Usage: ./approve_transaction.sh <TRANSACTION_ID> <RECEIPT_PATH>

if [ $# -ne 2 ]; then
    echo "Usage: $0 <TRANSACTION_ID> <RECEIPT_PATH>"
    echo "Example: $0 2491002 test_data/receipt.pdf"
    exit 1
fi

TRANSACTION_ID="$1"
RECEIPT_PATH="$2"

# Check if receipt file exists
if [ ! -f "$RECEIPT_PATH" ]; then
    echo "Error: Receipt file not found: $RECEIPT_PATH"
    exit 1
fi

# Check if cookies file exists
if [ ! -f ".gate_cookies.json" ]; then
    echo "Error: Authentication cookies not found. Please run './test.sh gate-login' first."
    exit 1
fi

echo "Approving transaction $TRANSACTION_ID with receipt $RECEIPT_PATH..."
echo

# Run the approval tool
cargo run --bin gate_approve_transaction -- \
    --transaction-id "$TRANSACTION_ID" \
    --receipt-path "$RECEIPT_PATH" \
    --cookie-file .gate_cookies.json