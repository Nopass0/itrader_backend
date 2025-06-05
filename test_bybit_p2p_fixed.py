#!/usr/bin/env python3
"""
Test Bybit P2P API with corrected signature generation
"""

import time
import hmac
import hashlib
import json
import requests
import sys
from collections import OrderedDict

def test_public_p2p_endpoint():
    """Test public P2P endpoint without authentication"""
    print("=== Testing Public P2P Endpoint (No Auth) ===")
    
    url = "https://api.bybit.com/v5/p2p/item/online"
    
    params = {
        "tokenId": "USDT",
        "currencyId": "RUB",
        "side": "0",
        "page": "1",
        "size": "5"
    }
    
    headers = {
        "Content-Type": "application/json"
    }
    
    try:
        response = requests.post(url, json=params, headers=headers, timeout=10)
        result = response.json()
        
        print(f"Response: {json.dumps(result, indent=2)[:500]}...")
        
        if result.get("ret_code") == 0:
            print("\n✅ Public endpoint works!")
            items = result.get("result", {}).get("items", [])
            if items:
                print(f"Found {len(items)} P2P ads")
            return True
        else:
            print(f"\n❌ Public endpoint failed: {result.get('ret_msg')}")
            return False
            
    except Exception as e:
        print(f"\n❌ Request failed: {str(e)}")
        return False

def test_p2p_auth_fixed(api_key, api_secret):
    """Test P2P authentication with fixed signature generation"""
    print("\n\n=== Testing P2P Authentication (Fixed) ===")
    
    # Test with a simpler endpoint first - get P2P user info
    base_url = "https://api.bybit.com"
    endpoint = "/v5/p2p/user/personal/info"
    url = base_url + endpoint
    
    # Empty body for this request
    params = {}
    
    # Generate authentication headers
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    
    # For POST requests with JSON body: timestamp + api_key + recv_window + json_body
    # Make sure JSON has no spaces
    param_str = json.dumps(params, separators=(',', ':'), sort_keys=True)
    sign_str = timestamp + api_key + recv_window + param_str
    
    print(f"\nDebug Info:")
    print(f"Timestamp: {timestamp}")
    print(f"API Key: {api_key[:10]}...")
    print(f"Recv Window: {recv_window}")
    print(f"JSON Body: {param_str}")
    print(f"Sign String: {sign_str}")
    
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
    
    print("\nSending request to:", url)
    
    try:
        # Make request
        response = requests.post(url, json=params, headers=headers, timeout=10)
        
        print(f"\nResponse Status: {response.status_code}")
        
        result = response.json()
        print(f"\nResponse Body: {json.dumps(result, indent=2)}")
        
        if result.get("ret_code") == 0:
            print("\n✅ Authentication successful!")
            user_info = result.get("result", {}).get("userInfo", {})
            if user_info:
                print(f"User ID: {user_info.get('userId')}")
                print(f"Nickname: {user_info.get('nickName')}")
            return True
        else:
            print(f"\n❌ Authentication failed: {result.get('ret_msg')}")
            
            # Specific error handling
            if result.get("ret_code") == 10004:
                print("\n⚠️  Error 10004: Invalid signature")
                print("The signature generation is incorrect.")
            elif result.get("ret_code") == 10005:
                print("\n⚠️  Error 10005: Permission denied")
                print("Your API key might not have P2P permissions.")
            elif result.get("ret_code") == 110025:
                print("\n⚠️  Error 110025: P2P not available")
                print("P2P functionality might not be available for your account.")
            
            return False
            
    except Exception as e:
        print(f"\n❌ Request failed: {str(e)}")
        return False

def test_create_ad(api_key, api_secret):
    """Test creating a P2P advertisement"""
    print("\n\n=== Testing Create P2P Ad ===")
    
    base_url = "https://api.bybit.com"
    endpoint = "/v5/p2p/item/create"
    url = base_url + endpoint
    
    # Ad parameters
    params = {
        "tokenId": "USDT",
        "currencyId": "RUB",
        "side": "1",  # 1 = sell
        "priceType": "0",  # Fixed price
        "premium": "",
        "price": "98.50",
        "minAmount": "1000",
        "maxAmount": "10000",
        "remark": "Test ad from API",
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
        "paymentIds": ["75"],  # Tinkoff
        "quantity": "100",  # 100 USDT
        "paymentPeriod": "15",
        "itemType": "ORIGIN"
    }
    
    # Generate authentication
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    
    # Create compact JSON without spaces
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
    
    print(f"Creating ad with price: {params['price']} RUB/USDT")
    print(f"Amount: {params['quantity']} USDT")
    
    try:
        response = requests.post(url, json=params, headers=headers, timeout=10)
        result = response.json()
        
        print(f"\nResponse: {json.dumps(result, indent=2)}")
        
        if result.get("ret_code") == 0:
            print("\n✅ Successfully created P2P ad!")
            ad_info = result.get("result", {})
            print(f"Ad ID: {ad_info.get('itemId')}")
            return True
        else:
            print(f"\n❌ Failed to create ad: {result.get('ret_msg')}")
            return False
            
    except Exception as e:
        print(f"\n❌ Request failed: {str(e)}")
        return False

def main():
    # Read credentials
    try:
        with open("test_data/bybit_creditials.json", "r") as f:
            creds = json.load(f)
            api_key = creds["api_key"]
            api_secret = creds["api_secret"]
    except Exception as e:
        print(f"Failed to load credentials: {e}")
        sys.exit(1)
    
    print(f"Using API Key: {api_key[:10]}...")
    
    # Test public endpoint first
    test_public_p2p_endpoint()
    
    # Test authenticated endpoints
    auth_success = test_p2p_auth_fixed(api_key, api_secret)
    
    # Only test creating ad if auth succeeded
    if auth_success and "--create-ad" in sys.argv:
        test_create_ad(api_key, api_secret)
    elif auth_success:
        print("\nTo test creating an ad, run with --create-ad flag")
    
    print("\n" + "="*50)
    print("Test completed!")

if __name__ == "__main__":
    main()