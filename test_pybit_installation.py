#!/usr/bin/env python3
"""Test script to verify pybit installation and basic functionality"""

import json
import sys

def test_pybit_import():
    """Test that pybit can be imported successfully"""
    try:
        from pybit.unified_trading import HTTP
        print("✅ Successfully imported pybit")
        return True
    except ImportError as e:
        print(f"❌ Failed to import pybit: {e}")
        return False

def test_bybit_connection():
    """Test basic connection to Bybit API"""
    try:
        from pybit.unified_trading import HTTP
        
        # Create testnet session (no auth needed for public endpoints)
        session = HTTP(testnet=True)
        
        # Test public endpoint - get server time
        result = session.get_server_time()
        
        if result["retCode"] == 0:
            print(f"✅ Successfully connected to Bybit testnet")
            print(f"   Server time: {result['result']['timeSecond']}")
            return True
        else:
            print(f"❌ Failed to connect: {result['retMsg']}")
            return False
            
    except Exception as e:
        print(f"❌ Connection test failed: {e}")
        return False

def test_p2p_endpoint():
    """Test P2P endpoint access"""
    try:
        import requests
        
        # P2P public endpoint
        url = "https://api.bybit.com/v5/otc/item/list"
        params = {
            "tokenId": "USDT",
            "currencyId": "RUB",
            "side": "1",  # Buy
            "limit": "5"
        }
        
        response = requests.get(url, params=params)
        data = response.json()
        
        if response.status_code == 200 and data.get("retCode") == 0:
            print("✅ Successfully accessed P2P endpoint")
            items = data.get("result", {}).get("items", [])
            if items:
                print(f"   Found {len(items)} P2P offers")
                print(f"   Best rate: {items[0].get('price', 'N/A')} RUB/USDT")
            return True
        else:
            print(f"❌ P2P endpoint test failed: {data.get('retMsg', 'Unknown error')}")
            return False
            
    except Exception as e:
        print(f"❌ P2P endpoint test failed: {e}")
        return False

def main():
    """Run all tests"""
    print("🔍 Testing pybit installation and Bybit connectivity...")
    print("-" * 50)
    
    tests_passed = 0
    tests_total = 3
    
    # Test 1: Import
    if test_pybit_import():
        tests_passed += 1
    print()
    
    # Test 2: Connection
    if test_bybit_connection():
        tests_passed += 1
    print()
    
    # Test 3: P2P Endpoint
    if test_p2p_endpoint():
        tests_passed += 1
    print()
    
    # Summary
    print("-" * 50)
    if tests_passed == tests_total:
        print(f"✅ All tests passed ({tests_passed}/{tests_total})")
        return 0
    else:
        print(f"⚠️  Some tests failed ({tests_passed}/{tests_total})")
        return 1

if __name__ == "__main__":
    sys.exit(main())