#!/bin/bash

# Run iTrader Backend with Python libraries

# Set Python library path
export LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu:/usr/lib/python3.11/config-3.11-x86_64-linux-gnu:/usr/local/lib:$LD_LIBRARY_PATH"

# Find Python installation
PYTHON_VERSION=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
PYTHON_LIB_PATH=$(python3 -c 'import sys; import os; print(os.path.dirname(sys.executable) + "/../lib")')

# Add Python library path
export LD_LIBRARY_PATH="$PYTHON_LIB_PATH:$LD_LIBRARY_PATH"

echo "Starting iTrader Backend with Python $PYTHON_VERSION support..."
echo "Library path: $LD_LIBRARY_PATH"
echo

# Check if running in auto mode
if [ "$1" == "--auto" ]; then
    echo "🤖 Running in AUTOMATIC mode - all actions will be auto-confirmed!"
    echo "⚠️  Transactions will be processed WITHOUT manual confirmation!"
    cargo run --bin itrader-backend -- --auto
else
    echo "👤 Running in MANUAL mode - actions require confirmation"
    echo "💡 To run in automatic mode, use: ./run_with_python.sh --auto"
    cargo run --bin itrader-backend
fi