#!/usr/bin/env python3
"""
Debug script to test P2P ad creation with detailed error info
"""

import json
import time
import hmac
import hashlib
import requests

def debug_create_ad():
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    
    base_url = "https://api.bybit.com"
    endpoint = "/v5/p2p/item/create"
    url = base_url + endpoint
    
    # Use minimal required parameters
    params = {
        "tokenId": "USDT",
        "currencyId": "RUB", 
        "side": "0",  # Buy USDT for RUB
        "priceType": "0",  # Fixed rate
        "premium": "",
        "price": "90.00",  # Within allowed range
        "minAmount": "80",  # Less than total (90 RUB)
        "maxAmount": "90",  # Equal to total available
        "remark": "Test ad",
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
        "paymentIds": ["7110"],  # Try EUR payment method from the example
        "quantity": "1",  # Minimum quantity
        "paymentPeriod": "15",
        "itemType": "ORIGIN"
    }
    
    # Generate authentication
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    
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
    
    print("🔍 Testing P2P ad creation with parameters:")
    print(json.dumps(params, indent=2))
    print(f"\n📡 Sending request to: {url}")
    print(f"🔐 Signature string: {sign_str}")
    
    response = requests.post(url, data=param_str, headers=headers, timeout=10)
    
    print(f"\n📬 Response status: {response.status_code}")
    
    if response.status_code == 200:
        try:
            result = response.json()
            print("📋 Response data:")
            print(json.dumps(result, indent=2))
            
            ret_code = result.get("ret_code", -1)
            if ret_code == 0:
                print("✅ Ad creation successful!")
            else:
                print(f"❌ API Error {ret_code}: {result.get('ret_msg', 'Unknown')}")
        except json.JSONDecodeError:
            print(f"❌ Invalid JSON response: {response.text}")
    else:
        print(f"❌ HTTP Error: {response.text}")

if __name__ == "__main__":
    debug_create_ad()