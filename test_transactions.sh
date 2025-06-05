#!/bin/bash

# Test transaction fetching
cargo run --bin gate_list_available 2>&1 | grep -v warning