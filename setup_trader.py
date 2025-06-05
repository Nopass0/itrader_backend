#!/usr/bin/env python3
"""
Setup script for Auto Trader system
Initializes the environment and configures the system
"""

import os
import sys
import json
import secrets
import subprocess
from pathlib import Path
from getpass import getpass


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
    """Check if uv is installed, install if not"""
    print("\nChecking for uv...")
    
    # Check if uv is installed
    try:
        subprocess.run(["uv", "--version"], capture_output=True, check=True)
        print("✓ uv is already installed")
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("uv not found, installing...")
        
    # Install uv
    try:
        # Download and run uv installer
        if sys.platform == "win32":
            # Windows
            subprocess.check_call([
                "powershell", "-c",
                "irm https://astral.sh/uv/install.ps1 | iex"
            ])
        else:
            # Linux/macOS
            subprocess.check_call([
                "sh", "-c",
                "curl -LsSf https://astral.sh/uv/install.sh | sh"
            ])
        
        # Add to PATH for current session
        home = Path.home()
        uv_bin = home / ".cargo" / "bin"
        if uv_bin.exists():
            os.environ["PATH"] = f"{uv_bin}:{os.environ.get('PATH', '')}"
        
        print("✓ uv installed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to install uv: {e}")
        print("\nPlease install uv manually:")
        print("  curl -LsSf https://astral.sh/uv/install.sh | sh")
        print("  or")
        print("  pip install uv")
        return False


def create_venv_with_uv():
    """Create virtual environment using uv"""
    print("\nCreating virtual environment with uv...")
    venv_path = Path(".venv")
    
    if venv_path.exists():
        print("✓ Virtual environment already exists")
        return venv_path
    
    try:
        # Create venv with Python 3.11
        subprocess.check_call(["uv", "venv", "--python", "3.11"])
        print("✓ Virtual environment created with Python 3.11")
        return venv_path
    except subprocess.CalledProcessError:
        # Try with default Python
        try:
            subprocess.check_call(["uv", "venv"])
            print("✓ Virtual environment created")
            return venv_path
        except subprocess.CalledProcessError as e:
            print(f"✗ Failed to create virtual environment: {e}")
            sys.exit(1)


def install_dependencies_with_uv():
    """Install Python dependencies using uv"""
    print("\nInstalling Python dependencies with uv...")
    
    try:
        # Install dependencies
        subprocess.check_call(["uv", "pip", "install", "-r", "requirements_trader.txt"])
        print("✓ Dependencies installed successfully")
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to install dependencies: {e}")
        sys.exit(1)


def setup_gmail_credentials():
    """Setup Gmail OAuth2 credentials"""
    print("\n=== Gmail Setup ===")
    print("To use Gmail API, you need to:")
    print("1. Go to https://console.cloud.google.com/")
    print("2. Create a new project or select existing")
    print("3. Enable Gmail API")
    print("4. Create OAuth2 credentials (Desktop application)")
    print("5. Download the credentials JSON file")
    
    credentials_path = input("\nEnter path to Gmail credentials JSON file (or 'skip' to configure later): ")
    
    if credentials_path.lower() != 'skip' and os.path.exists(credentials_path):
        # Copy credentials to db/gmail/
        with open(credentials_path, 'r') as f:
            credentials = json.load(f)
            
        output_path = Path("db/gmail/credentials.json")
        with open(output_path, 'w') as f:
            json.dump(credentials, f, indent=2)
            
        print(f"✓ Gmail credentials saved to {output_path}")
        return True
    else:
        print("⚠ Gmail setup skipped. You can configure it later.")
        return False


def setup_initial_accounts():
    """Setup initial trading accounts"""
    accounts = {
        "gate": [],
        "bybit": []
    }
    
    print("\n=== Account Setup ===")
    
    # Gate.io accounts
    while True:
        print("\nAdd Gate.io account? (y/n): ", end='')
        if input().lower() != 'y':
            break
            
        login = input("Gate.io login (email): ")
        password = getpass("Gate.io password: ")
        
        account_id = secrets.token_urlsafe(16)
        account_data = {
            "id": account_id,
            "login": login,
            "password": password,
            "status": "inactive"
        }
        
        # Save account
        account_path = Path(f"db/gate/{account_id}.json")
        with open(account_path, 'w') as f:
            json.dump(account_data, f, indent=2)
            
        accounts["gate"].append(account_id)
        print(f"✓ Added Gate.io account: {account_id}")
    
    # Bybit accounts
    while True:
        print("\nAdd Bybit account? (y/n): ", end='')
        if input().lower() != 'y':
            break
            
        api_key = input("Bybit API key: ")
        api_secret = getpass("Bybit API secret: ")
        
        account_id = secrets.token_urlsafe(16)
        account_data = {
            "id": account_id,
            "api_key": api_key,
            "api_secret": api_secret,
            "status": "inactive"
        }
        
        # Save account
        account_path = Path(f"db/bybit/{account_id}.json")
        with open(account_path, 'w') as f:
            json.dump(account_data, f, indent=2)
            
        accounts["bybit"].append(account_id)
        print(f"✓ Added Bybit account: {account_id}")
    
    return accounts


