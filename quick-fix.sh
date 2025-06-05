#!/bin/bash

echo "Quick fix for build dependencies..."
echo

# Quick install for Ubuntu/Debian
if command -v apt-get &> /dev/null; then
    echo "Installing required packages..."
    sudo apt-get update
    sudo apt-get install -y \
        libssl-dev \
        pkg-config \
        redis-server \
        libtesseract-dev \
        libleptonica-dev \
        clang
    
    echo "Starting Redis..."
    sudo systemctl start redis-server
    
    echo
    echo "✅ Dependencies installed!"
    echo "You can now run: cargo build"
else
    echo "This quick fix is for Ubuntu/Debian only."
    echo "Please run: ./install-deps.sh for full installation"
fi