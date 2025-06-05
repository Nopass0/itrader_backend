#!/bin/bash

# Start Auto Trader without Gmail monitoring

echo "Starting Auto Trader without Gmail..."

# Set environment variable to disable Gmail
export DISABLE_GMAIL=true

# Run the main start script
./start_auto_trader.sh "$@"