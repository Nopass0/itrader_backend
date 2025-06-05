#!/usr/bin/env python3
"""
Quick setup for Auto Trader - non-interactive version
Creates basic structure and installs dependencies
"""

import os
import sys
import json
import secrets
import subprocess
from pathlib import Path


def create_directories():
    """Create necessary directories"""
    directories = [
        "db",
        "db/gate",
        "db/bybit",
        "db/gmail",
        "db/transactions",
        "db/checks",
        "logs"
    ]
    
    for directory in directories:
        Path(directory).mkdir(parents=True, exist_ok=True)
        print(f"✓ Created directory: {directory}")


def check_and_install_uv():
    """Check if uv is installed"""
    try:
        subprocess.run(["uv", "--version"], capture_output=True, check=True)
        print("✓ uv is already installed")
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("✗ uv not found")
        print("\nTo install uv:")
        print("  curl -LsSf https://astral.sh/uv/install.sh | sh")
        print("\nOr with pip:")
        print("  pip install uv")
        return False


def setup_venv():
    """Setup virtual environment with uv"""
    if not check_and_install_uv():
        return False
    
    print("\nSetting up virtual environment...")
    
    # Create venv if not exists
    if not Path(".venv").exists():
        try:
            subprocess.check_call(["uv", "venv", "--python", "3.11"])
            print("✓ Created virtual environment with Python 3.11")
        except subprocess.CalledProcessError:
            try:
                subprocess.check_call(["uv", "venv"])
                print("✓ Created virtual environment")
            except subprocess.CalledProcessError as e:
                print(f"✗ Failed to create venv: {e}")
                return False
    
    # Install dependencies
    try:
        subprocess.check_call(["uv", "pip", "install", "-r", "requirements_trader.txt"])
        print("✓ Dependencies installed")
        return True
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to install dependencies: {e}")
        return False


def create_default_settings():
    """Create default settings file"""
    admin_token = secrets.token_urlsafe(32)
    
    settings = {
        "admin_token": admin_token,
        "balance_update_interval": 14400,  # 4 hours
        "gate_relogin_interval": 1800,     # 30 minutes
        "rate_limit_per_minute": 240,
        "payment_methods": ["SBP", "Tinkoff"],
        "alternate_payments": True,
        "ocr_validation": True,
        "cleanup_days": 30,
        "receipt_email": "your-receipts@gmail.com"
    }
    
    settings_path = Path("db/settings.json")
    with open(settings_path, 'w') as f:
        json.dump(settings, f, indent=2)
    
    print(f"\n✓ Created settings file")
    print(f"🔑 Admin token: {admin_token}")
    print(f"   Save this token! You'll need it for admin access.")
    
    return admin_token


def create_env_file():
    """Create .env.trader template"""
    env_content = """# Auto Trader Environment Variables

# Bybit API (add your credentials)
BYBIT_API_KEY=
BYBIT_API_SECRET=
BYBIT_TESTNET=false

# Payment Details
PAYMENT_BANK=Тинькофф
PAYMENT_PHONE=+7 900 123-45-67
RECEIPT_EMAIL=your-receipts@gmail.com

# Email Monitoring (Gmail)
EMAIL_USERNAME=
EMAIL_PASSWORD=

# Trading Settings
PROFIT_MARGIN_PERCENT=2.0
MIN_ORDER_AMOUNT=1000.0
MAX_ORDER_AMOUNT=50000.0
"""
    
    with open(".env.trader", 'w') as f:
        f.write(env_content)
    
    print("✓ Created .env.trader template")


def main():
    """Main setup function"""
    print("=== Auto Trader Quick Setup ===\n")
    
    # Create directories
    print("Creating directories...")
    create_directories()
    
    # Setup virtual environment
    if not setup_venv():
        print("\n✗ Failed to setup environment")
        print("Please install uv and try again")
        sys.exit(1)
    
    # Create default settings
    create_default_settings()
    
    # Create env template
    create_env_file()
    
    print("\n=== Setup Complete ===")
    print("\n📋 Next steps:")
    print("1. Add accounts:")
    print("   python3 setup_interactive.py")
    print("\n2. Configure Gmail:")
    print("   - Get OAuth2 credentials from Google Cloud Console")
    print("   - Save as db/gmail/credentials.json")
    print("\n3. Start the trader:")
    print("   ./start_auto_trader.sh")
    print("\n4. Connect admin client:")
    print("   python3 admin_client_example.py")
    
    print("\n✅ Basic setup complete!")


if __name__ == "__main__":
    main()