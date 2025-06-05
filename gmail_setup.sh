#!/bin/bash

echo "=== Gmail OAuth2 Setup ==="
echo
echo "This will set up Gmail authentication for the iTrader backend."
echo

# Run the gmail_auth binary
./target/debug/gmail_auth

echo
echo "Setup complete! You can now run Gmail tests:"
echo "  ./test.sh gmail-list-today"
echo "  ./test.sh gmail-latest"
echo "  ./test.sh gmail-latest-pdf"