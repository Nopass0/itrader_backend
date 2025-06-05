#!/usr/bin/env python3
"""
Test script for P2P Order Manager
"""

import json
import sys
sys.path.append('.')
from scripts.bybit_p2p_order_manager import P2POrderManager

def test_order_manager():
    """Test P2P order management functions"""
    print("🚀 P2P Order Manager Test")
    print("=" * 60)
    
    # Load credentials
    try:
        with open("test_data/bybit_creditials.json", "r") as f:
            credentials = json.load(f)
    except FileNotFoundError:
        print("❌ No credentials file found")
        return
    
    # Create manager
    manager = P2POrderManager(
        api_key=credentials["api_key"],
        api_secret=credentials["api_secret"],
        testnet=False
    )
    
    # 1. Get all orders
    print("\n📋 Getting all orders...")
    orders = manager.get_orders(page=1, size=5)
    print(f"Found {len(orders)} orders")
    
    for order in orders:
        print(manager.format_order_info(order))
    
    # 2. Get pending orders
    print("\n⏳ Getting pending orders...")
    pending_orders = manager.get_pending_orders()
    print(f"Found {len(pending_orders)} pending orders")
    
    # Show orders that need action
    for order in pending_orders:
        status = order.get("status", 0)
        side = order.get("side", 0)
        
        if status == 20 and side == 1:  # Seller waiting to release
            print(f"\n💰 Order {order.get('id')} waiting for asset release")
            print("   Action needed: Release assets")
        elif status == 10 and side == 0:  # Buyer waiting to pay
            print(f"\n💳 Order {order.get('id')} waiting for payment")
            print("   Action needed: Mark as paid")
    
    # 3. Test chat functionality (if there are orders)
    if orders:
        test_order = orders[0]
        order_id = test_order.get("id")
        
        print(f"\n💬 Testing chat for order {order_id}...")
        
        # Get chat messages
        messages = manager.get_chat_messages(order_id)
        print(f"Found {len(messages)} messages")
        
        # Show last 5 messages
        if isinstance(messages, list):
            for msg in messages[-5:]:
                print(manager.format_message(msg))
        else:
            # messages might be a single dict
            print(manager.format_message(messages))
        
        # Example: Send a message (commented out to avoid spamming)
        # success = manager.send_message(
        #     order_id=order_id,
        #     message="Спасибо за сделку! / Thank you for the trade!"
        # )
        # if success:
        #     print("✅ Message sent successfully")
    
    # 4. Show available actions
    print("\n🔧 Available Actions:")
    print("1. List orders: action='list_orders'")
    print("2. Get pending: action='get_pending'")
    print("3. Get chat: action='get_chat', order_id='xxx'")
    print("4. Send message: action='send_message', order_id='xxx', message='text'")
    print("5. Release assets: action='release_assets', order_id='xxx'")
    
    # 5. Show example usage
    print("\n📚 Example Usage:")
    print("""
# List all orders
echo '{
  "api_key": "YOUR_KEY",
  "api_secret": "YOUR_SECRET",
  "action": "list_orders"
}' | python scripts/bybit_p2p_order_manager.py

# Send message
echo '{
  "api_key": "YOUR_KEY",
  "api_secret": "YOUR_SECRET",
  "action": "send_message",
  "order_id": "ORDER_ID",
  "message": "Hello!"
}' | python scripts/bybit_p2p_order_manager.py

# Release assets
echo '{
  "api_key": "YOUR_KEY",
  "api_secret": "YOUR_SECRET",
  "action": "release_assets",
  "order_id": "ORDER_ID"
}' | python scripts/bybit_p2p_order_manager.py
""")

def test_specific_order(order_id: str):
    """Test specific order functionality"""
    print(f"\n🔍 Testing specific order: {order_id}")
    print("=" * 60)
    
    # Load credentials
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    manager = P2POrderManager(
        api_key=credentials["api_key"],
        api_secret=credentials["api_secret"],
        testnet=False
    )
    
    # Get chat messages
    messages = manager.get_chat_messages(order_id)
    print(f"\n💬 Chat messages ({len(messages)} total):")
    
    for msg in messages:
        print(manager.format_message(msg))

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Test specific order
        test_specific_order(sys.argv[1])
    else:
        # General test
        test_order_manager()