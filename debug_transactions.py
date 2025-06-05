#!/usr/bin/env python3
import json
import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from gate_client import GateClient

def main():
    # Load credentials
    with open('db/gate/RuJnDK0z-Ha3jb1TDK8e1Q.json', 'r') as f:
        account_data = json.load(f)
    
    print(f"Using account: {account_data['login']}")
    
    # Create client
    client = GateClient()
    
    # Login
    print("Logging in...")
    if client.login(account_data['login'], account_data['password']):
        print("Login successful!")
        
        # Get available transactions
        print("\nFetching available transactions...")
        transactions = client.get_available_transactions()
        
        if transactions:
            print(f"\nFound {len(transactions)} available transactions:")
            for tx in transactions:
                print(f"- ID: {tx['id']}, Status: {tx['status']}, Amount: {tx.get('amount', {}).get('trader', {})}")
        else:
            print("No available transactions found")
            
        # Get all transactions to see what's there
        print("\nFetching all transactions...")
        all_tx = client.get_transactions(page=1)
        if all_tx:
            print(f"\nFound {len(all_tx)} total transactions:")
            for tx in all_tx[:5]:  # Show first 5
                print(f"- ID: {tx['id']}, Status: {tx['status']}")
    else:
        print("Login failed!")

if __name__ == "__main__":
    main()