#!/bin/bash

# Check if transaction ID is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <transaction_id>"
    echo "Example: $0 2450530"
    exit 1
fi

# Build and run the transaction search
cargo run --bin gate_search_transaction -- "$1"