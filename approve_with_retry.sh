#!/bin/bash

# Gate.io Transaction Approval with Auto-Retry Script
# Usage: ./approve_with_retry.sh <transaction_id> <receipt_path>

if [ $# -lt 2 ]; then
    echo "Usage: $0 <transaction_id> <receipt_path> [--yes]"
    echo "Examples:"
    echo "  $0 2521474 test_data/receipt.pdf"
    echo "  $0 2521474 test_data/receipt.pdf --yes"
    exit 1
fi

TRANSACTION_ID=$1
RECEIPT_PATH=$2
YES_FLAG=$3

# Build the binary
echo "Building Gate.io approval tool..."
cargo build --bin gate_approve_with_retry --release

# Run the approval tool
if [ "$YES_FLAG" = "--yes" ]; then
    ./target/release/gate_approve_with_retry \
        --transaction-id "$TRANSACTION_ID" \
        --receipt-path "$RECEIPT_PATH" \
        --yes
else
    ./target/release/gate_approve_with_retry \
        --transaction-id "$TRANSACTION_ID" \
        --receipt-path "$RECEIPT_PATH"
fi