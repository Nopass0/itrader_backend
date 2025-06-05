#!/usr/bin/env python3
"""
Test Bybit P2P API Authentication
"""

import time
import hmac
import hashlib
import json
import requests
import sys

def test_p2p_auth(api_key, api_secret):
    """Test P2P authentication with correct signature"""
    print("=== Testing Bybit P2P Authentication ===")
    
    # Test endpoint - P2P get online ads (public endpoint but we'll test with auth)
    base_url = "https://api.bybit.com"
    endpoint = "/v5/p2p/item/online"
    url = base_url + endpoint
    
    # Parameters
    params = {
        "tokenId": "USDT",
        "currencyId": "RUB",
        "side": "0",  # Buy
        "page": "1",
        "size": "5"
    }
    
    # Generate authentication headers
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    
    # For POST requests with JSON body: timestamp + api_key + recv_window + json_body
    param_str = json.dumps(params, separators=(',', ':'))
    sign_str = timestamp + api_key + recv_window + param_str
    
    print(f"\nDebug Info:")
    print(f"Timestamp: {timestamp}")
    print(f"API Key: {api_key[:10]}...")
    print(f"Recv Window: {recv_window}")
    print(f"JSON Body: {param_str}")
    print(f"Sign String: {sign_str[:50]}...")
    
    # Create signature
    signature = hmac.new(
        api_secret.encode('utf-8'),
        sign_str.encode('utf-8'),
        hashlib.sha256
    ).hexdigest()
    
    print(f"Signature: {signature}")
    
    # Headers
    headers = {
        "X-BAPI-API-KEY": api_key,
        "X-BAPI-TIMESTAMP": timestamp,
        "X-BAPI-SIGN": signature,
        "X-BAPI-RECV-WINDOW": recv_window,
        "Content-Type": "application/json"
    }
    
    print("\nSending request...")
    
    try:
        # Make request
        response = requests.post(url, json=params, headers=headers, timeout=10)
        
        print(f"\nResponse Status: {response.status_code}")
        print(f"Response Headers: {dict(response.headers)}")
        
        result = response.json()
        print(f"\nResponse Body: {json.dumps(result, indent=2)}")
        
        if result.get("ret_code") == 0:
            print("\n✅ Authentication successful!")
            return True
        else:
            print(f"\n❌ Authentication failed: {result.get('ret_msg')}")
            
            # Check specific error codes
            if result.get("ret_code") == 10004:
                print("\n⚠️  Error 10004: Invalid signature")
                print("Common causes:")
                print("- Incorrect signature algorithm")
                print("- Wrong parameter order in sign string")
                print("- API secret encoding issues")
            elif result.get("ret_code") == 10002:
                print("\n⚠️  Error 10002: Invalid timestamp")
                print("Your system time might be out of sync")
            
            return False
            
    except Exception as e:
        print(f"\n❌ Request failed: {str(e)}")
        return False

def test_account_info(api_key, api_secret):
    """Test getting P2P account info (requires P2P permissions)"""
    print("\n\n=== Testing P2P Account Info ===")
    
    base_url = "https://api.bybit.com"
    endpoint = "/v5/p2p/user/personal/info"
    url = base_url + endpoint
    
    # Empty body for this request
    params = {}
    
    # Generate authentication
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    
    # For POST requests with JSON body
    param_str = json.dumps(params, separators=(',', ':'))
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
    
    try:
        response = requests.post(url, json=params, headers=headers, timeout=10)
        result = response.json()
        
        print(f"Response: {json.dumps(result, indent=2)}")
        
        if result.get("ret_code") == 0:
            print("\n✅ Successfully got P2P account info!")
            user_info = result.get("result", {}).get("userInfo", {})
            print(f"User ID: {user_info.get('userId')}")
            print(f"Nickname: {user_info.get('nickName')}")
            return True
        else:
            print(f"\n❌ Failed to get account info: {result.get('ret_msg')}")
            return False
            
    except Exception as e:
        print(f"\n❌ Request failed: {str(e)}")
        return False

def main():
    # Read credentials from file
    try:
        with open("test_data/bybit_creditials.json", "r") as f:
            creds = json.load(f)
            api_key = creds["api_key"]
            api_secret = creds["api_secret"]
    except Exception as e:
        print(f"Failed to load credentials: {e}")
        print("Please ensure test_data/bybit_creditials.json exists with:")
        print('{"api_key": "your_key", "api_secret": "your_secret"}')
        sys.exit(1)
    
    print(f"Using API Key: {api_key[:10]}...")
    
    # Test authentication
    auth_success = test_p2p_auth(api_key, api_secret)
    
    # Test account info if auth succeeded
    if auth_success:
        test_account_info(api_key, api_secret)
    
    print("\n" + "="*50)
    print("Test completed!")

if __name__ == "__main__":
    main()