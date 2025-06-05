#\!/bin/bash

# Set up environment for running the iTrader Backend
export LD_LIBRARY_PATH="/home/user/.local/share/uv/python/cpython-3.11.12-linux-x86_64-gnu/lib:$LD_LIBRARY_PATH"

# Run the application
exec ./target/debug/itrader-backend "$@"
EOF < /dev/null