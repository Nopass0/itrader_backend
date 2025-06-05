#!/bin/bash

echo "=== Gate.io Login and Cookie Test Script ==="
echo
echo "This script demonstrates:"
echo "1. Login with credentials to get fresh cookies"
echo "2. Save cookies to file"
echo "3. Use saved cookies for authenticated requests"
echo
echo "To use this in production:"
echo
echo "1. Run the login test to get fresh cookies:"
echo "   ./test.sh gate-login"
echo
echo "2. The cookies will be saved to test_data/gate_cookie.json"
echo
echo "3. Use the cookies for authenticated requests:"
echo "   - Set balance: ./test.sh gate-balance"
echo "   - Get transactions: ./test.sh gate-tx"
echo
echo "Note: Cookies expire after 24 hours, so you'll need to login again"
echo
echo "The system is designed to:"
echo "- Automatically login all Gate.io accounts on startup"
echo "- Set balance to 1,000,000 RUB"
echo "- Monitor for pending transactions continuously"
echo "- Accept transactions and create Bybit ads"
echo