def generate_admin_token():
    """Generate admin token for WebSocket API"""
    token = secrets.token_urlsafe(32)
    print(f"\n=== Admin Token Generated ===")
    print(f"Admin token: {token}")
    print("Save this token securely! You'll need it to connect to the admin API.")
    return token


def create_default_settings(admin_token):
    """Create default settings file"""
    settings = {
        "balance_update_interval": 14400,  # 4 hours
        "gate_relogin_interval": 1800,     # 30 minutes
        "rate_limit_per_minute": 240,      # Gate.io rate limit
        "admin_token": admin_token,
        "payment_methods": ["SBP", "Tinkoff"],
        "alternate_payments": True,
        "ocr_validation": True,
        "cleanup_days": 30
    }
    
    settings_path = Path("db/settings.json")
    with open(settings_path, 'w') as f:
        json.dump(settings, f, indent=2)
        
    print(f"\n✓ Created default settings: {settings_path}")
    return settings


def create_startup_script():
    """Create startup script"""
    script_content = """#!/bin/bash
# Auto Trader startup script

echo "Starting Auto Trader..."

# Activate virtual environment
if [ -d ".venv" ]; then
    source .venv/bin/activate
else
    echo "Virtual environment not found at .venv"
    echo "Please run: python3 setup_trader.py"
    exit 1
fi

# Set Python path
export PYTHONPATH="${PYTHONPATH}:$(pwd)"

# Start the auto trader
python auto_trader.py

# Or run with logging
# python auto_trader.py 2>&1 | tee -a logs/auto_trader.log
"""
    
    script_path = Path("start_trader.sh")
    with open(script_path, 'w') as f:
        f.write(script_content)
        
    # Make executable
    os.chmod(script_path, 0o755)
    
    print(f"✓ Created startup script: {script_path}")


def create_systemd_service():
    """Create systemd service file (optional)"""
    print("\nCreate systemd service for auto-start? (y/n): ", end='')
    if input().lower() != 'y':
        return
        
    service_content = f"""[Unit]
Description=Auto Trader P2P Trading System
After=network.target

[Service]
Type=simple
User={os.getenv('USER')}
WorkingDirectory={os.getcwd()}
ExecStart={sys.executable} {os.path.join(os.getcwd(), 'auto_trader.py')}
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
"""
    
    service_path = Path("auto_trader.service")
    with open(service_path, 'w') as f:
        f.write(service_content)
        
    print(f"\n✓ Created systemd service file: {service_path}")
    print("\nTo install the service:")
    print(f"  sudo cp {service_path} /etc/systemd/system/")
    print("  sudo systemctl daemon-reload")
    print("  sudo systemctl enable auto_trader.service")
    print("  sudo systemctl start auto_trader.service")


def main():
    """Main setup function"""
    print("=== Auto Trader Setup ===\n")
    
    # Create directories
    print("Creating directories...")
    create_directories()
    
    # Check and install uv
    if not check_and_install_uv():
        print("\n✗ Could not install uv. Please install it manually and run setup again.")
        sys.exit(1)
    
    # Create virtual environment with uv
    venv_path = create_venv_with_uv()
    
    # Install dependencies with uv
    install_dependencies_with_uv()
    
    # Setup Gmail
    gmail_configured = setup_gmail_credentials()
    
    # Setup initial accounts
    accounts = setup_initial_accounts()
    
    # Generate admin token
    admin_token = generate_admin_token()
    
    # Create settings
    settings = create_default_settings(admin_token)
    
    # Create startup script
    create_startup_script()
    
    # Optionally create systemd service
    create_systemd_service()
    
    # Summary
    print("\n=== Setup Complete ===")
    print(f"✓ Directories created")
    print(f"✓ Dependencies installed")
    print(f"✓ Gmail configured: {'Yes' if gmail_configured else 'No (configure later)'}")
    print(f"✓ Gate.io accounts: {len(accounts['gate'])}")
    print(f"✓ Bybit accounts: {len(accounts['bybit'])}")
    print(f"✓ Admin token generated")
    print(f"✓ Settings configured")
    
    print("\n=== Next Steps ===")
    print("1. Start the auto trader:")
    print("   ./start_trader.sh")
    print("   or")
    print("   python3 auto_trader.py")
    print("\n2. Connect to admin WebSocket:")
    print(f"   ws://localhost:8765")
    print(f"   Token: {admin_token}")
    print("\n3. Monitor logs in the logs/ directory")
    
    if not gmail_configured:
        print("\n4. Configure Gmail:")
        print("   - Get OAuth2 credentials from Google Cloud Console")
        print("   - Save as db/gmail/credentials.json")
        print("   - Restart the auto trader")


if __name__ == "__main__":
    main()