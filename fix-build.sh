#!/bin/bash

echo "Fixing build issues..."

# Add missing sqlx features
echo "Adding decimal support to sqlx..."
sed -i 's/sqlx = { version = "0.7", features = \["runtime-tokio-rustls", "postgres", "json", "chrono", "uuid"\] }/sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "json", "chrono", "uuid", "rust_decimal"] }/' Cargo.toml

echo "Build fixes applied. Running cargo build..."
cargo build