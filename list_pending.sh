#\!/bin/bash
# Quick script to list pending Gate.io transactions

# Set Python library path
export LD_LIBRARY_PATH="/home/user/.local/share/uv/python/cpython-3.11.12-linux-x86_64-gnu/lib:$LD_LIBRARY_PATH"

# Set admin token to avoid config error
export APP__admin_token="${APP__admin_token:-dev-token}"

# Run the gate_list_pending binary
cargo run --bin gate_list_pending
