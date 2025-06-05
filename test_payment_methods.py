#!/usr/bin/env python3
"""
Test script to get available payment methods for the account
"""

import json
import time
import hmac
import hashlib
import requests

def get_payment_methods():
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    
    base_url = "https://api.bybit.com"
    endpoint = "/v5/user/query-api"  # This will show account info
    url = base_url + endpoint
    
    # Generate authentication
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    query_string = ""
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
    
    response = requests.get(url, headers=headers, timeout=10)
    
    if response.status_code == 200:
        result = response.json()
        print("✅ Account API info:")
        print(json.dumps(result, indent=2))
    else:
        print(f"❌ HTTP {response.status_code}: {response.text}")

if __name__ == "__main__":
    get_payment_methods()