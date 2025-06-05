#!/usr/bin/env python3
"""
Interactive setup for Auto Trader
Run this after setup_trader.py to configure accounts
"""

import json
import secrets
from pathlib import Path
from getpass import getpass
import sys

def setup_gmail():
    """Setup Gmail credentials"""
    print("\n=== Gmail Setup ===")
    print("Gmail API is used to monitor email receipts from noreply@tinkoff.ru")
    
    choice = input("\nSetup Gmail now? (y/n): ")
    if choice.lower() != 'y':
        print("⚠ Skipping Gmail setup. You can configure it later.")
        return False
    
    print("\nTo use Gmail API:")
    print("1. Go to https://console.cloud.google.com/")
    print("2. Create a new project or select existing")
    print("3. Enable Gmail API")
    print("4. Create OAuth2 credentials (Desktop application)")
    print("5. Download the credentials JSON file")
    
    credentials_path = input("\nEnter path to credentials.json (or 'skip'): ")
    
    if credentials_path.lower() == 'skip':
        return False
    
    if Path(credentials_path).exists():
        try:
            with open(credentials_path, 'r') as f:
                creds = json.load(f)
            
            # Save to db/gmail/
            output_path = Path("db/gmail/credentials.json")
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            with open(output_path, 'w') as f:
                json.dump(creds, f, indent=2)
            
            print(f"✓ Gmail credentials saved to {output_path}")
            return True
        except Exception as e:
            print(f"✗ Error saving credentials: {e}")
            return False
    else:
        print(f"✗ File not found: {credentials_path}")
        return False


def add_gate_account():
    """Add Gate.io account"""
    print("\n=== Add Gate.io Account ===")
    
    login = input("Gate.io email: ")
    password = getpass("Gate.io password: ")
    
    account_id = secrets.token_urlsafe(16)
    account_data = {
        "id": account_id,
        "login": login,
        "password": password,
        "status": "inactive",
        "cookies": None,
        "last_login": None,
        "balance": 0
    }
    
    # Save account
    account_path = Path(f"db/gate/{account_id}.json")
    account_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(account_path, 'w') as f:
        json.dump(account_data, f, indent=2)
    
    print(f"✓ Added Gate.io account: {account_id}")
    return account_id


def add_bybit_account():
    """Add Bybit account"""
    print("\n=== Add Bybit Account ===")
    
    api_key = input("Bybit API key: ")
    api_secret = getpass("Bybit API secret: ")
    testnet = input("Use testnet? (y/n): ").lower() == 'y'
    
    account_id = secrets.token_urlsafe(16)
    account_data = {
        "id": account_id,
        "api_key": api_key,
        "api_secret": api_secret,
        "testnet": testnet,
        "status": "inactive",
        "active_ads": 0
    }
    
    # Save account
    account_path = Path(f"db/bybit/{account_id}.json")
    account_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(account_path, 'w') as f:
        json.dump(account_data, f, indent=2)
    
    print(f"✓ Added Bybit account: {account_id}")
    return account_id


def view_accounts():
    """View all accounts"""
    print("\n=== Current Accounts ===")
    
    # Gate accounts
    gate_dir = Path("db/gate")
    if gate_dir.exists():
        gate_accounts = list(gate_dir.glob("*.json"))
        print(f"\nGate.io accounts: {len(gate_accounts)}")
        for acc_path in gate_accounts:
            try:
                with open(acc_path, 'r') as f:
                    acc = json.load(f)
                print(f"  - {acc['id']}: {acc['login']} [{acc['status']}]")
            except:
                pass
    
    # Bybit accounts
    bybit_dir = Path("db/bybit")
    if bybit_dir.exists():
        bybit_accounts = list(bybit_dir.glob("*.json"))
        print(f"\nBybit accounts: {len(bybit_accounts)}")
        for acc_path in bybit_accounts:
            try:
                with open(acc_path, 'r') as f:
                    acc = json.load(f)
                testnet = " (testnet)" if acc.get('testnet') else ""
                print(f"  - {acc['id']}: {acc['api_key'][:10]}...{testnet} [{acc['status']}]")
            except:
                pass


def view_settings():
    """View current settings"""
    settings_path = Path("db/settings.json")
    if settings_path.exists():
        with open(settings_path, 'r') as f:
            settings = json.load(f)
        
        print("\n=== Current Settings ===")
        print(f"Admin token: {settings.get('admin_token', 'Not set')}")
        print(f"Balance update interval: {settings.get('balance_update_interval', 14400)} seconds")
        print(f"Gate relogin interval: {settings.get('gate_relogin_interval', 1800)} seconds")
        print(f"Rate limit: {settings.get('rate_limit_per_minute', 240)} req/min")
        print(f"Receipt email: {settings.get('receipt_email', 'Not set')}")
        print(f"Payment methods: {', '.join(settings.get('payment_methods', []))}")
    else:
        print("\n✗ Settings file not found. Run setup_trader.py first.")


def update_settings():
    """Update settings"""
    settings_path = Path("db/settings.json")
    if not settings_path.exists():
        print("\n✗ Settings file not found. Run setup_trader.py first.")
        return
    
    with open(settings_path, 'r') as f:
        settings = json.load(f)
    
    print("\n=== Update Settings ===")
    print("Press Enter to keep current value")
    
    receipt_email = input(f"Receipt email [{settings.get('receipt_email', '')}]: ")
    if receipt_email:
        settings['receipt_email'] = receipt_email
    
    balance_interval = input(f"Balance update interval (seconds) [{settings.get('balance_update_interval', 14400)}]: ")
    if balance_interval:
        settings['balance_update_interval'] = int(balance_interval)
    
    rate_limit = input(f"Gate.io rate limit (req/min) [{settings.get('rate_limit_per_minute', 240)}]: ")
    if rate_limit:
        settings['rate_limit_per_minute'] = int(rate_limit)
    
    with open(settings_path, 'w') as f:
        json.dump(settings, f, indent=2)
    
    print("✓ Settings updated")


def main_menu():
    """Main interactive menu"""
    while True:
        print("\n=== Auto Trader Setup Menu ===")
        print("1. Add Gate.io account")
        print("2. Add Bybit account")
        print("3. Setup Gmail")
        print("4. View accounts")
        print("5. View settings")
        print("6. Update settings")
        print("0. Exit")
        
        choice = input("\nChoose option: ")
        
        if choice == '1':
            add_gate_account()
        elif choice == '2':
            add_bybit_account()
        elif choice == '3':
            setup_gmail()
        elif choice == '4':
            view_accounts()
        elif choice == '5':
            view_settings()
        elif choice == '6':
            update_settings()
        elif choice == '0':
            break
        else:
            print("✗ Invalid choice")
    
    print("\n✓ Setup complete")
    print("\nTo start the auto trader:")
    print("  ./start_auto_trader.sh")
    print("\nTo connect admin client:")
    print("  python3 admin_client_example.py")


if __name__ == "__main__":
    print("=== Auto Trader Interactive Setup ===")
    print("\nThis will help you configure accounts and settings.")
    
    # Check if basic setup is done
    if not Path("db").exists():
        print("\n✗ Please run setup_trader.py first!")
        sys.exit(1)
    
    main_menu()