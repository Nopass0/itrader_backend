#!/usr/bin/env python3
"""Test getting online P2P ads - checking if this is a public or private endpoint"""

import time
import hmac
import hashlib
import json
import requests

# First, test without authentication
print("=== Testing without authentication ===")
url = "https://api.bybit.com/v5/p2p/item/online"

params = {
    "tokenId": "USDT",
    "currencyId": "RUB",
    "side": "0",
    "page": "1",
    "size": "5"
}

headers = {
    "Content-Type": "application/json"
}

response = requests.post(url, json=params, headers=headers, timeout=10)
print(f"Status: {response.status_code}")
print(f"Response: {response.text[:500]}")

# Now test with different endpoint paths
print("\n=== Testing alternative endpoints ===")

# Try v5/p2p/online/list (hypothetical)
alt_endpoints = [
    "/v5/p2p/ad/list",
    "/v5/p2p/list",
    "/v5/p2p/public/list",
    "/v5/p2p/public/item/online"
]

for endpoint in alt_endpoints:
    url = f"https://api.bybit.com{endpoint}"
    try:
        response = requests.post(url, json=params, headers=headers, timeout=5)
        print(f"\n{endpoint}: Status {response.status_code}")
        if response.status_code != 404:
            print(f"Response: {response.text[:200]}")
    except:
        print(f"\n{endpoint}: Failed to connect")

# Check if we need to use GET instead of POST
print("\n=== Testing with GET method ===")
url = "https://api.bybit.com/v5/p2p/item/online"
try:
    response = requests.get(url, params=params, headers=headers, timeout=10)
    print(f"GET Status: {response.status_code}")
    print(f"Response: {response.text[:500]}")
except Exception as e:
    print(f"GET failed: {e}")