#!/bin/bash
# Direct approval of Gate.io transaction without fetching details first

if [ $# -lt 2 ]; then
    echo "Usage: $0 <TRANSACTION_ID> <RECEIPT_PDF>"
    echo "Example: $0 2518352 test_data/receipt.pdf"
    exit 1
fi

TRANSACTION_ID=$1
RECEIPT_PATH=$2

echo "=== Direct Transaction Approval ==="
echo "Transaction ID: $TRANSACTION_ID"
echo "Receipt: $RECEIPT_PATH"
echo
echo "WARNING: This will directly approve the transaction without verification!"
echo "Make sure the transaction ID is correct."
echo

./test.sh gate-approve-direct "$TRANSACTION_ID" "$RECEIPT_PATH"