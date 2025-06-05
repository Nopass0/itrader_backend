#!/usr/bin/env python3
"""
Test Bybit authentication directly with pybit library
"""

import json
from pybit.unified_trading import HTTP

def test_with_pybit():
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    
    print(f"Testing with API key: {api_key}")
    
    # Create session
    session = HTTP(
        testnet=False,  # Use production
        api_key=api_key,
        api_secret=api_secret,
    )
    
    try:
        # Test account info first
        print("Testing account info...")
        account_info = session.get_wallet_balance(accountType="UNIFIED")
        print(f"✅ Account info: {account_info}")
        
        # Test if we can get P2P info
        print("\nTesting P2P related endpoints...")
        
        # Try getting account info that might be relevant to P2P
        try:
            # This should work if API key is valid
            coin_balance = session.get_coin_balance(coin="USDT")
            print(f"✅ USDT balance: {coin_balance}")
        except Exception as e:
            print(f"⚠️ Coin balance error: {e}")
        
        # Try any other endpoints that might give us insight
        try:
            # Check if we have any trading permissions
            positions = session.get_positions(category="linear", limit=1)
            print(f"✅ Can access positions: {positions}")
        except Exception as e:
            print(f"⚠️ Positions error: {e}")
            
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    test_with_pybit()