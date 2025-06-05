#!/bin/bash

echo "=== iTrader Backend Test Suite ==="
echo

# Check if specific test requested
if [ "$1" == "p2p" ]; then
    echo "🚀 Running P2P Ad Creation Test..."
    # Export test database URL for Python script
    export DATABASE_URL="postgresql://postgres:root@localhost/itrader_test"
    .venv/bin/python tests/test_bybit_p2p_ad.py
    exit 0
fi

# Simple Python bridge tests
if [ "$1" == "bybit-simple" ]; then
    echo "🚀 Running simple Bybit Python bridge tests..."
    .venv/bin/python test_bybit_python_simple.py
    exit 0
fi

# Set up UV Python environment
if [ -d ".venv" ]; then
    source .venv/bin/activate
    # Set up Python library path for PyO3 from virtual environment
    PYTHON_LIB=$(.venv/bin/python -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))' 2>/dev/null)
    if [ -n "$PYTHON_LIB" ]; then
        export LD_LIBRARY_PATH="$PYTHON_LIB:$LD_LIBRARY_PATH"
    fi
    export LD_LIBRARY_PATH="$(pwd)/.venv/lib:$LD_LIBRARY_PATH"
else
    echo "⚠️  Virtual environment not found. Run './run.sh' first to set up the environment."
    exit 1
fi

# Parse flags
SAFE_ONLY=false
USE_TESTNET=false
VERBOSE=false
ARGS=()

for arg in "$@"; do
    case $arg in
        --safe)
            SAFE_ONLY=true
            echo "🛡️  Running SAFE tests only (no data modification)"
            ;;
        --testnet)
            USE_TESTNET=true
            echo "🌐 Using Bybit testnet environment"
            ;;
        --verbose)
            VERBOSE=true
            ;;
        *)
            ARGS+=("$arg")
            ;;
    esac
done

# Check if .env.test exists, if not create from .env
if [ ! -f .env.test ]; then
    if [ -f .env ]; then
        cp .env .env.test
        echo "Created .env.test from .env"
    else
        echo "Creating .env.test with default values..."
        cat > .env.test << EOF
DATABASE_URL=postgresql://postgres:root@localhost/itrader_test
DATABASE_MAX_CONNECTIONS=5
DATABASE_MIN_CONNECTIONS=1
REDIS_URL=redis://localhost:6379/1
REDIS_POOL_SIZE=5
OPENROUTER_API_KEY=test-api-key
JWT_SECRET=test-jwt-secret
EMAIL_ADDRESS=test@example.com
EMAIL_PASSWORD=test-password
EOF
        echo "Created .env.test with default values"
    fi
fi

# Set DATABASE_URL for SQLx compile-time checks
export DATABASE_URL="postgresql://postgres:root@localhost/itrader_test"

# Set Bybit URL based on testnet flag
if [ "$USE_TESTNET" = true ]; then
    export APP__BYBIT__REST_URL="https://api-testnet.bybit.com"
    export APP__BYBIT__WS_URL="wss://stream-testnet.bybit.com"
    export BYBIT_CREDENTIALS_FILE="test_data/bybit_testnet_creditials.json"
else
    export APP__BYBIT__REST_URL="https://api.bybit.com"
    export APP__BYBIT__WS_URL="wss://stream.bybit.com"
    export BYBIT_CREDENTIALS_FILE="test_data/bybit_creditials.json"
fi

# Categories for tests
declare -A TEST_CATEGORIES
declare -A TEST_DESCRIPTIONS

# SAFE TESTS - Read-only operations
TEST_CATEGORIES[gate-auth]="safe"
TEST_DESCRIPTIONS[gate-auth]="Test Gate.io authentication with cookies"

TEST_CATEGORIES[gate-get-tx]="safe"
TEST_DESCRIPTIONS[gate-get-tx]="Get Gate.io transactions (read-only)"

TEST_CATEGORIES[gate-search]="safe"
TEST_DESCRIPTIONS[gate-search]="Search Gate.io transaction by ID"

TEST_CATEGORIES[gate-list-pending]="safe"
TEST_DESCRIPTIONS[gate-list-pending]="List pending transactions"

TEST_CATEGORIES[bybit-python-auth]="safe"
TEST_DESCRIPTIONS[bybit-python-auth]="Test Bybit authentication (Python bridge)"

TEST_CATEGORIES[bybit-python-rates]="safe"
TEST_DESCRIPTIONS[bybit-python-rates]="Get Bybit P2P rates (Python bridge)"

TEST_CATEGORIES[bybit-python-account]="safe"
TEST_DESCRIPTIONS[bybit-python-account]="Get Bybit account info (Python bridge)"

