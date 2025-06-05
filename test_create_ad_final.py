#!/usr/bin/env python3
"""Final test creating ad with all parameters as strings"""

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

# Ad parameters - all values as strings based on documentation
params = {
    "tokenId": "USDT",
    "currencyId": "RUB",
    "side": "1",  # Sell
    "priceType": "0",  # Fixed price
    "premium": "",
    "price": "90.50",  # Within allowed range 71.42-91.26
    "minAmount": "1000",  # Minimum 1000 RUB
    "maxAmount": "1810",  # Maximum 1810 RUB (20 USDT * 90.50)
    "remark": "Fast trade via API",
    "tradingPreferenceSet": {
        "hasUnPostAd": "0",
        "isKyc": "0", 
        "isEmail": "0",
        "isMobile": "0",
        "hasRegisterTime": "0",
        "registerTimeThreshold": "0",
        "orderFinishNumberDay30": "0",
        "completeRateDay30": "0",
        "nationalLimit": "",
        "hasOrderFinishNumberDay30": "0",
        "hasCompleteRateDay30": "0",
        "hasNationalLimit": "0"
    },
    "paymentIds": ["18175385"],  # User's actual Tinkoff payment ID
    "quantity": "20",  # 20 USDT
    "paymentPeriod": "15",
    "itemType": "ORIGIN"
}

print("=== Testing Create Ad with All String Parameters ===")
print(f"Parameters: {json.dumps(params, indent=2)}")

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

try:
    response = requests.post(url, data=param_str, headers=headers, timeout=10)
    
    print(f"\nStatus Code: {response.status_code}")
    
    result = response.json()
    
    if result.get("ret_code") == 0:
        print("✅ Successfully created ad!")
        ad_info = result.get("result", {})
        print(f"Ad ID: {ad_info.get('itemId')}")
    else:
        print(f"❌ Failed: {result.get('ret_msg')}")
        print(f"Error code: {result.get('ret_code')}")
        
        # Try to understand what's wrong
        if result.get("ret_code") == 10001:
            print("\nPossible issues:")
            print("1. Payment ID might be invalid or not active")
            print("2. Quantity might be too low/high for the account")
            print("3. Price might be outside allowed range")
            print("4. Account might not have P2P trading enabled")
            
except Exception as e:
    print(f"❌ Request failed: {str(e)}")