#!/usr/bin/env python3
"""
Test script to check account balance
"""

import json
import time
import hmac
import hashlib
import requests

def check_balance():
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    api_key = credentials["api_key"]
    api_secret = credentials["api_secret"]
    
    base_url = "https://api.bybit.com"
    endpoint = "/v5/account/wallet-balance"
    url = base_url + endpoint
    
    # Check different account types
    account_types = ["UNIFIED", "FUND", "SPOT"]
    
    for account_type in account_types:
        print(f"\n🔍 Checking {account_type} account balance...")
        
        # Generate authentication
        timestamp = str(int(time.time() * 1000))
        recv_window = "5000"
        query_string = f"accountType={account_type}"
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
        
        params = {"accountType": account_type}
        response = requests.get(url, params=params, headers=headers, timeout=10)
        
        if response.status_code == 200:
            result = response.json()
            if result.get("retCode") == 0:
                list_data = result.get("result", {}).get("list", [])
                if list_data:
                    for account in list_data:
                        coins = account.get("coin", [])
                        usdt_balance = None
                        for coin in coins:
                            if coin.get("coin") == "USDT":
                                usdt_balance = float(coin.get("walletBalance", "0"))
                                break
                        
                        if usdt_balance and usdt_balance > 0:
                            print(f"✅ USDT Balance in {account_type}: {usdt_balance}")
                        else:
                            print(f"❌ No USDT balance in {account_type}")
                else:
                    print(f"❌ No accounts found for {account_type}")
            else:
                print(f"❌ API Error {result.get('retCode')}: {result.get('retMsg')}")
        else:
            print(f"❌ HTTP {response.status_code}: {response.text}")

if __name__ == "__main__":
    check_balance()