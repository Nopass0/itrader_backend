#!/usr/bin/env python3
"""Test creating ad with correct payment ID"""

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

# Endpoint
url = "https://api.bybit.com/v5/p2p/item/create"

# Ad parameters with correct payment ID
params = {
    "tokenId": "USDT",
    "currencyId": "RUB",
    "side": "1",  # Sell
    "priceType": "0",  # Fixed price
    "premium": "",
    "price": "98.50",
    "minAmount": "1000",
    "maxAmount": "5000",
    "remark": "Fast trade via API, reliable seller",
    "tradingPreferenceSet": {
        "hasUnPostAd": 0,
        "isKyc": 0,
        "isEmail": 0,
        "isMobile": 0,
        "hasRegisterTime": 0,
        "registerTimeThreshold": 0,
        "orderFinishNumberDay30": 0,
        "completeRateDay30": "0",
        "nationalLimit": "",
        "hasOrderFinishNumberDay30": 0,
        "hasCompleteRateDay30": 0,
        "hasNationalLimit": 0
    },
    "paymentIds": ["18175385"],  # User's actual Tinkoff payment ID
    "quantity": "50",  # 50 USDT
    "paymentPeriod": "15",
    "itemType": "ORIGIN"
}

# Generate timestamp
timestamp = str(int(time.time() * 1000))
recv_window = "5000"

# Create JSON string
param_str = json.dumps(params, separators=(',', ':'), sort_keys=True)

# Build signature string
sign_str = timestamp + api_key + recv_window + param_str

# Create signature
signature = hmac.new(
    api_secret.encode('utf-8'),
    sign_str.encode('utf-8'),
    hashlib.sha256
).hexdigest()

# Headers
headers = {
    "X-BAPI-API-KEY": api_key,
    "X-BAPI-TIMESTAMP": timestamp,
    "X-BAPI-SIGN": signature,
    "X-BAPI-RECV-WINDOW": recv_window,
    "Content-Type": "application/json"
}

print("=== Creating P2P Ad with Correct Payment ID ===")
print(f"Creating ad: Sell {params['quantity']} USDT at {params['price']} RUB/USDT")
print(f"Payment method: Tinkoff (ID: 18175385)")

try:
    # Make request
    response = requests.post(
        url, 
        data=param_str,  # Send as raw string
        headers=headers, 
        timeout=10
    )
    
    print(f"\nStatus Code: {response.status_code}")
    
    result = response.json()
    
    if result.get("ret_code") == 0:
        print("✅ Successfully created ad!")
        ad_info = result.get("result", {})
        print(f"Ad ID: {ad_info.get('itemId')}")
        print(f"\nFull response: {json.dumps(result, indent=2)}")
        
        print("\n⚠️  Note: This is a real P2P ad that will be visible to other users!")
        print("You may want to delete it if this was just a test.")
    else:
        print(f"❌ Failed to create ad: {result.get('ret_msg')}")
        print(f"Error code: {result.get('ret_code')}")
        print(f"\nFull response: {json.dumps(result, indent=2)}")
        
except Exception as e:
    print(f"❌ Request failed: {str(e)}")