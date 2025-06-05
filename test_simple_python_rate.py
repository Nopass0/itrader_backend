#!/usr/bin/env python3
"""
Simple test to check Python rate fetching directly
"""

import sys
import os

# Add the project directory to Python path
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from python_modules.bybit_wrapper import BybitP2PWrapper

# Test direct rate fetching
client = BybitP2PWrapper("dummy_key", "dummy_secret", testnet=False)

params = {
    "token_id": "USDT",
    "currency_id": "RUB",
    "side": 0,  # Buy
    "payment": ["382", "75"],  # СБП and Тинькофф
    "page": 4,
    "size": 10
}

print("Fetching P2P rates...")
result = client.fetch_p2p_rates(params)

if result.get('success'):
    items = result.get('result', {}).get('items', [])
    print(f"Found {len(items)} items")
    if len(items) >= 2:
        rate = float(items[-2]['price'])
        print(f"Penultimate rate: {rate} RUB/USDT")
else:
    print(f"Error: {result.get('error')}")