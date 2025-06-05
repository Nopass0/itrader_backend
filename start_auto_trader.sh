#\!/bin/bash
#
# Auto-trader workflow starter script
# This script runs the automated trading workflow based on test.sh functionality
#

echo "=== Auto-Trader Workflow Automation ==="
echo "This script automates the complete trading workflow:"
echo "1. Authenticate all Gate accounts and save cookies"
echo "2. Get all transactions with status 4 and 5"
echo "3. Accept all status 4 transactions"
echo "4. For status 5 transactions: get rate, find available Bybit account, create ad"
echo ""

# Set up Python environment
if [ -f ".venv/bin/activate" ]; then
    echo "Activating Python virtual environment..."
    source .venv/bin/activate
fi

# Check if db/settings.json exists
if [ \! -f "db/settings.json" ]; then
    echo "Error: db/settings.json not found\!"
    echo "Please run setup_trader.py first to configure accounts."
    exit 1
fi

# Check if test.sh exists
if [ \! -f "test.sh" ]; then
    echo "Error: test.sh not found\!"
    echo "This script depends on test.sh for Gate.io and Bybit operations."
    exit 1
fi

# Parse command line arguments
DAEMON_MODE=""
INTERVAL=""
CONFIG_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --daemon)
            DAEMON_MODE="--daemon"
            shift
            ;;
        --interval)
            INTERVAL="--interval $2"
            shift 2
            ;;
        --config)
            CONFIG_FILE="--config $2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --daemon           Run in daemon mode (continuous operation)"
            echo "  --interval <sec>   Interval between runs in daemon mode (default: 300s)"
            echo "  --config <file>    Config file path (default: db/settings.json)"
            echo "  --help             Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                      # Run once"
            echo "  $0 --daemon             # Run continuously every 5 minutes"
            echo "  $0 --daemon --interval 600  # Run every 10 minutes"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run the workflow
echo "Starting auto-trader workflow..."
python3 auto_trader_workflow.py $DAEMON_MODE $INTERVAL $CONFIG_FILE
EOF < /dev/null