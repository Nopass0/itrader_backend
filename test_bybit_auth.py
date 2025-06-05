#!/usr/bin/env python3
"""
Test Bybit authentication with account info endpoint
"""

import json
import time
import hmac
import hashlib
import requests

def test_auth():
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    
    # Test with account info endpoint (GET request)
    base_url = "https://api.bybit.com"
    endpoint = "/v5/account/wallet-balance"
    url = base_url + endpoint
    
    # Parameters for GET request
    params = {
        "accountType": "UNIFIED"
    }
    
    # Generate signature for GET request
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    
    # For GET requests: timestamp + api_key + recv_window + query_string
    query_string = "&".join([f"{k}={v}" for k, v in sorted(params.items())])
    sign_str = timestamp + api_key + recv_window + query_string
    
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
    }
    
    # Make GET request
    response = requests.get(url, params=params, headers=headers)
    
    print(f"Status: {response.status_code}")
    print(f"Response: {response.text}")
    
    if response.status_code == 200:
        result = response.json()
        if result.get("retCode") == 0:
            print("✅ Authentication successful!")
            return True
        else:
            print(f"❌ API error: {result}")
            return False
    else:
        print(f"❌ HTTP error: {response.status_code}")
        return False

if __name__ == "__main__":
    test_auth()