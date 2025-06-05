#!/usr/bin/env python3
"""
Test different Bybit endpoints to find working P2P/OTC endpoints
"""

import json
import time
import hmac
import hashlib
import requests

def test_different_endpoints():
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    base_url = "https://api.bybit.com"
    
    # List of endpoints to try
    endpoints_to_try = [
        # P2P related
        "/v5/p2p/item/list",
        "/v5/otc/item/list", 
        "/v5/market/tickers",  # This should work
        "/v5/market/kline",    # This should work
        
        # User/account related
        "/v5/user/query-api",   # Check API permissions
        "/v5/account/info",     # Account info
        
        # Asset related  
        "/v5/asset/coin/query-info",  # Coin info
    ]
    
    for endpoint in endpoints_to_try:
        try:
            print(f"\n🔍 Testing: {endpoint}")
            
            # Prepare request
            timestamp = str(int(time.time() * 1000))
            recv_window = "5000"
            
            # For GET requests - empty query string
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
            response = requests.get(url, headers=headers, timeout=10)
            
            print(f"Status: {response.status_code}")
            
            if response.status_code == 200:
                result = response.json()
                ret_code = result.get("retCode", -1)
                ret_msg = result.get("retMsg", "Unknown")
                
                if ret_code == 0:
                    print(f"✅ SUCCESS: {ret_msg}")
                    if "result" in result:
                        # Print first few keys of result to see structure
                        result_keys = list(result["result"].keys()) if isinstance(result["result"], dict) else "list" if isinstance(result["result"], list) else "other"
                        print(f"   Result structure: {result_keys}")
                else:
                    print(f"❌ API Error {ret_code}: {ret_msg}")
            else:
                print(f"❌ HTTP Error: {response.text[:200]}")
                
        except Exception as e:
            print(f"❌ Exception: {str(e)[:100]}")

if __name__ == "__main__":
    test_different_endpoints()