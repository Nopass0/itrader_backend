#!/bin/bash

# Run the iTrader backend application

echo "Starting iTrader Backend..."
echo

# Check if running in auto mode
if [ "$1" == "--auto" ]; then
    echo "🤖 Running in AUTOMATIC mode - all actions will be auto-confirmed!"
    echo "⚠️  Transactions will be processed WITHOUT manual confirmation!"
    cargo run --bin itrader-backend -- --auto
else
    echo "👤 Running in MANUAL mode - actions require confirmation"
    echo "💡 To run in automatic mode, use: ./run_app.sh --auto"
    cargo run --bin itrader-backend
fi