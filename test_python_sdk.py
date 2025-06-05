#!/usr/bin/env python3
"""
Quick test of the Python Bybit wrapper
"""

import os
import json
from python_modules.bybit_wrapper import BybitP2PWrapper

def test_wrapper():
    # Load credentials
    cred_file = os.getenv("BYBIT_CREDENTIALS_FILE", "test_data/bybit_creditials.json")
    
    try:
        with open(cred_file, 'r') as f:
            creds = json.load(f)
    except FileNotFoundError:
        print(f"Credentials file not found: {cred_file}")
        print("Using dummy credentials for test")
        creds = {"api_key": "test_key", "api_secret": "test_secret"}
    
    # Determine if testnet
    testnet = "testnet" in cred_file
    
    print(f"Testing Bybit Python wrapper (testnet: {testnet})")
    print(f"Using credentials from: {cred_file}")
    
    # Create wrapper
    try:
        wrapper = BybitP2PWrapper(
            api_key=creds.get("api_key", ""),
            api_secret=creds.get("api_secret", ""),
            testnet=testnet
        )
        print("✓ Wrapper created successfully")
        
        # Test server time
        try:
            server_time = wrapper.get_server_time()
            print(f"✓ Server time: {server_time}")
        except Exception as e:
            print(f"✗ Failed to get server time: {e}")
        
        # Test account info
        try:
            account = wrapper.get_account_info()
            print(f"✓ Account info: {account}")
        except Exception as e:
            print(f"✗ Failed to get account info: {e}")
            
    except Exception as e:
        print(f"✗ Failed to create wrapper: {e}")

if __name__ == "__main__":
    test_wrapper()