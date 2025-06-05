#!/bin/bash

# Script to compare Gate.io transaction with receipt PDF

# Usage examples:
echo "Transaction-Receipt Comparison Tool"
echo "==================================="
echo ""
echo "Usage:"
echo "  1. As a test (using environment variables):"
echo "     TRANSACTION_ID=2463024 RECEIPT_FILE=test_data/receipt_27.05.2025.pdf cargo test test_compare_transaction_with_receipt"
echo ""
echo "  2. As a binary (with command line arguments):"
echo "     cargo run --bin compare_transaction_receipt -- --transaction-id 2463024 --receipt-file test_data/receipt_27.05.2025.pdf"
echo ""
echo "  3. Using this script (pass arguments):"
echo "     ./compare_transaction_receipt.sh 2463024 test_data/receipt_27.05.2025.pdf"
echo ""

# Check if arguments are provided
if [ $# -eq 2 ]; then
    echo "Running comparison for:"
    echo "  Transaction ID: $1"
    echo "  Receipt file: $2"
    echo ""
    
    # Run the binary with provided arguments
    cargo run --bin compare_transaction_receipt -- --transaction-id "$1" --receipt-file "$2"
else
    echo "To run comparison, provide transaction ID and receipt file:"
    echo "  ./compare_transaction_receipt.sh <transaction_id> <receipt_file>"
fi