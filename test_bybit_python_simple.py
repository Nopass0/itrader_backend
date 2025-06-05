#!/usr/bin/env python3
"""
Simple test for Bybit Python bridge
Tests both rate fetching and ad creation
"""

import subprocess
import json
import sys

def test_get_rates():
    """Test rate fetching"""
    print("=== Testing Rate Fetching ===")

    # Load real credentials from file
    try:
        with open("test_data/bybit_creditials.json", "r") as f:
            credentials = json.load(f)
    except FileNotFoundError:
        print("❌ No credentials file found at test_data/bybit_creditials.json")
        return False

    test_data = {
        "amount_rub": 10000, 
        "testnet": False,  # Use production API with real credentials
        "api_key": credentials["api_key"],
        "api_secret": credentials["api_secret"]
    }

    # Run the script
    result = subprocess.run(
        ["python3", "scripts/bybit_get_rates.py"],
        input=json.dumps(test_data),
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"❌ Script failed: {result.stderr}")
        return False

    # Parse response
    try:
        response = json.loads(result.stdout)
        if response["success"]:
            data = response["data"]
            print(f"✅ Buy rate: {data['buy_rate']} RUB/USDT")
            print(f"✅ Sell rate: {data['sell_rate']} RUB/USDT")
            print(f"✅ Spread: {data['spread']} RUB")
            return True
        else:
            print(f"❌ API error: {response.get('error', 'Unknown error')}")
            return False
    except json.JSONDecodeError as e:
        print(f"❌ Failed to parse response: {e}")
        print(f"Response: {result.stdout}")
        return False

def test_create_ad():
    """Test ad creation"""
    print("\n=== Testing Ad Creation ===")

    # Load real credentials from file
    try:
        with open("test_data/bybit_creditials.json", "r") as f:
            credentials = json.load(f)
    except FileNotFoundError:
        print("❌ No credentials file found at test_data/bybit_creditials.json")
        return False
    
    test_data = {
        "api_key": credentials["api_key"],
        "api_secret": credentials["api_secret"],
        "testnet": False,  # Use production API with real credentials
        "ad_params": {
            "side": "0",  # Buy USDT for RUB
            "currency": "RUB", 
            "price": "90.00",  # Within allowed range 71.42-91.26
            "quantity": "100",  # 100 USDT * 90 RUB = 9000 RUB total
            "min_amount": "900",  # Minimum 900 RUB 
            "max_amount": "5000",
            "payment_methods": ["582"],
            "remarks": "Test ad from Python"
        }
    }

    # Run the script
    result = subprocess.run(
        ["python3", "scripts/bybit_create_ad.py"],
        input=json.dumps(test_data),
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"❌ Script failed: {result.stderr}")
        return False

    # Parse response
    try:
        response = json.loads(result.stdout)
        # Handle both retCode and ret_code formats
        ret_code = response.get("retCode", response.get("ret_code", -999))
        ret_msg = response.get("retMsg", response.get("ret_msg", "Unknown error"))

        if ret_code == 0:
            # Success with P2P API format (uses ret_code instead of retCode)
            result_data = response.get("result", {})
            item_id = result_data.get('itemId', result_data.get('adId', 'Unknown'))
            print(f"✅ Created ad ID: {item_id}")
            print(f"✅ Status: {result_data.get('status', 'online')}")
            if result_data.get('needSecurityRisk'):
                print(f"⚠️  Security risk check required")
            return True
        else:
            print(f"❌ API error code {ret_code}: {ret_msg}")
            return False
    except json.JSONDecodeError as e:
        print(f"❌ Failed to parse response: {e}")
        print(f"Response stdout: {result.stdout}")
        print(f"Response stderr: {result.stderr}")
        return False

def main():
    """Run all tests"""
    print("🚀 Bybit Python Bridge Tests")
    print("=" * 40)

    all_passed = True

    # Test rate fetching
    if not test_get_rates():
        all_passed = False

    # Test ad creation
    if not test_create_ad():
        all_passed = False

    print("\n" + "=" * 40)
    if all_passed:
        print("✅ All tests passed!")
        return 0
    else:
        print("❌ Some tests failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())