TEST_CATEGORIES[receipt-parse]="safe"
TEST_DESCRIPTIONS[receipt-parse]="Parse PDF receipt (local file)"

TEST_CATEGORIES[receipt-compare-mock]="safe"
TEST_DESCRIPTIONS[receipt-compare-mock]="Mock transaction-receipt comparison"

TEST_CATEGORIES[orchestrator-mock]="safe"
TEST_DESCRIPTIONS[orchestrator-mock]="Test orchestrator with mock functions"

TEST_CATEGORIES[unit-tests]="safe"
TEST_DESCRIPTIONS[unit-tests]="All unit tests (no external calls)"

# DANGEROUS TESTS - Write operations
TEST_CATEGORIES[gate-login]="dangerous"
TEST_DESCRIPTIONS[gate-login]="Login to Gate.io (creates session)"

TEST_CATEGORIES[gate-set-balance]="dangerous"
TEST_DESCRIPTIONS[gate-set-balance]="Set Gate.io balance (modifies account)"

TEST_CATEGORIES[gate-approve-tx]="dangerous"
TEST_DESCRIPTIONS[gate-approve-tx]="Approve Gate.io transaction (modifies data)"

TEST_CATEGORIES[bybit-python-create-ad]="dangerous"
TEST_DESCRIPTIONS[bybit-python-create-ad]="Create Bybit ad (Python bridge)"

TEST_CATEGORIES[bybit-python-delete-ad]="dangerous"
TEST_DESCRIPTIONS[bybit-python-delete-ad]="Delete Bybit ad (Python bridge)"

TEST_CATEGORIES[bybit-python-bridge-all]="safe"
TEST_DESCRIPTIONS[bybit-python-bridge-all]="All Bybit Python bridge tests"

TEST_CATEGORIES[gmail-auth]="dangerous"
TEST_DESCRIPTIONS[gmail-auth]="Gmail OAuth2 authentication (creates token)"

TEST_CATEGORIES[integration-tests]="dangerous"
TEST_DESCRIPTIONS[integration-tests]="Full integration tests (may modify data)"

# Function to run test with safety check
run_test() {
    local test_name=$1
    local category=${TEST_CATEGORIES[$test_name]}
    local description=${TEST_DESCRIPTIONS[$test_name]}
    
    # Check if running safe-only mode
    if [ "$SAFE_ONLY" = true ] && [ "$category" = "dangerous" ]; then
        echo "⚠️  Skipping DANGEROUS test: $test_name - $description"
        return
    fi
    
    # Display test info
    if [ "$category" = "dangerous" ]; then
        echo "🔴 Running DANGEROUS test: $test_name"
    else
        echo "🟢 Running SAFE test: $test_name"
    fi
    echo "   $description"
    echo "----------------------------------------"
    
    # Run the actual test
    case $test_name in
        # Safe tests
        gate-auth)
            cargo test test_gate_auth_with_cookies -- --nocapture --test-threads=1
            ;;
        gate-get-tx)
            cargo test test_gate_get_transactions -- --nocapture --test-threads=1
            ;;
        gate-search)
            if [ -n "$2" ]; then
                export TRANSACTION_ID="$2"
            fi
            cargo test test_gate_search_transaction -- --nocapture --test-threads=1
            ;;
        gate-list-pending)
            cargo run --bin gate_list_pending
            ;;
        bybit-python-auth)
            cargo test test_bybit_p2p_python_integration -- --nocapture --test-threads=1
            ;;
        bybit-python-rates)
            cargo test test_bybit_p2p_get_rates_via_python -- --nocapture --test-threads=1
            ;;
        bybit-python-account)
            cargo test test_bybit_p2p_python_integration -- --nocapture --test-threads=1
            ;;
        bybit-python-bridge-all)
            cargo test bybit_p2p_python_test -- --nocapture --test-threads=1
            ;;
        receipt-parse)
            if [ -n "$2" ]; then
                export RECEIPT_FILE="$2"
            fi
            cargo test test_pdf_receipt_parser -- --nocapture --test-threads=1
            ;;
        receipt-compare-mock)
            cargo test transaction_receipt_comparison_mock_test -- --nocapture --test-threads=1
            ;;
        orchestrator-mock)
            echo "Running orchestrator mock tests..."
            cargo test test_orchestrator_mock -- --nocapture --test-threads=1
            ;;
        unit-tests)
            echo "Running all unit tests..."
            cargo test --lib -- --nocapture --test-threads=1
            ;;
            
        # Dangerous tests
        gate-login)
            cargo test test_gate_login_with_credentials -- --nocapture --test-threads=1
            ;;
        gate-set-balance)
            if [ -n "$2" ]; then
                export BALANCE_AMOUNT="$2"
            fi
            cargo test test_gate_set_balance -- --nocapture --test-threads=1
            ;;
        gate-approve-tx)
            if [ -n "$2" ] && [ -n "$3" ]; then
                cargo run --bin gate_approve_transaction -- --transaction-id "$2" --receipt-path "$3"
            else
                echo "Error: Transaction ID and receipt path required"
                exit 1
            fi
            ;;
        bybit-python-create-ad)
            cargo test test_bybit_p2p_create_ad_via_python -- --nocapture --test-threads=1
            ;;
        bybit-python-delete-ad)
            echo "Delete ad test not implemented yet"
            ;;
        gmail-auth)
            cargo test test_gmail_auth -- --ignored --nocapture --test-threads=1
            ;;
        integration-tests)
            cargo test --test '*' -- --nocapture --test-threads=1
            ;;
    esac
    
    echo
}

