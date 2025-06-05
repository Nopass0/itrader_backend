#!/bin/bash

# Auto-Trader Demo Startup Script

echo "=================================="
echo "    iTrader Auto-Trader System    "
echo "==================================  (DEMO)"
echo

if [ "$1" = "--auto" ]; then
    echo "🤖 AUTOMATIC MODE"
    echo "⚠️  WARNING: All actions will be auto-confirmed!"
    echo
    read -p "Are you sure you want to run in AUTOMATIC mode? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Cancelled. Exiting..."
        exit 1
    fi
    rustc src/main_simple.rs -o itrader_demo && ./itrader_demo --auto
else
    echo "👤 MANUAL MODE (default)"
    echo "✅ All transactions will require manual confirmation"
    echo "💡 This is the recommended mode for testing"
    echo
    echo "To run in automatic mode, use: ./start_demo.sh --auto"
    echo
    rustc src/main_simple.rs -o itrader_demo && ./itrader_demo
fi