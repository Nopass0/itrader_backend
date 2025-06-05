#!/bin/bash

echo "=== iTrader Backend Test Suite ==="
echo

# Set up Python library path for PyO3
export LD_LIBRARY_PATH="/home/user/.local/share/uv/python/cpython-3.11.12-linux-x86_64-gnu/lib:$LD_LIBRARY_PATH"

# Check for testnet flag
USE_TESTNET=false
if [[ "$*" == *"--testnet"* ]]; then
    USE_TESTNET=true
    echo "Using Bybit testnet environment"
fi

# Check if .env.test exists, if not create from .env
if [ ! -f .env.test ]; then
    if [ -f .env ]; then
        cp .env .env.test
        echo "Created .env.test from .env"
    else
        echo "Creating .env.test with default values..."
        cat > .env.test << EOF
DATABASE_URL=postgresql://postgres:root@localhost/itrader_test_db
DATABASE_MAX_CONNECTIONS=5
DATABASE_MIN_CONNECTIONS=1
REDIS_URL=redis://localhost:6379/1
REDIS_POOL_SIZE=5
ENCRYPTION_KEY=test-encryption-key-32-bytes-long
OPENROUTER_API_KEY=test-api-key
JWT_SECRET=test-jwt-secret
EMAIL_ADDRESS=test@example.com
EMAIL_PASSWORD=test-password
EOF
        echo "Created .env.test with default values"
    fi
fi

# Set DATABASE_URL for SQLx compile-time checks (if needed)
export DATABASE_URL="postgresql://postgres:root@localhost/itrader_test_db"

# Set Bybit URL and credentials file based on testnet flag
if [ "$USE_TESTNET" = true ]; then
    export APP__BYBIT__REST_URL="https://api-testnet.bybit.com"
    export APP__BYBIT__WS_URL="wss://stream-testnet.bybit.com"
    export BYBIT_CREDENTIALS_FILE="test_data/bybit_testnet_creditials.json"
else
    export APP__BYBIT__REST_URL="https://api.bybit.com"
    export APP__BYBIT__WS_URL="wss://stream.bybit.com"
    export BYBIT_CREDENTIALS_FILE="test_data/bybit_creditials.json"
fi

# Function to run specific test
run_test() {
    local test_name=$1
    echo "Running $test_name..."
    echo "----------------------------------------"
    cargo test $test_name -- --nocapture --test-threads=1
    echo
}

