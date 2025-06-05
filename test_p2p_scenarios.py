#!/usr/bin/env python3
"""
Test different P2P scenarios with smart ad creation
"""

import json
import sys
import time

sys.path.append('scripts')
from bybit_smart_ad_creator import SmartAdCreator

def test_scenario(creator, scenario_name, expected_behavior):
    """Test a specific scenario"""
    print(f"\n🧪 Scenario: {scenario_name}")
    print("=" * 60)
    
    # Get current state
    active_ads = creator.get_active_ads()
    print(f"📊 Active ads: {len(active_ads)}")
    
    blocked_payment_types, available_payment_types = creator.analyze_active_ads(active_ads)
    
    # Print analysis
    for i, ad in enumerate(active_ads):
        frozen = float(ad.get('frozenQuantity', '0'))
        payment_types = []
        payment_names = []
        
        for term in ad.get('paymentTerms', []):
            payment_type = str(term.get('paymentType', ''))
            payment_config = term.get('paymentConfig', {})
            payment_name = payment_config.get('paymentName', 'Unknown')
            payment_types.append(payment_type)
            payment_names.append(payment_name)
        
        print(f"\n  Ad #{i+1}:")
        print(f"    ID: {ad.get('id')}")
        print(f"    Payment types: {payment_types} ({', '.join(payment_names)})")
        print(f"    Frozen quantity: {frozen} USDT")
        print(f"    Status: {'Has responses' if frozen > 0 else 'No responses'}")
    
    print(f"\n🚫 Blocked payment types: {blocked_payment_types}")
    print(f"✅ Available payment types: {available_payment_types}")
    print(f"\n📝 Expected: {expected_behavior}")
    
    # Get user methods
    user_methods = creator.get_user_payment_methods()
    
    # Try to select method
    selected = creator.select_payment_method(user_methods, blocked_payment_types, active_ads)
    
    if selected:
        method_name = "Unknown"
        for method in user_methods:
            if method.get('id') == selected:
                config = method.get('paymentConfigVo', {})
                method_name = config.get('paymentName', 'Unknown')
                break
        print(f"✅ Would select: {selected} ({method_name})")
    else:
        print("❌ No suitable payment method available")
    
    return selected

def main():
    print("🚀 P2P Smart Ad Creation - Scenario Testing")
    print("=" * 60)
    
    # Load credentials
    try:
        with open("test_data/bybit_creditials.json", "r") as f:
            credentials = json.load(f)
    except FileNotFoundError:
        print("❌ No credentials file found")
        return
    
    creator = SmartAdCreator(credentials["api_key"], credentials["api_secret"], testnet=False)
    
    # Test current scenario
    print("\n📋 Current Account State:")
    print("-" * 60)
    
    active_ads = creator.get_active_ads()
    user_methods = creator.get_user_payment_methods()
    
    print(f"Active ads: {len(active_ads)}")
    print("Available payment methods:")
    for method in user_methods:
        if method.get('id') != '-1':  # Skip Balance
            config = method.get('paymentConfigVo', {})
            name = config.get('paymentName', 'Unknown')
            print(f"  - {method.get('id')}: {name} (type: {method.get('paymentType')})")
    
    # Scenario 1: Current state
    test_scenario(
        creator, 
        "Current state - 1 ad with SBP and responses",
        "Should select Tinkoff since SBP has responses"
    )
    
    # Scenario 2: What if we had 2 ads without responses
    print("\n" + "="*60)
    print("📌 Hypothetical Scenarios:")
    print("="*60)
    
    print("""
Scenario 2: Two ads without responses
- Ad 1: SBP, no responses (frozen=0)
- Ad 2: Tinkoff, no responses (frozen=0)
Expected: Cannot create new ad (limit reached)

Scenario 3: Two ads, one with response
- Ad 1: SBP, has responses (frozen>0)  
- Ad 2: Tinkoff, no responses (frozen=0)
Expected: Can create new ad with SBP

Scenario 4: One ad without response
- Ad 1: SBP, no responses (frozen=0)
Expected: Should select Tinkoff for new ad
""")
    
    # Show smart logic summary
    print("\n📚 Smart Ad Creation Logic Summary:")
    print("="*60)
    print("""
1. Maximum 2 active ads allowed
2. If ad has responses (frozenQuantity > 0):
   - Its payment method can be reused
3. If ad has no responses (frozenQuantity = 0):
   - Its payment method is blocked for new ads
4. Priority: SPB ↔ Tinkoff (alternate between them)
5. Two ads without responses cannot have same payment method
""")

if __name__ == "__main__":
    main()