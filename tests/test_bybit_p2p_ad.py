#!/usr/bin/env python3
"""
Test script for Bybit P2P ad creation
Uses PostgreSQL database
"""

import asyncio
import json
import os
import sys
from datetime import datetime
import logging
import random
import psycopg2
from psycopg2.extras import RealDictCursor

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Database connection
DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://postgres:root@localhost/itrader")

class PostgresDBManager:
    """PostgreSQL database manager for testing"""
    def __init__(self):
        self.conn = None
        
    def connect(self):
        """Connect to database"""
        self.conn = psycopg2.connect(DATABASE_URL)
        
    def close(self):
        """Close database connection"""
        if self.conn:
            self.conn.close()
            
    def get_bybit_accounts(self):
        """Get all available Bybit accounts"""
        self.connect()
        try:
            with self.conn.cursor(cursor_factory=RealDictCursor) as cursor:
                cursor.execute("""
                    SELECT id, account_name, api_key, api_secret, active_ads, status 
                    FROM bybit_accounts 
                    WHERE status = 'available'
                    ORDER BY id
                """)
                return cursor.fetchall()
        finally:
            self.close()
            
    def update_account_ads(self, account_id, increment=1):
        """Update account's active ads count"""
        self.connect()
        try:
            with self.conn.cursor() as cursor:
                cursor.execute("""
                    UPDATE bybit_accounts 
                    SET active_ads = active_ads + %s, last_used = %s
                    WHERE id = %s
                """, (increment, datetime.now(), account_id))
                self.conn.commit()
        finally:
            self.close()

async def create_test_ad(account):
    """Create a test P2P ad"""
    logger.info(f"Creating ad for account: {account['account_name']}")
    
    print("\n=== Create P2P Ad ===")
    print("1. Buy USDT (покупка)")
    print("2. Sell USDT (продажа)")
    print("3. Random test ad")
    
    choice = input("\nSelect type: ")
    
    if choice == "3":
        # Random ad
        ad_type = random.choice(["buy", "sell"])
        price = round(random.uniform(95, 105), 2)
        quantity = random.choice([100, 500, 1000, 2000])
        
        ad_data = {
            "side": "1" if ad_type == "buy" else "0",
            "asset": "USDT",
            "fiatCurrency": "RUB",
            "price": str(price),
            "quantity": str(quantity),
            "minAmount": "1000",
            "maxAmount": str(int(quantity * price / 2)),
            "paymentMethods": ["582"],  # Tinkoff
            "remarks": f"Test {ad_type} ad - {datetime.now().strftime('%H:%M:%S')}"
        }
    else:
        # Manual input
        ad_type = "buy" if choice == "1" else "sell"
        
        print(f"\n=== {ad_type.upper()} USDT ===")
        price = input("Price per USDT (RUB) [100.00]: ") or "100.00"
        quantity = input("Total USDT quantity [1000]: ") or "1000"
        min_amount = input("Min order (RUB) [1000]: ") or "1000"
        max_amount = input("Max order (RUB) [50000]: ") or "50000"
        
        # Payment methods
        print("\nPayment methods:")
        print("1. Tinkoff (582)")
        print("2. Sberbank (583)")
        print("3. Alfa Bank (584)")
        payment_choice = input("Select payment method [1]: ") or "1"
        payment_map = {"1": "582", "2": "583", "3": "584"}
        payment_method = payment_map.get(payment_choice, "582")
        
        ad_data = {
            "side": "1" if ad_type == "buy" else "0",
            "asset": "USDT",
            "fiatCurrency": "RUB",
            "price": price,
            "quantity": quantity,
            "minAmount": min_amount,
            "maxAmount": max_amount,
            "paymentMethods": [payment_method],
            "remarks": input("Ad description: ") or f"{ad_type.capitalize()} USDT quickly"
        }
    
    # Display ad summary
    print(f"\n📋 Ad Summary:")
    print(f"Type: {ad_type.upper()} {ad_data['asset']}")
    print(f"Price: {ad_data['price']} {ad_data['fiatCurrency']}")
    print(f"Quantity: {ad_data['quantity']} {ad_data['asset']}")
    print(f"Order limits: {ad_data['minAmount']}-{ad_data['maxAmount']} {ad_data['fiatCurrency']}")
    print(f"Payment: {ad_data['paymentMethods']}")
    print(f"Description: {ad_data['remarks']}")
    
    confirm = input("\nCreate this ad? (y/n): ")
    if confirm.lower() != 'y':
        print("❌ Cancelled")
        return False
    
    # Simulate ad creation
    print("\n🔄 Creating ad...")
    await asyncio.sleep(1)  # Simulate API call
    
    # Generate fake ad ID
    ad_id = f"AD_{account['id']}_{datetime.now().strftime('%Y%m%d%H%M%S')}"
    
    print(f"\n✅ Ad created successfully!")
    print(f"Ad ID: {ad_id}")
    print(f"Status: ACTIVE")
    
    # Log the request data
    logger.info(f"Ad request data: {json.dumps(ad_data, indent=2, ensure_ascii=False)}")
    
    return True

