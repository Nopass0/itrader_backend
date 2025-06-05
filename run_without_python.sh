#!/bin/bash

# Use LD_PRELOAD to redirect Python library
export LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libpython3.13.so.1.0

# Run with logging
RUST_LOG=info,itrader_backend=debug,itrader_backend::core::orchestrator=debug,itrader_backend::gate=debug target/debug/itrader-backend "$@"