# Parse command line arguments (remove --testnet from args)
TEST_ARG=${1/--testnet/}
TEST_ARG=${TEST_ARG## }

case "$TEST_ARG" in
    gate-login)
        run_test "test_gate_login_with_credentials"
        ;;
    gate-auth)
        run_test "test_gate_auth_with_cookies"
        ;;
    gate-tx)
        run_test "test_gate_get_transactions"
        ;;
    gate-search)
        # Check if transaction ID is provided as second argument
        if [ -n "$2" ]; then
            export TRANSACTION_ID="$2"
            echo "Using transaction ID: $TRANSACTION_ID"
        fi
        run_test "test_gate_search_transaction"
        ;;
    gate-service)
        echo "Running Gate.io transaction service tests..."
        echo "----------------------------------------"
        cargo test gate_transaction_service_tests -- --ignored --nocapture --test-threads=1
        ;;
    gate-balance)
        # Check if balance amount is provided as second argument
        if [ -n "$2" ]; then
            export BALANCE_AMOUNT="$2"
            echo "Setting balance to: $BALANCE_AMOUNT RUB"
        fi
        run_test "test_gate_set_balance"
        ;;
    gate-all)
        echo "Running all Gate.io tests..."
        run_test "gate_tests"
        ;;
    gate-approve)
        if [ -z "$2" ] || [ -z "$3" ]; then
            echo "Error: Transaction ID and receipt path required"
            echo "Usage: $0 gate-approve <TRANSACTION_ID> <RECEIPT_PATH>"
            echo "Example: $0 gate-approve 2491002 test_data/receipt.pdf"
            exit 1
        fi
        echo "Testing Gate.io transaction approval..."
        echo "Transaction ID: $2"
        echo "Receipt path: $3"
        echo "----------------------------------------"
        cargo run --bin gate_approve_transaction -- --transaction-id "$2" --receipt-path "$3" --cookie-file .gate_cookies.json
        ;;
    gate-pending)
        echo "Listing pending Gate.io transactions..."
        echo "----------------------------------------"
        cargo run --bin gate_list_pending
        ;;
    gate-approve-direct)
        if [ -z "$2" ] || [ -z "$3" ]; then
            echo "Error: Transaction ID and receipt path required"
            echo "Usage: $0 gate-approve-direct <TRANSACTION_ID> <RECEIPT_PATH>"
            echo "Example: $0 gate-approve-direct 2518352 test_data/receipt.pdf"
            exit 1
        fi
        echo "Testing Gate.io direct transaction approval..."
        echo "Transaction ID: $2"
        echo "Receipt path: $3"
        echo "----------------------------------------"
        cargo run --bin gate_approve_transaction_direct -- --transaction-id "$2" --receipt-path "$3" --cookie-file .gate_cookies.json
        ;;
    gate-test-proxy)
        echo "Testing Gate.io API with proxy..."
        echo "----------------------------------------"
        cargo run --bin gate_test_with_proxy
        ;;
    gate-test-endpoints)
        echo "Testing all Gate.io API endpoint variations..."
        echo "----------------------------------------"
        cargo run --bin gate_test_all_endpoints
        ;;
    bybit-auth)
        run_test "test_bybit_auth"
        ;;
    bybit-ads)
        run_test "test_bybit_get_advertisements"
        ;;
    bybit-all-ads)
        run_test "test_bybit_get_all_advertisements"
        ;;
    bybit-active-ads)
        run_test "test_bybit_get_active_advertisements"
        ;;
    bybit-chats)
        run_test "test_bybit_get_advertisement_chats"
        ;;
    bybit-available)
        run_test "test_bybit_check_availability"
        ;;
    bybit-orders)
        run_test "test_bybit_get_orders"
        ;;
    bybit-rates)
        # Check if amount is provided as second argument
        if [ -n "$2" ]; then
            export TEST_AMOUNT="$2"
            echo "Running Bybit P2P rate fetcher tests with amount: $2 RUB"
        else
                    echo "Running Bybit P2P rate fetcher tests..."
        fi
        echo "----------------------------------------"
        cargo test bybit_rate_tests -- --nocapture --test-threads=1
        ;;
    bybit-rates-python)
        echo "Running Bybit P2P rate fetcher tests (Python SDK)"
        if [ -n "$2" ] && [ "$2" != "--testnet" ]; then
            echo " with amount: $2 RUB"
            cargo run --bin bybit_check_rate_python -- "$2"
        else
            echo " with default amount (50000 RUB)"
            cargo run --bin bybit_check_rate_python -- 50000
        fi
        ;;
    bybit-all)
        echo "Running all Bybit tests..."
        run_test "bybit_tests"
        ;;
    gmail-auth)
        echo "Running interactive Gmail OAuth2 authentication..."
        echo "----------------------------------------"
        cargo test test_gmail_auth -- --ignored --nocapture --test-threads=1
        ;;
    gmail-list-today)
        echo "Running Gmail list emails from today..."
        echo "----------------------------------------"
        cargo test test_gmail_list_emails_today -- --ignored --nocapture --test-threads=1
        ;;
    gmail-list-sender)
        echo "Running Gmail list emails from sender (interactive)..."
        echo "----------------------------------------"
        cargo test test_gmail_list_emails_from_sender -- --ignored --nocapture --test-threads=1
        ;;
    gmail-latest)
        echo "Running Gmail get latest email..."
        echo "----------------------------------------"
        cargo test test_gmail_get_latest_email -- --ignored --nocapture --test-threads=1
        ;;
    gmail-latest-pdf)
        echo "Running Gmail get latest PDF..."
        echo "----------------------------------------"
        cargo test test_gmail_get_latest_pdf -- --ignored --nocapture --test-threads=1
        ;;
    gmail-pdf-sender)
        echo "Running Gmail download PDF from sender (interactive)..."
        echo "----------------------------------------"
        cargo test test_gmail_download_pdf_from_sender -- --ignored --nocapture --test-threads=1
        ;;
    gmail-sender-info)
        echo "Running Gmail get sender info..."
        echo "----------------------------------------"
        cargo test test_gmail_get_sender_info -- --ignored --nocapture --test-threads=1
        ;;
    gmail-all)
        echo "Running all Gmail tests..."
        echo "----------------------------------------"
        cargo test gmail_tests -- --ignored --nocapture --test-threads=1
        ;;
    receipt-parse)
        # Check if receipt file is provided as second argument
        if [ -n "$2" ]; then
            export RECEIPT_FILE="$2"
            echo "Using receipt file: $RECEIPT_FILE"
        fi
        run_test "test_pdf_receipt_parser"
        ;;
    receipt-compare)
        # Check if transaction ID and receipt file are provided
        if [ -n "$2" ] && [ -n "$3" ]; then
            export TRANSACTION_ID="$2"
            export RECEIPT_FILE="$3"
            echo "Comparing transaction $TRANSACTION_ID with receipt $RECEIPT_FILE"
            echo "----------------------------------------"
            # Run only the specific test from transaction_receipt_comparison_test.rs
            cargo test --test transaction_receipt_comparison_test test_compare_transaction_with_receipt -- --exact --nocapture --test-threads=1
        else
            echo "Error: Both transaction ID and receipt file are required"
            echo "Usage: $0 receipt-compare <transaction_id> <receipt_file>"
            exit 1
        fi
        ;;
    receipt-compare-mock)
        echo "Running mock transaction-receipt comparison tests..."
        echo "----------------------------------------"
        cargo test transaction_receipt_comparison_mock_test -- --nocapture --test-threads=1
        ;;
    history)
        echo "Testing history transactions with receipts"
        echo "----------------------------------------"
        cargo test --test history_transactions_test test_history_transactions_with_receipts -- --exact --nocapture --test-threads=1
        ;;
    all)
        echo "Running all tests..."
        cargo test -- --nocapture --test-threads=1
        ;;
    *)
        echo "Usage: $0 [test-name] [--testnet]"
        echo
        echo "Available tests:"
        echo "  Gate.io tests:"
        echo "    gate-login    - Test Gate.io login with credentials and save cookies"
        echo "    gate-auth     - Test Gate.io authentication with cookies"
        echo "    gate-tx       - Test getting Gate.io transactions"
        echo "    gate-search [ID] - Test searching Gate.io transaction by ID (default: 2450530)"
        echo "                      Usage: ./test.sh gate-search 123456"
        echo "                      Or use: ./search_transaction.sh <ID>"
        echo "    gate-service  - Test Gate.io transaction service with caching"
        echo "    gate-balance [AMOUNT] - Test setting Gate.io balance (default: 100000)"
        echo "                           Usage: ./test.sh gate-balance 500000"
        echo "    gate-approve ID RECEIPT - Approve transaction with receipt verification"
        echo "                           Usage: ./test.sh gate-approve 2491002 test_data/receipt.pdf"
        echo "                           Status 5 required, verifies amount/bank/phone match"
        echo "    gate-approve-direct ID RECEIPT - Approve transaction directly (no status check)"
        echo "                           Usage: ./test.sh gate-approve-direct 2518352 test_data/receipt.pdf"
        echo "    gate-pending  - List pending transactions (status 5) ready for approval"
        echo "    gate-all      - Run all Gate.io tests"
        echo
        echo "  Bybit tests:"
        echo "    bybit-auth        - Test Bybit authentication"
        echo "    bybit-ads         - Test getting Bybit advertisements"
        echo "    bybit-all-ads     - Test getting all Bybit advertisements (active/inactive/hidden)"
        echo "    bybit-active-ads  - Test getting only active Bybit advertisements"
        echo "    bybit-chats       - Test getting advertisement chats"
        echo "    bybit-available   - Test checking Bybit account availability"
        echo "    bybit-orders      - Test getting Bybit P2P orders"
        echo "    bybit-rates [AMOUNT] - Test Bybit P2P rate fetcher (default: all amounts)"
        echo "                           Usage: ./test.sh bybit-rates 75000"
        echo "    bybit-rates-python [AMOUNT] - Test Bybit P2P rate fetcher using Python SDK"
        echo "                           Usage: ./test.sh bybit-rates-python 75000"
        echo "    bybit-all         - Run all Bybit tests"
        echo
        echo "  Gmail tests:"
        echo "    gmail-auth        - Test Gmail OAuth2 authentication"
        echo "    gmail-list-today  - Test listing emails from today"
        echo "    gmail-list-sender - Test listing emails from specific sender"
        echo "    gmail-latest      - Test getting latest email"
        echo "    gmail-latest-pdf  - Test getting latest email with PDF"
        echo "    gmail-pdf-sender  - Test downloading PDFs from specific sender"
        echo "    gmail-sender-info - Test getting sender information"
        echo "    gmail-all         - Run all Gmail tests"
        echo
        echo "  Receipt tests:"
        echo "    receipt-parse [FILE]       - Test parsing PDF receipt"
        echo "                                Usage: ./test.sh receipt-parse test_data/receipt_27.05.2025.pdf"
        echo "    receipt-compare ID FILE    - Compare transaction with receipt (requires Gate.io auth)"
        echo "                                Usage: ./test.sh receipt-compare 2463024 test_data/receipt_27.05.2025.pdf"
        echo "                                Or use: ./compare_transaction_receipt.sh <ID> <FILE>"
        echo "    receipt-compare-mock       - Run mock transaction-receipt comparison tests"
        echo "                                Usage: ./test.sh receipt-compare-mock"
        echo "    history                    - Compare all history transactions with their PDF receipts"
        echo "                                Usage: ./test.sh history"
        echo
        echo "  all - Run all tests"
        echo
        echo "Options:"
        echo "  --testnet     - Use Bybit testnet environment (https://api-testnet.bybit.com)"
        echo
        echo "Examples:"
        echo "  $0 gate-auth                # Test Gate.io authentication only"
        echo "  $0 bybit-ads                # Test getting Bybit advertisements (mainnet)"
        echo "  $0 bybit-ads --testnet      # Test getting Bybit advertisements (testnet)"
        echo "  $0 all                      # Run all tests"
        exit 1
        ;;
esac