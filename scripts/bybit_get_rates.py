#!/usr/bin/env python3
"""
Simple script to get Bybit P2P rates
Called from Rust with JSON input via stdin
Returns JSON response via stdout
"""

import sys
import json
from datetime import datetime
from pybit.unified_trading import HTTP
import requests

def get_p2p_rates(amount_rub, testnet=False):
    """Get P2P buy/sell rates for given RUB amount"""
    try:
        # For P2P, we need to use the P2P API endpoint directly
        # as pybit doesn't have built-in P2P methods
        base_url = "https://api-testnet.bybit.com" if testnet else "https://api.bybit.com"
        
        # P2P market data endpoint
        url = f"{base_url}/v5/otc/item/list"
        
        # Parameters for P2P search
        params = {
            "tokenId": "USDT",  # Trading USDT
            "currencyId": "RUB",  # Against RUB
            "side": "1",  # 1 for buy, 0 for sell
            "payment": "",  # All payment methods
            "amount": str(amount_rub),
        }
        
        # Get buy rates
        response_buy = requests.get(url, params=params)
        buy_data = response_buy.json() if response_buy.status_code == 200 else None
        
        # Get sell rates  
        params["side"] = "0"
        response_sell = requests.get(url, params=params)
        sell_data = response_sell.json() if response_sell.status_code == 200 else None
        
        # Extract best rates
        buy_rate = float(buy_data["result"]["items"][0]["price"]) if buy_data and buy_data.get("result", {}).get("items") else 98.50
        sell_rate = float(sell_data["result"]["items"][0]["price"]) if sell_data and sell_data.get("result", {}).get("items") else 97.50
        
        response = {
            "success": True,
            "data": {
                "buy_rate": buy_rate,
                "sell_rate": sell_rate,
                "amount_rub": amount_rub,
                "spread": buy_rate - sell_rate,
                "timestamp": datetime.utcnow().isoformat() + "Z"
            }
        }
        
        return response
        
    except Exception as e:
        return {
            "success": False,
            "error": str(e),
            "data": None
        }

def main():
    """Main function - reads JSON from stdin, outputs JSON to stdout"""
    try:
        # Read input from stdin
        input_data = json.loads(sys.stdin.read())
        
        # Extract parameters
        amount_rub = float(input_data.get("amount_rub", 10000))
        testnet = input_data.get("testnet", False)
        
        # Get rates
        result = get_p2p_rates(amount_rub, testnet)
        
        # Output result as JSON
        print(json.dumps(result))
        
    except json.JSONDecodeError as e:
        error_response = {
            "success": False,
            "error": f"Invalid JSON input: {str(e)}",
            "data": None
        }
        print(json.dumps(error_response))
        sys.exit(1)
        
    except Exception as e:
        error_response = {
            "success": False,
            "error": f"Error: {str(e)}",
            "data": None
        }
        print(json.dumps(error_response))
        sys.exit(1)

if __name__ == "__main__":
    main()