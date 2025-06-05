#!/bin/bash

# Auto-Trader System Startup Script

echo "=========================================="
echo "   Gate.io ↔ Bybit P2P Auto-Trader"
echo "=========================================="
echo

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "❌ Virtual environment not found!"
    echo "Please run ./setup.sh first"
    exit 1
fi

# Check for required files
if [ ! -f ".env" ]; then
    echo "❌ .env file not found!"
    echo "Please copy .env.example to .env and configure it"
    exit 1
fi

if [ ! -f ".gate_cookies.json" ]; then
    echo "⚠️  WARNING: .gate_cookies.json not found!"
    echo "The system will not be able to connect to Gate.io"
    echo
    read -p "Continue anyway? (y/n): " confirm
    if [ "$confirm" != "y" ]; then
        exit 1
    fi
fi

# Parse command line arguments
MODE="manual"
if [ "$1" = "--auto" ] || [ "$1" = "--automatic" ]; then
    MODE="automatic"
fi

# Display mode information
if [ "$MODE" = "automatic" ]; then
    echo "🤖 AUTOMATIC MODE"
    echo "⚠️  WARNING: All transactions will be processed automatically!"
    echo "⚠️  Chat messages will be sent without manual confirmation!"
    echo
    read -p "Are you SURE you want to run in AUTOMATIC mode? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Cancelled. Starting in manual mode instead..."
        MODE="manual"
    fi
else
    echo "👤 MANUAL MODE"
    echo "✅ All transactions require manual confirmation"
    echo "💡 To run in automatic mode, use: ./start_trader.sh --auto"
fi

echo
echo "Starting trader system in $MODE mode..."
echo

# Activate virtual environment and run
source venv/bin/activate
python trader_system.py --mode $MODE