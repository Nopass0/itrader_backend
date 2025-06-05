#!/usr/bin/env python3
"""Test P2P rates with debug output"""

import time
import hmac
import hashlib
import json
import requests

# Load credentials
with open("test_data/bybit_creditials.json", "r") as f:
    creds = json.load(f)

api_key = creds["api_key"]
api_secret = creds["api_secret"]

# Test endpoint
url = "https://api.bybit.com/v5/p2p/item/online"

# Parameters
params = {
    "tokenId": "USDT",
    "currencyId": "RUB",
    "side": "0",  # Buy
    "page": "1",
    "size": "20"
}

# Generate authentication
timestamp = str(int(time.time() * 1000))
recv_window = "5000"

# Create signature
param_str = json.dumps(params, separators=(',', ':'), sort_keys=True)
sign_str = timestamp + api_key + recv_window + param_str

signature = hmac.new(
    api_secret.encode('utf-8'),
    sign_str.encode('utf-8'),
    hashlib.sha256
).hexdigest()

headers = {
    "X-BAPI-API-KEY": api_key,
    "X-BAPI-TIMESTAMP": timestamp,
    "X-BAPI-SIGN": signature,
    "X-BAPI-RECV-WINDOW": recv_window,
    "Content-Type": "application/json"
}

print(f"URL: {url}")
print(f"Params: {param_str}")
print(f"Headers: {headers}")

# Make request
response = requests.post(url, json=params, headers=headers, timeout=10)

print(f"\nStatus Code: {response.status_code}")
print(f"Response: {response.text[:1000]}")

if response.status_code == 200:
    result = response.json()
    if result.get("ret_code") == 0:
        items = result.get("result", {}).get("items", [])
        print(f"\nFound {len(items)} P2P ads")
        
        if items:
            print("\nTop 5 ads:")
            for i, item in enumerate(items[:5]):
                print(f"{i+1}. Price: {item.get('price')} RUB/USDT, Min: {item.get('minAmount')}, Max: {item.get('maxAmount')}")
    else:
        print(f"\nError: {result.get('ret_msg')}")