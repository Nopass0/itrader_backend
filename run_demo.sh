#!/bin/bash

# Interactive Demo Runner

echo "=================================="
echo "    iTrader Auto-Trader Demo      "
echo "=================================="
echo
echo "This demo simulates the auto-trader confirmation system."
echo
echo "Choose mode:"
echo "1) Manual mode (requires confirmation for each action)"
echo "2) Automatic mode (processes all actions automatically)"
echo
read -p "Enter your choice (1 or 2): " mode

if [ "$mode" = "2" ]; then
    echo
    echo "⚠️  WARNING: Automatic mode will process all actions without confirmation!"
    read -p "Are you sure you want to run in AUTOMATIC mode? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Cancelled. Running in manual mode instead..."
        python3 demo.py
    else
        python3 demo.py --auto
    fi
else
    python3 demo.py
fi