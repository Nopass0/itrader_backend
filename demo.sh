#!/bin/bash

echo "=== iTrader Backend Demo ==="
echo
echo "This demonstrates the complete authentication flow:"
echo

# Step 1: Login to get cookies
echo "1. Login with credentials to get fresh cookies:"
echo "   Running: ./test.sh gate-login"
./test.sh gate-login
echo

# Step 2: Show saved cookies
echo "2. Cookies saved to test_data/gate_cookie.json:"
echo "   First few lines of cookie file:"
head -n 10 test_data/gate_cookie.json
echo

# Step 3: Use cookies for authenticated requests
echo "3. Using cookies for authenticated requests:"
echo

echo "   a) Testing balance setting:"
./test.sh gate-balance
echo

echo "   b) Testing transaction retrieval:"
./test.sh gate-tx
echo

echo "=== Demo Complete ==="
echo
echo "In production:"
echo "- Replace panel.gate.cx with real Gate.io API"
echo "- The 'Internal server error' will be replaced with actual API responses"
echo "- Cookies expire after 24 hours, so automatic refresh is needed"