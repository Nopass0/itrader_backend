#!/usr/bin/env python3
"""
Check Bybit API permissions and find available endpoints
"""

import json
import time
import hmac
import hashlib
import requests

def check_api_permissions():
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    base_url = "https://api.bybit.com"
    endpoint = "/v5/user/query-api"
    
    # Prepare request
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
    
    # Make request
    url = base_url + endpoint
    response = requests.get(url, headers=headers)
    
    if response.status_code == 200:
        result = response.json()
        if result.get("retCode") == 0:
            api_info = result["result"]
            
            print("🔑 API Key Information:")
            print(f"   API Key: {api_info.get('apiKey', 'N/A')}")
            print(f"   Read Only: {api_info.get('readOnly', 'N/A')}")
            print(f"   Type: {api_info.get('type', 'N/A')}")
            print(f"   VIP Level: {api_info.get('vipLevel', 'N/A')}")
            print(f"   KYC Level: {api_info.get('kycLevel', 'N/A')}")
            print(f"   KYC Region: {api_info.get('kycRegion', 'N/A')}")
            print(f"   Unified: {api_info.get('unified', 'N/A')}")
            print(f"   UTA: {api_info.get('uta', 'N/A')}")
            
            permissions = api_info.get('permissions', {})
            print(f"\n🎯 API Permissions:")
            for category, perms in permissions.items():
                print(f"   {category}: {perms}")
                
            print(f"\n📋 Full API Info:")
            print(json.dumps(api_info, indent=2, ensure_ascii=False))
            
        else:
            print(f"❌ API Error: {result}")
    else:
        print(f"❌ HTTP Error: {response.status_code} - {response.text}")

def test_spot_trading():
    """Test if we can access spot trading (which might include OTC)"""
    print("\n🔍 Testing Spot Trading Access...")
    
    # Load credentials  
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    base_url = "https://api.bybit.com"
    
    # Try spot market data (should work without special permissions)
    endpoint = "/v5/market/tickers"
    params = {"category": "spot", "symbol": "BTCUSDT"}
    
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    
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
    
    url = base_url + endpoint
    response = requests.get(url, params=params, headers=headers)
    
    if response.status_code == 200:
        result = response.json()
        if result.get("retCode") == 0:
            print("✅ Spot market data accessible")
            tickers = result.get("result", {}).get("list", [])
            if tickers:
                ticker = tickers[0]
                print(f"   BTC/USDT: {ticker.get('lastPrice', 'N/A')}")
        else:
            print(f"❌ Spot API Error: {result.get('retMsg', 'Unknown')}")
    else:
        print(f"❌ Spot HTTP Error: {response.status_code}")

if __name__ == "__main__":
    check_api_permissions()
    test_spot_trading()