#!/usr/bin/env python3
"""
Test script for smart P2P ad creation
"""

import json
import subprocess

def test_smart_ad_creation():
    """Test smart ad creation with payment method logic"""
    print("🚀 Testing Smart P2P Ad Creator")
    print("=" * 50)
    
    # Load credentials
    try:
        with open("test_data/bybit_creditials.json", "r") as f:
            credentials = json.load(f)
    except FileNotFoundError:
        print("❌ No credentials file found at test_data/bybit_creditials.json")
        return False
    
    # Test parameters
    test_data = {
        "api_key": credentials["api_key"],
        "api_secret": credentials["api_secret"],
        "testnet": False,  # Use production API
        "ad_params": {
            "amount": "10000",  # Сумма транзакции в рублях
            "remark": "Быстрая продажа USDT. Безопасная сделка"
        }
    }
    
    print("📋 Test parameters:")
    print(f"  Type: Sell USDT (всегда продажа)")
    print(f"  Transaction amount: {test_data['ad_params']['amount']} RUB")
    print(f"  Payment period: 15 minutes")
    print()
    
    # Run the smart ad creator
    result = subprocess.run(
        ["python3", "scripts/bybit_smart_ad_creator.py"],
        input=json.dumps(test_data),
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"❌ Script failed with error: {result.stderr}")
        return False
    
    # Parse output - split by lines and get the last line (JSON result)
    output_lines = result.stdout.strip().split('\n')
    
    # Print debug output
    for line in output_lines[:-1]:
        print(line)
    
    # Parse JSON result from last line
    try:
        response = json.loads(output_lines[-1])
        ret_code = response.get("ret_code", -1)
        ret_msg = response.get("ret_msg", "Unknown error")
        
        print("\n📊 Result:")
        if ret_code == 0:
            result_data = response.get("result", {})
            print(f"✅ Success! Ad ID: {result_data.get('itemId', 'Unknown')}")
            if result_data.get('needSecurityRisk'):
                print(f"⚠️  Security risk check required")
            return True
        else:
            print(f"❌ Failed with code {ret_code}: {ret_msg}")
            # Provide helpful hints based on error code
            if ret_code == 912120022:
                print("💡 Hint: Price is outside allowed range")
            elif ret_code == 912120023:
                print("💡 Hint: Min amount must be <= total amount (price * quantity)")
            elif ret_code == 912300013:
                print("💡 Hint: Payment method not found or not configured")
            elif ret_code == 912300015:
                print("💡 Hint: Minimum amount parameter error")
            elif ret_code == 10001:
                print("💡 Hint: Request parameter error - check all required fields")
            return False
            
    except json.JSONDecodeError as e:
        print(f"❌ Failed to parse response: {e}")
        print(f"Raw output: {output_lines[-1] if output_lines else 'No output'}")
        return False
    except IndexError:
        print("❌ No output from script")
        return False

def test_get_active_ads():
    """Test getting active ads directly"""
    print("\n🔍 Testing Get Active Ads")
    print("=" * 50)
    
    import sys
    sys.path.append('scripts')
    from bybit_smart_ad_creator import SmartAdCreator
    
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    creator = SmartAdCreator(credentials["api_key"], credentials["api_secret"], testnet=False)
    
    # Get active ads
    active_ads = creator.get_active_ads()
    print(f"📊 Found {len(active_ads)} active ads")
    
    for i, ad in enumerate(active_ads):
        print(f"\n📌 Ad #{i+1}:")
        print(f"  ID: {ad.get('id')}")
        print(f"  Side: {'Buy' if ad.get('side') == 0 else 'Sell'}")
        print(f"  Price: {ad.get('price')} RUB/USDT")
        print(f"  Quantity: {ad.get('quantity')} USDT")
        print(f"  Frozen: {ad.get('frozenQuantity')} USDT")
        print(f"  Payment methods: {ad.get('payments', [])}")
        
        # Показываем название метода оплаты
        payment_terms = ad.get('paymentTerms', [])
        for term in payment_terms:
            payment_type = term.get('paymentType')
            payment_config = term.get('paymentConfig', {})
            payment_name = payment_config.get('paymentName', 'Unknown')
            print(f"    - {payment_name} (type: {payment_type})")
    
    # Get user payment methods
    print("\n💳 User Payment Methods:")
    user_methods = creator.get_user_payment_methods()
    
    for method in user_methods:
        payment_id = method.get('id')
        payment_config = method.get('paymentConfigVo', {})
        payment_name = payment_config.get('paymentName', 'Unknown')
        payment_type = method.get('paymentType')
        print(f"  - ID: {payment_id}, Name: {payment_name}, Type: {payment_type}")

if __name__ == "__main__":
    # First check what ads and payment methods we have
    test_get_active_ads()
    
    # Then test smart ad creation
    test_smart_ad_creation()