async def list_test_ads():
    """List simulated ads for all accounts"""
    db = PostgresDBManager()
    accounts = db.get_bybit_accounts()
    
    print("\n=== Active P2P Ads (Simulated) ===")
    
    for account in accounts:
        if account['active_ads'] > 0:
            print(f"\n📋 {account['account_name']}:")
            # Simulate some ads
            for i in range(min(account['active_ads'], 3)):
                ad_type = random.choice(["BUY", "SELL"])
                price = round(random.uniform(95, 105), 2)
                print(f"  - {ad_type} USDT @ {price} RUB (Active)")
    
    if not any(acc['active_ads'] > 0 for acc in accounts):
        print("\n❌ No active ads found")

async def main():
    """Main test function"""
    print("🚀 Bybit P2P Ad Creation Test")
    print("=" * 40)
    
    # Initialize database manager
    db = PostgresDBManager()
    
    try:
        # Get available accounts
        accounts = db.get_bybit_accounts()
        
        if not accounts:
            print("\n❌ No Bybit accounts found!")
            print("\nTo add accounts:")
            print("1. Run: ./run.sh --settings")
            print("2. Select option 6 (Add Bybit account)")
            return
        
        while True:
            print("\n=== Test Menu ===")
            print("1. Create P2P ad")
            print("2. List active ads (simulated)")
            print("3. Show account stats")
            print("0. Exit")
            
            choice = input("\nSelect option: ")
            
            if choice == "0":
                break
            elif choice == "1":
                # Show accounts
                print("\n=== Available Accounts ===")
                for i, acc in enumerate(accounts, 1):
                    print(f"{i}. {acc['account_name']} (Active ads: {acc['active_ads']})")
                
                # Select account
                try:
                    acc_choice = int(input("\nSelect account number: ")) - 1
                    if acc_choice < 0 or acc_choice >= len(accounts):
                        raise ValueError
                    account = accounts[acc_choice]
                except (ValueError, IndexError):
                    print("❌ Invalid selection")
                    continue
                
                print(f"\n✅ Selected: {account['account_name']}")
                
                # Create ad
                success = await create_test_ad(account)
                
                if success:
                    # Update database
                    db.update_account_ads(account['id'], 1)
                    account['active_ads'] += 1  # Update local copy
                    print(f"\n📊 Account {account['account_name']} now has {account['active_ads']} active ads")
                    
            elif choice == "2":
                await list_test_ads()
                
            elif choice == "3":
                print("\n=== Account Statistics ===")
                total_ads = sum(acc['active_ads'] for acc in accounts)
                print(f"Total accounts: {len(accounts)}")
                print(f"Total active ads: {total_ads}")
                for acc in accounts:
                    print(f"  - {acc['account_name']}: {acc['active_ads']} ads")
            
            input("\nPress Enter to continue...")
    
    except psycopg2.Error as e:
        logger.error(f"Database error: {e}")
        print(f"\n❌ Database error: {e}")
        print("\nMake sure PostgreSQL is running and configured correctly")
    except Exception as e:
        logger.error(f"Test failed: {e}", exc_info=True)
        print(f"\n❌ Test failed: {e}")
    
    print("\n✅ Test completed!")

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n👋 Test stopped")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        logger.error(f"Test failed: {e}", exc_info=True)