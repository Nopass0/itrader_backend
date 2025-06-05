#!/bin/bash

# Script to test Gate.io API with exact browser headers
# Usage: ./gate_browser_request.sh

echo "=== Gate.io Browser Request Test ==="
echo
echo "This script will make a request exactly as the browser does."
echo "Make sure you have cookies in .gate_cookies.json"
echo

# Load cookies from JSON file
if [ ! -f ".gate_cookies.json" ]; then
    echo "Error: .gate_cookies.json not found. Run gate-login first."
    exit 1
fi

# Extract cookies using jq (or python if jq not available)
if command -v jq &> /dev/null; then
    SID=$(jq -r '.[] | select(.name=="sid") | .value' .gate_cookies.json)
    RSID=$(jq -r '.[] | select(.name=="rsid") | .value' .gate_cookies.json)
else
    # Use python as fallback
    SID=$(python3 -c "import json; cookies=json.load(open('.gate_cookies.json')); print([c['value'] for c in cookies if c['name']=='sid'][0])")
    RSID=$(python3 -c "import json; cookies=json.load(open('.gate_cookies.json')); print([c['value'] for c in cookies if c['name']=='rsid'][0])")
fi

if [ -z "$SID" ] || [ -z "$RSID" ]; then
    echo "Error: Could not extract cookies from .gate_cookies.json"
    exit 1
fi

echo "Found cookies:"
echo "  sid: ${SID:0:20}..."
echo "  rsid: ${RSID:0:20}..."
echo

# URL with filters - properly encoded
URL="https://panel.gate.cx/api/v1/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1"

echo "Making request to: $URL"
echo

# Make the request with exact browser headers
curl -v "$URL" \
  -H 'accept: application/json, text/plain, */*' \
  -H 'accept-language: ru,en;q=0.9,pl;q=0.8' \
  -H "cookie: sid=$SID; rsid=$RSID" \
  -H 'priority: u=1, i' \
  -H 'referer: https://panel.gate.cx/' \
  -H 'sec-ch-ua: "Google Chrome";v="131", "Chromium";v="131", "Not_A Brand";v="24"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "Windows"' \
  -H 'sec-fetch-dest: empty' \
  -H 'sec-fetch-mode: cors' \
  -H 'sec-fetch-site: same-origin' \
  -H 'user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36' \
  2>&1 | tee gate_browser_request.log

echo
echo "Response saved to gate_browser_request.log"