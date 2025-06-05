#!/bin/bash
# Quick start script
echo "Starting iTrader Backend..."
export PATH="$HOME/.cargo/bin:$PATH"
source .venv/bin/activate 2>/dev/null || true
./run.sh
