#!/usr/bin/env python3
"""Test P2P scripts with fixed authentication"""

import json
import subprocess
import sys

def test_get_rates():
    """Test getting P2P rates"""
    print("=== Testing Get P2P Rates ===")
    
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        creds = json.load(f)
    
    # Prepare input for script
    input_data = {
        "api_key": creds["api_key"],
        "api_secret": creds["api_secret"],
        "amount_rub": 10000,
        "testnet": False
    }
    
    # Run script
    result = subprocess.run(
        ["python3", "scripts/bybit_get_rates.py"],
        input=json.dumps(input_data),
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        try:
            output = json.loads(result.stdout)
            print(f"Success: {output.get('success')}")
            if output.get('success'):
                data = output.get('data', {})
                print(f"Buy Rate: {data.get('buy_rate')} RUB/USDT")
                print(f"Sell Rate: {data.get('sell_rate')} RUB/USDT")
                print(f"Spread: {data.get('spread')}")
                print(f"Source: {output.get('source')}")
            else:
                print(f"Error: {output.get('error')}")
        except json.JSONDecodeError:
            print(f"Failed to parse output: {result.stdout}")
    else:
        print(f"Script failed: {result.stderr}")

def test_create_ad():
    """Test creating a P2P ad"""
    print("\n\n=== Testing Create P2P Ad ===")
    
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        creds = json.load(f)
    
    # Prepare ad parameters
    ad_params = {
        "tokenId": "USDT",
        "currency": "RUB",
        "side": "1",  # Sell
        "price": "98.50",
        "min_amount": "1000",
        "max_amount": "5000",
        "quantity": "50",  # 50 USDT
        "remarks": "Test ad via API",
        "payment_methods": ["75"]  # Tinkoff
    }
    
    # Prepare input for script
    input_data = {
        "api_key": creds["api_key"],
        "api_secret": creds["api_secret"],
        "ad_params": ad_params,
        "testnet": False
    }
    
    # Run script
    result = subprocess.run(
        ["python3", "scripts/bybit_create_ad.py"],
        input=json.dumps(input_data),
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        try:
            output = json.loads(result.stdout)
            ret_code = output.get('ret_code')
            print(f"Return Code: {ret_code}")
            print(f"Message: {output.get('ret_msg')}")
            
            if ret_code == 0:
                result_data = output.get('result', {})
                print(f"Ad ID: {result_data.get('itemId')}")
                print("✅ Successfully created P2P ad!")
            else:
                print("❌ Failed to create ad")
                
        except json.JSONDecodeError:
            print(f"Failed to parse output: {result.stdout}")
    else:
        print(f"Script failed: {result.stderr}")

def main():
    # Test get rates
    test_get_rates()
    
    # Ask before creating ad
    if "--create-ad" in sys.argv:
        test_create_ad()
    else:
        print("\nTo test creating an ad, run with --create-ad flag")

if __name__ == "__main__":
    main()