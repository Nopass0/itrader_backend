#!/usr/bin/env python3
"""
Simple script to create Bybit P2P ad
Called from Rust tests with JSON input via stdin
Returns JSON response via stdout
"""

import sys
import json
import os
import time
import hmac
import hashlib
import requests
from datetime import datetime

def create_ad(api_key, api_secret, ad_params, testnet=False):
    """Create P2P ad on Bybit"""
    try:
        # Base URL for API
        base_url = "https://api-testnet.bybit.com" if testnet else "https://api.bybit.com"
        endpoint = "/v5/p2p/item/create"
        url = base_url + endpoint

        # Required parameters for P2P ad creation
        params = {
            "tokenId": ad_params.get("tokenId", "USDT"),
            "currencyId": ad_params.get("currency", ad_params.get("currencyId", "RUB")),
            "side": str(ad_params.get("side", "0")),  # "0" for buy, "1" for sell
            "priceType": "0",  # Fixed rate
            "premium": "",
            "price": str(ad_params.get("price", "98.50")),
            "minAmount": str(ad_params.get("min_amount", ad_params.get("minAmount", "1000"))),
            "maxAmount": str(ad_params.get("max_amount", ad_params.get("maxAmount", "100000"))),
            "remark": ad_params.get("remarks", ad_params.get("remark", "Fast trade")),
            "tradingPreferenceSet": {
                "hasUnPostAd": 0,
                "isKyc": 0,
                "isEmail": 0,
                "isMobile": 0,
                "hasRegisterTime": 0,
                "registerTimeThreshold": 0,
                "orderFinishNumberDay30": 0,
                "completeRateDay30": "0",
                "nationalLimit": "",
                "hasOrderFinishNumberDay30": 0,
                "hasCompleteRateDay30": 0,
                "hasNationalLimit": 0
            },
            "paymentIds": ad_params.get("payment_methods", ["582"]),
            "quantity": str(ad_params.get("quantity", "10")),
            "paymentPeriod": "15",
            "itemType": "ORIGIN"
        }

        # Generate authentication headers according to Bybit v5 API docs
        timestamp = str(int(time.time() * 1000))
        recv_window = "5000"
        
        # For POST requests with JSON body: timestamp + api_key + recv_window + json_body
        param_str = json.dumps(params, separators=(',', ':'))
        sign_str = timestamp + api_key + recv_window + param_str
        
        signature = hmac.new(
            api_secret.encode('utf-8'),
            sign_str.encode('utf-8'),
            hashlib.sha256
        ).hexdigest()

        # Headers according to Bybit v5 API specification
        headers = {
            "X-BAPI-API-KEY": api_key,
            "X-BAPI-TIMESTAMP": timestamp,
            "X-BAPI-SIGN": signature,
            "X-BAPI-RECV-WINDOW": recv_window,
            "Content-Type": "application/json"
        }

        # Make request
        response = requests.post(url, json=params, headers=headers)

        # Debug response
        if response.status_code != 200:
            return {
                "retCode": -1,
                "retMsg": f"HTTP {response.status_code}: {response.text}",
                "result": None
            }

        # Parse response
        try:
            result = response.json()
            # Check if it was successful
            if result.get("retCode") == 0:
                return {
                    "retCode": 0,
                    "retMsg": "OK",
                    "result": {
                        "adId": result.get("result", {}).get("itemId", f"AD_{int(time.time())}"),
                        "status": "online",
                        "createdTime": datetime.utcnow().isoformat() + "Z"
                    },
                    "retExtInfo": result.get("retExtInfo", {}),
                    "time": result.get("time", int(time.time() * 1000))
                }
            else:
                return result
        except Exception as e:
            return {
                "retCode": -1,
                "retMsg": f"JSON parse error: {str(e)}",
                "result": None
            }

    except Exception as e:
        return {
            "retCode": -1,
            "retMsg": f"Error creating ad: {str(e)}",
            "result": None
        }

def main():
    try:
        # Read JSON input from stdin
        input_data = sys.stdin.read()
        data = json.loads(input_data)
        
        # Extract parameters
        api_key = data.get("api_key", "")
        api_secret = data.get("api_secret", "")
        ad_params = data.get("ad_params", {})
        testnet = data.get("testnet", False)
        
        # Create the ad
        result = create_ad(api_key, api_secret, ad_params, testnet)
        
        # Output result as JSON
        print(json.dumps(result))
        
    except Exception as e:
        error_result = {
            "retCode": -1,
            "retMsg": f"Script error: {str(e)}",
            "result": None
        }
        print(json.dumps(error_result))

if __name__ == "__main__":
    main()