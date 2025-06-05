#!/usr/bin/env python3
"""
Bybit P2P Ad Creation Script
Uses correct P2P API endpoints with POST requests
Based on official Bybit P2P API documentation
"""

import sys
import json
import time
import hmac
import hashlib
import requests
from datetime import datetime

def create_ad(api_key, api_secret, ad_params, testnet=False):
    """Create P2P ad on Bybit using correct P2P API"""
    try:
        # Импортируем умный создатель объявлений
        from bybit_smart_ad_creator import SmartAdCreator
        
        # Если включен умный режим, используем SmartAdCreator
        if ad_params.get("smart_mode", False):
            creator = SmartAdCreator(api_key, api_secret, testnet)
            return creator.create_smart_ad(ad_params)
        
        # Иначе используем обычную логику
        # Base URL for API
        base_url = "https://api-testnet.bybit.com" if testnet else "https://api.bybit.com"
        endpoint = "/v5/p2p/item/create"
        url = base_url + endpoint

        # Required parameters for P2P ad creation based on official docs
        params = {
            "tokenId": ad_params.get("tokenId", "USDT"),
            "currencyId": ad_params.get("currency", ad_params.get("currencyId", "RUB")),
            "side": str(ad_params.get("side", "0")),  # "0" for buy, "1" for sell
            "priceType": "0",  # Fixed rate
            "premium": "",  # Empty for fixed rate
            "price": str(ad_params.get("price", "98.50")),
            "minAmount": str(ad_params.get("min_amount", ad_params.get("minAmount", "1000"))),
            "maxAmount": str(ad_params.get("max_amount", ad_params.get("maxAmount", "100000"))),
            "remark": ad_params.get("remarks", ad_params.get("remark", "Fast trade")),
            "tradingPreferenceSet": ad_params.get("tradingPreferenceSet", {
                "hasUnPostAd": "0",
                "isKyc": "0",
                "isEmail": "0", 
                "isMobile": "0",
                "hasRegisterTime": "0",
                "registerTimeThreshold": "0",
                "orderFinishNumberDay30": "0",
                "completeRateDay30": "0",
                "nationalLimit": "",
                "hasOrderFinishNumberDay30": "0",
                "hasCompleteRateDay30": "0",
                "hasNationalLimit": "0"
            }),
            "paymentIds": ad_params.get("payment_methods", ad_params.get("paymentIds", ["582"])),  # Array of payment method IDs
            "quantity": str(ad_params.get("quantity", "10")),
            "paymentPeriod": str(ad_params.get("paymentPeriod", "15")),  # 15 minutes payment period
            "itemType": ad_params.get("itemType", "ORIGIN")  # Original P2P advertisement
        }

        # Generate authentication headers according to Bybit v5 API docs
        timestamp = str(int(time.time() * 1000))
        recv_window = "5000"
        
        # For POST requests with JSON body: timestamp + api_key + recv_window + json_body
        param_str = json.dumps(params, separators=(',', ':'), sort_keys=True)
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

        # Make POST request with raw string data (not json parameter)
        response = requests.post(url, data=param_str, headers=headers)

        # Debug response
        if response.status_code != 200:
            return {
                "ret_code": -1,
                "ret_msg": f"HTTP {response.status_code}: {response.text}",
                "result": None
            }

        # Parse response
        try:
            result = response.json()
            
            # P2P API uses ret_code instead of retCode
            ret_code = result.get("ret_code", -1)
            ret_msg = result.get("ret_msg", "Unknown error")
            
            if ret_code == 0:
                # Success response
                result_data = result.get("result", {})
                return {
                    "ret_code": 0,
                    "ret_msg": "SUCCESS",
                    "result": {
                        "itemId": result_data.get("itemId", f"AD_{int(time.time())}"),
                        "status": "online",
                        "createdTime": datetime.utcnow().isoformat() + "Z",
                        "securityRiskToken": result_data.get("securityRiskToken", ""),
                        "needSecurityRisk": result_data.get("needSecurityRisk", False)
                    },
                    "ext_code": result.get("ext_code", ""),
                    "ext_info": result.get("ext_info", {}),
                    "time_now": result.get("time_now", str(time.time()))
                }
            else:
                # Error response
                return {
                    "ret_code": ret_code,
                    "ret_msg": ret_msg,
                    "result": result.get("result"),
                    "ext_code": result.get("ext_code", ""),
                    "ext_info": result.get("ext_info", {})
                }
                
        except Exception as e:
            return {
                "ret_code": -1,
                "ret_msg": f"JSON parse error: {str(e)}",
                "result": None
            }

    except Exception as e:
        return {
            "ret_code": -1,
            "ret_msg": f"Error creating ad: {str(e)}",
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
            "ret_code": -1,
            "ret_msg": f"Script error: {str(e)}",
            "result": None
        }
        print(json.dumps(error_result))

if __name__ == "__main__":
    main()