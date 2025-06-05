#!/usr/bin/env python3
"""Test P2P rates directly"""

import sys
sys.path.insert(0, 'scripts')

import json
from bybit_get_rates import get_authenticated_rates

# Load credentials
with open("test_data/bybit_creditials.json", "r") as f:
    creds = json.load(f)

# Test
result = get_authenticated_rates(
    creds["api_key"],
    creds["api_secret"],
    amount_rub=10000,
    testnet=False
)

print(json.dumps(result, indent=2))