#!/usr/bin/env python3
import json

params = {
    "tokenId": "USDT",
    "currencyId": "RUB",
    "side": "0",
    "page": "1",
    "size": "5"
}

# Test different JSON formatting
print("Default json.dumps:")
print(json.dumps(params))

print("\nWith separators=(',', ':'):")
print(json.dumps(params, separators=(',', ':')))

print("\nBybit expects:")
print('{"tokenId":"USDT","currencyId":"RUB","side":"0","page":"1","size":"5"}')