#!/usr/bin/env python3
"""Final test for Bybit P2P API with proper authentication"""

import time
import hmac
import hashlib
import json
import requests
import urllib3

# Disable SSL warnings for testing
urllib3.disable_warnings()

def test_online_ads_auth():
    """Test getting online ads with authentication"""
    
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        creds = json.load(f)
    
    api_key = creds["api_key"]
    api_secret = creds["api_secret"]
    
    # Endpoint
    url = "https://api.bybit.com/v5/p2p/item/online"
    
    # Parameters - MUST match exactly what's expected
    params = {
        "tokenId": "USDT",
        "currencyId": "RUB",
        "side": "0"  # Buy ads
    }
    
    # Generate timestamp
    timestamp = str(int(time.time() * 1000))
    recv_window = "5000"
    
    # Create JSON string with no spaces, sorted keys
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
    
    print("=== Testing Get Online Ads ===")
    print(f"URL: {url}")
    print(f"Params: {param_str}")
    print(f"Timestamp: {timestamp}")
    print(f"Signature: {signature}")
    
    try:
        # Make request
        response = requests.post(
            url, 
            data=param_str,  # Send as raw string, not json parameter
            headers=headers, 
            timeout=10
        )
        
        print(f"\nStatus Code: {response.status_code}")
        
        result = response.json()
        
        if result.get("ret_code") == 0:
            print("✅ Success!")
            items = result.get("result", {}).get("items", [])
            print(f"\nFound {len(items)} P2P ads")
            
            if items:
                print("\nTop 3 ads:")
                for i, item in enumerate(items[:3]):
                    print(f"\n{i+1}. Seller: {item.get('nickName')}")
                    print(f"   Price: {item.get('price')} RUB/USDT")
                    print(f"   Available: {item.get('lastQuantity')} USDT")
                    print(f"   Min: {item.get('minAmount')} RUB, Max: {item.get('maxAmount')} RUB")
                    print(f"   Payment methods: {item.get('payments')}")
            
            return True
        else:
            print(f"❌ Error: {result.get('ret_msg')}")
            print(f"Full response: {json.dumps(result, indent=2)}")
            return False
            
    except Exception as e:
        print(f"❌ Request failed: {str(e)}")
        return False

def test_create_ad():
    """Test creating a P2P ad"""
    
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        creds = json.load(f)
    
    api_key = creds["api_key"]
    api_secret = creds["api_secret"]
    
    # Endpoint
    url = "https://api.bybit.com/v5/p2p/item/create"
    
    # Ad parameters - minimal required fields
    params = {
        "tokenId": "USDT",
        "currencyId": "RUB",
        "side": "1",  # Sell
        "priceType": "0",  # Fixed price
        "premium": "",
        "price": "98.50",
        "minAmount": "1000",
        "maxAmount": "5000",
        "remark": "Fast trade, reliable seller",
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
    
    print("\n\n=== Testing Create Ad ===")
    print(f"Creating ad: Sell {params['quantity']} USDT at {params['price']} RUB/USDT")
    
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
            return True
        else:
            print(f"❌ Failed to create ad: {result.get('ret_msg')}")
            
            # Handle specific errors
            if result.get("ret_code") == 110025:
                print("Note: P2P functionality might not be available for your account")
            elif result.get("ret_code") == 10005:
                print("Note: Your API key might not have P2P permissions")
                
            return False
            
    except Exception as e:
        print(f"❌ Request failed: {str(e)}")
        return False

def main():
    print("Testing Bybit P2P API with proper authentication")
    print("=" * 50)
    
    # Test getting online ads
    ads_success = test_online_ads_auth()
    
    # Test creating ad if requested
    if ads_success and "--create-ad" in __import__('sys').argv:
        test_create_ad()
    elif ads_success:
        print("\nTo test creating an ad, run with --create-ad flag")
    
    print("\n" + "=" * 50)
    print("Test completed!")

if __name__ == "__main__":
    main()