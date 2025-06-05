#!/usr/bin/env python3
"""
Test Bybit operations through Python bridge
Uses PyO3 to call Rust functions that use Python SDK
"""

import subprocess
import json
import os
import sys
from datetime import datetime

# Test creating P2P ad through Python bridge
def test_create_ad_python_bridge():
    """Test creating Bybit P2P ad through Python SDK bridge"""
    print("\n=== Testing Bybit P2P Ad Creation (Python Bridge) ===")
    
    # This would normally call the Rust binary that uses Python bridge
    # For now, we'll simulate the test
    test_data = {
        "account_id": 1,
        "ad_type": "sell",
        "price": "98.50",
        "quantity": "100",
        "min_amount": "1000",
        "max_amount": "10000",
        "payment_methods": ["582"],  # Tinkoff
        "remarks": "Test ad from Python bridge"
    }
    
    print(f"Test data: {json.dumps(test_data, indent=2)}")
    
    # Run Rust test that uses Python bridge
    result = subprocess.run(
        ["cargo", "test", "test_bybit_python_bridge_create_ad", "--", "--nocapture"],
        capture_output=True,
        text=True,
        env={**os.environ, "TEST_AD_DATA": json.dumps(test_data)}
    )
    
    if result.returncode == 0:
        print("✅ Test passed!")
        print(result.stdout)
    else:
        print("❌ Test failed!")
        print("STDOUT:", result.stdout)
        print("STDERR:", result.stderr)
        return False
    
    return True

# Test getting rates through Python bridge
def test_get_rates_python_bridge():
    """Test getting Bybit P2P rates through Python SDK bridge"""
    print("\n=== Testing Bybit Rate Fetching (Python Bridge) ===")
    
    # Run Rust test that uses Python bridge for rates
    result = subprocess.run(
        ["cargo", "test", "test_bybit_python_rate_fetcher", "--", "--nocapture"],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        print("✅ Test passed!")
        print(result.stdout)
    else:
        print("❌ Test failed!")
        print("STDOUT:", result.stdout)
        print("STDERR:", result.stderr)
        return False
    
    return True

# Test account info through Python bridge
def test_account_info_python_bridge():
    """Test getting Bybit account info through Python SDK bridge"""
    print("\n=== Testing Bybit Account Info (Python Bridge) ===")
    
    # Run Rust test
    result = subprocess.run(
        ["cargo", "test", "test_bybit_python_account_info", "--", "--nocapture"],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        print("✅ Test passed!")
        print(result.stdout)
    else:
        print("❌ Test failed!")
        print("STDOUT:", result.stdout)
        print("STDERR:", result.stderr)
        return False
    
    return True

def main():
    """Run all Python bridge tests"""
    print("🐍 Bybit Python Bridge Test Suite")
    print("=" * 50)
    
    # Check if we're in test environment
    db_url = os.getenv("DATABASE_URL", "")
    if "itrader_test" not in db_url:
        print("⚠️  Warning: Not using test database!")
        print(f"Current DATABASE_URL: {db_url}")
    
    all_passed = True
    
    # Run tests
    tests = [
        test_get_rates_python_bridge,
        test_account_info_python_bridge,
        test_create_ad_python_bridge,
    ]
    
    for test in tests:
        try:
            if not test():
                all_passed = False
        except Exception as e:
            print(f"❌ Test {test.__name__} failed with exception: {e}")
            all_passed = False
    
    print("\n" + "=" * 50)
    if all_passed:
        print("✅ All tests passed!")
        return 0
    else:
        print("❌ Some tests failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())