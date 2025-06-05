#!/usr/bin/env python3
"""
Test script for Bybit Python SDK P2P rate fetching
"""

import sys
import os
import json
import asyncio

# Add the project directory to Python path
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from python_modules.bybit_wrapper import BybitP2PWrapper


async def test_p2p_rates():
    """Test P2P rate fetching using the official SDK"""
    
    # Initialize client (using dummy keys for public endpoints)
    client = BybitP2PWrapper("dummy_key", "dummy_secret", testnet=False)
    
    print("Testing P2P Rate Fetching with Official Bybit SDK")
    print("=" * 50)
    
    # Test different scenarios
    test_scenarios = [
        {"name": "Small Amount Day (Page 4)", "page": 4},
        {"name": "Small Amount Night (Page 2)", "page": 2},
        {"name": "Large Amount Day (Page 5)", "page": 5},
        {"name": "Large Amount Night (Page 3)", "page": 3},
    ]
    
    for scenario in test_scenarios:
        print(f"\n{scenario['name']}:")
        
        params = {
            "token_id": "USDT",
            "currency_id": "RUB",
            "side": 0,  # Buy
            "payment": ["382", "75"],  # СБП and Тинькофф
            "page": scenario['page'],
            "size": 10
        }
        
        try:
            result = client.fetch_p2p_rates(params)
            
            if result.get('success'):
                items = result.get('result', {}).get('items', [])
                print(f"  Found {len(items)} listings")
                
                if len(items) >= 2:
                    # Get penultimate price
                    penultimate = items[-2]
                    print(f"  Penultimate price: {penultimate['price']} RUB")
                    print(f"  Trader: {penultimate['nickName']}")
                    print(f"  Available: {penultimate['lastQuantity']} USDT")
                else:
                    print(f"  Not enough items found")
                    
            else:
                print(f"  Error: {result.get('error')}")
                
        except Exception as e:
            print(f"  Exception: {e}")
    
    print("\nTesting direct rate fetch for 30,000 RUB at current time:")
    try:
        # Determine current scenario based on Moscow time
        from datetime import datetime
        import pytz
        
        moscow_tz = pytz.timezone('Europe/Moscow')
        moscow_time = datetime.now(moscow_tz)
        hour = moscow_time.hour
        
        is_night = 1 <= hour < 7
        is_small_amount = True  # 30k RUB < 50k
        
        if is_small_amount and not is_night:
            page = 4  # Small amount day
            scenario_name = "Small Amount Day"
        elif is_small_amount and is_night:
            page = 2  # Small amount night
            scenario_name = "Small Amount Night"
        else:
            page = 5  # Would be large amount day
            scenario_name = "Large Amount Day"
            
        print(f"  Moscow time: {moscow_time.strftime('%H:%M:%S')}")
        print(f"  Scenario: {scenario_name} (Page {page})")
        
        params = {
            "token_id": "USDT",
            "currency_id": "RUB",
            "side": 0,
            "payment": ["382", "75"],
            "page": page,
            "size": 10
        }
        
        result = client.fetch_p2p_rates(params)
        if result.get('success'):
            items = result.get('result', {}).get('items', [])
            if len(items) >= 2:
                rate = float(items[-2]['price'])
                print(f"  Current rate: {rate} RUB/USDT")
    except Exception as e:
        print(f"  Error determining current rate: {e}")


if __name__ == "__main__":
    asyncio.run(test_p2p_rates())