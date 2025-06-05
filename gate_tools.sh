#!/bin/bash
#
# Gate.io Tools Helper Script
# This script provides easy access to Gate.io utilities without admin_token errors
#

# Set up environment
export LD_LIBRARY_PATH="/home/user/.local/share/uv/python/cpython-3.11.12-linux-x86_64-gnu/lib:$LD_LIBRARY_PATH"
export APP__admin_token="${APP__admin_token:-dev-token}"

# Function to show usage
show_usage() {
    echo "Gate.io Tools - Helper script for Gate.io operations"
    echo ""
    echo "Usage: $0 <command> [args]"
    echo ""
    echo "Commands:"
    echo "  pending               - List pending transactions (status 5)"
    echo "  search <id>           - Search for a specific transaction"
    echo "  approve <id> <pdf>    - Approve transaction with receipt"
    echo "  balance <amount>      - Set account balance"
    echo "  login                 - Authenticate with Gate.io"
    echo "  available             - List all available transactions"
    echo ""
    echo "Examples:"
    echo "  $0 pending"
    echo "  $0 search 2450530"
    echo "  $0 approve 2491002 receipt.pdf"
    echo "  $0 balance 500000"
    echo ""
    echo "Environment variables:"
    echo "  COOKIE_FILE - Cookie file to use (default: .gate_cookies.json)"
    echo "  GATE_API_URL - Gate API URL (default: https://panel.gate.cx/api/v1)"
}

# Parse command
case "$1" in
    pending)
        echo "Listing pending transactions..."
        cargo run --bin gate_list_pending
        ;;
    search)
        if [ -z "$2" ]; then
            echo "Error: Transaction ID required"
            echo "Usage: $0 search <transaction_id>"
            exit 1
        fi
        echo "Searching for transaction $2..."
        cargo run --bin gate_search_transaction "$2"
        ;;
    approve)
        if [ -z "$2" ] || [ -z "$3" ]; then
            echo "Error: Transaction ID and receipt path required"
            echo "Usage: $0 approve <transaction_id> <receipt_path>"
            exit 1
        fi
        echo "Approving transaction $2 with receipt $3..."
        cargo run --bin gate_approve_transaction -- --transaction-id "$2" --receipt-path "$3"
        ;;
    balance)
        if [ -z "$2" ]; then
            echo "Setting default balance (100000 RUB)..."
            cargo run --bin gate_set_balance
        else
            echo "Setting balance to $2 RUB..."
            BALANCE_AMOUNT="$2" cargo run --bin gate_set_balance
        fi
        ;;
    login)
        echo "Authenticating with Gate.io..."
        cargo test test_gate_login_with_credentials -- --nocapture
        ;;
    available)
        echo "Listing all available transactions..."
        cargo run --bin gate_list_available
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        if [ -n "$1" ]; then
            echo "Unknown command: $1"
            echo ""
        fi
        show_usage
        exit 1
        ;;
esac