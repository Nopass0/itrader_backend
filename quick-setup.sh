#!/bin/bash

# Quick setup script for iTrader Backend
# This script runs the full installation and starts the server

set -e

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}================================================"
echo "iTrader Backend - Quick Setup"
echo -e "================================================${NC}"
echo

# Run the installation script
./install.sh

# If installation was successful, offer to start the server
if [ $? -eq 0 ]; then
    echo
    echo -e "${GREEN}Installation completed!${NC}"
    echo
    read -p "Start the server now? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Starting iTrader Backend..."
        ./run.sh
    else
        echo "You can start the server later with: ./run.sh"
    fi
fi