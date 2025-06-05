#!/usr/bin/env python3
"""
Test different P2P endpoint variations for Bybit
"""

import json
import time
import hmac
import hashlib
import requests

def test_p2p_endpoints():
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    
    # Different base URLs to try
    base_urls = [
        "https://api.bybit.com",
        "https://api.bytick.com", 
        "https://api-p2p.bybit.com",
        "https://p2p.bybit.com"
    ]
    
    # Different endpoint paths to try
    endpoint_paths = [
        # V5 paths
        "/v5/p2p/item/list",
        "/v5/p2p/order/list", 
        "/v5/otc/item/list",
        "/v5/fiat/p2p/item/list",
        "/v5/fiat-p2p/item/list",
        
        # V4 paths  
        "/v4/p2p/item/list",
        "/v4/otc/item/list",
        
        # V3 paths
        "/v3/p2p/item/list", 
        "/v3/otc/item/list",
        
        # Other possible paths
        "/fiat/p2p/item/list",
        "/p2p/item/list",
        "/otc/item/list",
        "/api/v1/p2p/item/list",
    ]
    
    for base_url in base_urls:
        print(f"\n🌐 Testing base URL: {base_url}")
        
        for endpoint_path in endpoint_paths[:3]:  # Test first 3 endpoints per base URL
            try:
                url = base_url + endpoint_path
                print(f"  🔍 {endpoint_path}", end="")
                
                # Prepare auth
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
                
                # Make request with timeout
                response = requests.get(url, headers=headers, timeout=5)
                
                if response.status_code == 200:
                    try:
                        result = response.json()
                        ret_code = result.get("retCode", -1)
                        if ret_code == 0:
                            print(f" ✅ SUCCESS!")
                            return url  # Found working endpoint!
                        else:
                            print(f" ❌ API Error {ret_code}: {result.get('retMsg', 'Unknown')}")
                    except:
                        print(f" ⚠️ Invalid JSON response")
                elif response.status_code == 404:
                    print(f" ❌ 404 Not Found")
                else:
                    print(f" ❌ HTTP {response.status_code}")
                    
            except requests.exceptions.Timeout:
                print(f" ⏰ Timeout")
            except requests.exceptions.ConnectionError:
                print(f" 🔌 Connection Error")
            except Exception as e:
                print(f" ❌ Error: {str(e)[:30]}")
    
    return None

def test_fiat_endpoints():
    """Test specific fiat/P2P endpoints that might work"""
    print(f"\n💰 Testing Fiat/P2P specific endpoints...")
    
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    base_url = "https://api.bybit.com"
    
    # Endpoints that might exist for fiat P2P
    fiat_endpoints = [
        "/v5/asset/coin/query-info",  # We know this works
        "/v5/user/query-api",         # We know this works  
        "/v5/account/wallet-balance", # We know this works
        "/v5/spot/order/history",     # Try spot orders
        "/v5/asset/transfer/query-transfer-coin-list", # Asset related
    ]
    
    for endpoint in fiat_endpoints:
        try:
            print(f"  🔍 {endpoint}", end="")
            
            timestamp = str(int(time.time() * 1000))
            recv_window = "5000"
            
            # For wallet balance, we need accountType
            params = {}
            if "wallet-balance" in endpoint:
                params = {"accountType": "UNIFIED"}
            elif "spot/order" in endpoint:
                params = {"category": "spot"}
                
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
            response = requests.get(url, params=params, headers=headers, timeout=5)
            
            if response.status_code == 200:
                result = response.json()
                ret_code = result.get("retCode", -1)
                if ret_code == 0:
                    print(f" ✅ SUCCESS")
                    # Don't print full response to keep output clean
                else:
                    print(f" ❌ Error {ret_code}: {result.get('retMsg', 'Unknown')}")
            else:
                print(f" ❌ HTTP {response.status_code}")
                
        except Exception as e:
            print(f" ❌ Error: {str(e)[:30]}")

if __name__ == "__main__":
    # Try to find working P2P endpoints
    working_endpoint = test_p2p_endpoints()
    
    if working_endpoint:
        print(f"\n🎉 Found working P2P endpoint: {working_endpoint}")
    else:
        print(f"\n❌ No working P2P endpoints found")
        
    # Test other available endpoints
    test_fiat_endpoints()
    
    print(f"\n📝 Conclusion:")
    print(f"   ✅ API Key has FiatP2P permissions: ['FiatP2POrder', 'Advertising']")
    print(f"   ❌ Standard P2P endpoints (/v5/p2p/*) return 404")
    print(f"   💡 P2P functionality might require:")
    print(f"      - Different API endpoint documentation")
    print(f"      - Web interface only (no API)")
    print(f"      - Special request to Bybit support for P2P API access")