# Main test execution
TEST_ARG=${ARGS[0]:-help}

case "$TEST_ARG" in
    all)
        echo "Running all tests..."
        echo
        
        # Run safe tests first
        for test in "${!TEST_CATEGORIES[@]}"; do
            if [ "${TEST_CATEGORIES[$test]}" = "safe" ]; then
                run_test "$test"
            fi
        done
        
        # Run dangerous tests if not in safe mode
        if [ "$SAFE_ONLY" = false ]; then
            echo "=== Running DANGEROUS tests ==="
            echo
            for test in "${!TEST_CATEGORIES[@]}"; do
                if [ "${TEST_CATEGORIES[$test]}" = "dangerous" ]; then
                    run_test "$test"
                fi
            done
        fi
        ;;
        
    safe)
        echo "Running all SAFE tests only..."
        echo
        for test in "${!TEST_CATEGORIES[@]}"; do
            if [ "${TEST_CATEGORIES[$test]}" = "safe" ]; then
                run_test "$test"
            fi
        done
        ;;
        
    dangerous)
        if [ "$SAFE_ONLY" = true ]; then
            echo "❌ Cannot run dangerous tests with --safe flag"
            exit 1
        fi
        echo "Running all DANGEROUS tests..."
        echo "⚠️  WARNING: These tests may modify data!"
        echo
        for test in "${!TEST_CATEGORIES[@]}"; do
            if [ "${TEST_CATEGORIES[$test]}" = "dangerous" ]; then
                run_test "$test"
            fi
        done
        ;;
        
    list)
        echo "Available tests:"
        echo
        echo "SAFE TESTS (read-only):"
        for test in "${!TEST_CATEGORIES[@]}"; do
            if [ "${TEST_CATEGORIES[$test]}" = "safe" ]; then
                printf "  %-20s - %s\n" "$test" "${TEST_DESCRIPTIONS[$test]}"
            fi
        done | sort
        echo
        echo "DANGEROUS TESTS (modify data):"
        for test in "${!TEST_CATEGORIES[@]}"; do
            if [ "${TEST_CATEGORIES[$test]}" = "dangerous" ]; then
                printf "  %-20s - %s\n" "$test" "${TEST_DESCRIPTIONS[$test]}"
            fi
        done | sort
        echo
        ;;
        
    help|*)
        # Check if it's a valid test name
        if [ -n "${TEST_CATEGORIES[$TEST_ARG]}" ]; then
            run_test "$TEST_ARG" "${ARGS[@]:1}"
            exit 0
        fi
        
        # If not help and not a valid test, show error
        if [ "$TEST_ARG" != "help" ]; then
            echo "❌ Unknown test: $TEST_ARG"
            echo
        fi
        
        echo "Usage: $0 [command] [options] [args]"
        echo
        echo "Commands:"
        echo "  all              - Run all tests (safe and dangerous)"
        echo "  safe             - Run only safe tests"
        echo "  dangerous        - Run only dangerous tests"
        echo "  list             - List all available tests"
        echo "  <test-name>      - Run specific test"
        echo
        echo "Options:"
        echo "  --safe           - Only run safe tests (no data modification)"
        echo "  --testnet        - Use Bybit testnet environment"
        echo "  --verbose        - Enable verbose output"
        echo
        echo "Examples:"
        echo "  $0 all --safe                    # Run all safe tests only"
        echo "  $0 gate-auth                     # Run specific safe test"
        echo "  $0 gate-approve-tx 123 file.pdf  # Run dangerous test with args"
        echo "  $0 safe --testnet                # Run all safe tests on testnet"
        echo
        echo "For list of all tests: $0 list"
        exit 1
        ;;
esac