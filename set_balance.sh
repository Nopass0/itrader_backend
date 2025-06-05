#!/bin/bash

# Check if amount is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <amount>"
    echo "Example: $0 500000"
    echo "         $0 1000000.50"
    exit 1
fi

# Build and run the balance setter
cargo run --bin gate_set_balance -- "$1"