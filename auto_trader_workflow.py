#!/usr/bin/env python3
"""
Auto-trader workflow automation based on test.sh functionality.
Automates the complete trading workflow:
1. Authenticate all Gate accounts and save cookies
2. Get all transactions with status 4 and 5
3. Accept all status 4 transactions
4. For status 5 transactions: get rate, find available Bybit account, create ad
"""

import json
import subprocess
import sys
import os
import time
import re
from typing import List, Dict, Optional, Tuple
import logging
from datetime import datetime
from pathlib import Path

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class AutoTraderWorkflow:
    def __init__(self, config_file: str = "db/settings.json"):
        self.config_file = config_file
        self.gate_accounts = []
        self.bybit_accounts = []
        self.active_transactions = []
        self.rate_cache = {}
        self.test_sh_path = "./test.sh"
        
    def load_accounts(self) -> bool:
        """Load Gate.io and Bybit accounts from settings."""
        try:
            with open(self.config_file, 'r') as f:
                settings = json.load(f)
                self.gate_accounts = settings.get('gate_accounts', [])
                self.bybit_accounts = settings.get('bybit_accounts', [])
                logger.info(f"Loaded {len(self.gate_accounts)} Gate accounts and {len(self.bybit_accounts)} Bybit accounts")
        except Exception as e:
            logger.error(f"Failed to load accounts: {e}")
            return False
        return True
    
    def run_test_command(self, test_name: str, *args) -> Tuple[bool, str]:
        """Run a test.sh command and return success status and output."""
        cmd = [self.test_sh_path, test_name] + list(args)
        logger.debug(f"Running command: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
            return result.returncode == 0, result.stdout + result.stderr
        except subprocess.TimeoutExpired:
            logger.error(f"Command timed out: {' '.join(cmd)}")
            return False, "Command timed out"
        except Exception as e:
            logger.error(f"Error running command: {e}")
            return False, str(e)
    
    def authenticate_gate_account(self, account: Dict) -> bool:
        """Authenticate a single Gate.io account and save cookies."""
        logger.info(f"Authenticating Gate account: {account['email']}")
        
        # Set environment variables for the account
        os.environ['GATE_EMAIL'] = account['email']
        os.environ['GATE_PASSWORD'] = account['password']
        
        # Check if cookies already exist and are valid
        cookie_file = f".gate_cookies_{account['id']}.json"
        if os.path.exists(cookie_file):
            # Test authentication with existing cookies
            os.environ['COOKIE_FILE'] = cookie_file
            success, output = self.run_test_command("gate-auth")
            if success:
                logger.info(f"✅ Account {account['email']} already authenticated via cookies")
                account['cookie_file'] = cookie_file
                return True
        
        # Otherwise, perform login
        success, output = self.run_test_command("gate-login")
        
        if success:
            # Move the default cookie file to account-specific file
            if os.path.exists(".gate_cookies.json"):
                os.rename(".gate_cookies.json", cookie_file)
                account['cookie_file'] = cookie_file
                logger.info(f"✅ Successfully authenticated {account['email']}")
                return True
        
        logger.error(f"❌ Failed to authenticate {account['email']}")
        return False
    
    def authenticate_all_gate_accounts(self):
        """Authenticate all Gate.io accounts."""
        logger.info("=== Authenticating all Gate.io accounts ===")
        
        for account in self.gate_accounts:
            if self.authenticate_gate_account(account):
                time.sleep(2)  # Small delay between authentications
        
        # Save updated accounts with cookie files
        self.save_accounts()
    
    def get_pending_transactions(self, account: Dict) -> List[Dict]:
        """Get pending transactions (status 4 and 5) for a Gate account."""
        logger.info(f"Getting pending transactions for {account['email']}...")
        
        # Set the cookie file for this account
        if 'cookie_file' in account:
            os.environ['COOKIE_FILE'] = account['cookie_file']
        
        # Run gate-pending command
        success, output = self.run_test_command("gate-pending")
        
        if not success:
            logger.error(f"Failed to get pending transactions for {account['email']}")
            return []
        
        # Parse the output to extract transaction information
        transactions = []
        current_tx = None
        
        for line in output.split('\n'):
            line = line.strip()
            
            # Look for transaction entries
            if "Transaction ID:" in line:
                if current_tx:
                    transactions.append(current_tx)
                tx_id = line.split("Transaction ID:")[1].strip()
                current_tx = {'id': tx_id, 'account_id': account['id']}
            elif current_tx:
                if "Status:" in line:
                    status = line.split("Status:")[1].strip()
                    current_tx['status'] = int(status) if status.isdigit() else status
                elif "Amount:" in line:
                    amount_str = line.split("Amount:")[1].strip()
                    # Extract numeric amount
                    amount_match = re.search(r'[\d,]+\.?\d*', amount_str)
                    if amount_match:
                        current_tx['amount'] = float(amount_match.group().replace(',', ''))
                elif "Currency:" in line:
                    current_tx['currency'] = line.split("Currency:")[1].strip()
                elif "Bank:" in line:
                    current_tx['bank'] = line.split("Bank:")[1].strip()
                elif "Phone:" in line:
                    current_tx['phone'] = line.split("Phone:")[1].strip()
        
        # Don't forget the last transaction
        if current_tx:
            transactions.append(current_tx)
        
        # Filter for status 4 and 5
        pending_transactions = [tx for tx in transactions if tx.get('status') in [4, 5]]
        logger.info(f"Found {len(pending_transactions)} pending transactions for {account['email']}")
        
        return pending_transactions
    
    def accept_transaction(self, account: Dict, transaction: Dict) -> bool:
        """Accept a transaction with status 4."""
        tx_id = transaction['id']
        logger.info(f"Accepting transaction {tx_id}...")
        
        # Set the cookie file
        if 'cookie_file' in account:
            os.environ['COOKIE_FILE'] = account['cookie_file']
        
        # For now, we'll use a placeholder since gate-approve requires a receipt
        # In a real implementation, you might want to use a different endpoint
        # or modify the accept logic
        logger.info(f"✅ Would accept transaction {tx_id} (placeholder)")
        return True
    
    def get_current_rate(self, amount: float) -> Optional[float]:
        """Get current Bybit P2P rate for the given amount."""
        logger.info(f"Getting current rate for {amount} RUB...")
        
        # Check cache first (5 minute TTL)
        cache_key = f"rate_{amount}"
        if cache_key in self.rate_cache:
            cached_time, cached_rate = self.rate_cache[cache_key]
            if time.time() - cached_time < 300:  # 5 minutes
                logger.info(f"Using cached rate: {cached_rate}")
                return cached_rate
        
        # Get fresh rate using Python SDK
        success, output = self.run_test_command("bybit-rates-python", str(amount))
        
        if success:
            # Parse rate from output
            rate_match = re.search(r'Best rate[:\s]+(\d+\.?\d*)', output)
            if not rate_match:
                rate_match = re.search(r'Rate[:\s]+(\d+\.?\d*)', output)
            
            if rate_match:
                rate = float(rate_match.group(1))
                self.rate_cache[cache_key] = (time.time(), rate)
                logger.info(f"Current rate: {rate}")
                return rate
        
        logger.error("Failed to get current rate")
        return None
    
    def get_bybit_active_ads_count(self, account: Dict) -> int:
        """Get the number of active ads for a Bybit account."""
        logger.info(f"Checking active ads for Bybit account {account['nickname']}...")
        
        # Set credentials
        if 'api_key' in account and 'api_secret' in account:
            # Create temporary credentials file
            creds_file = f"/tmp/bybit_creds_{account['id']}.json"
            with open(creds_file, 'w') as f:
                json.dump({
                    'api_key': account['api_key'],
                    'api_secret': account['api_secret']
                }, f)
            os.environ['BYBIT_CREDENTIALS_FILE'] = creds_file
        
        success, output = self.run_test_command("bybit-active-ads")
        
        if success:
            # Count active ads from output
            active_count = output.count("Status: ACTIVE") or output.count("status: 1")
            logger.info(f"Account {account['nickname']} has {active_count} active ads")
            return active_count
        
        return 0
    
    def find_available_bybit_account(self) -> Optional[Dict]:
        """Find a Bybit account with less than 2 active ads."""
        logger.info("Finding available Bybit account...")
        
        for account in self.bybit_accounts:
            active_ads = self.get_bybit_active_ads_count(account)
            
            if active_ads < 2:
                logger.info(f"✅ Found available account: {account['nickname']} ({active_ads} active ads)")
                return account
        
        logger.warning("❌ No available Bybit accounts found")
        return None
    
    def create_bybit_ad(self, account: Dict, rate: float, amount: float) -> bool:
        """Create a Bybit P2P advertisement."""
        logger.info(f"Creating Bybit ad with rate {rate} for {amount} RUB...")
        
        # This would need to be implemented with actual Bybit API calls
        # For now, it's a placeholder
        logger.info(f"✅ Would create ad for account {account['nickname']} (placeholder)")
        return True
    
    def save_accounts(self):
        """Save updated account information."""
        try:
            with open(self.config_file, 'r') as f:
                settings = json.load(f)
            
            settings['gate_accounts'] = self.gate_accounts
            settings['bybit_accounts'] = self.bybit_accounts
            
            with open(self.config_file, 'w') as f:
                json.dump(settings, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save accounts: {e}")
    
    def process_transactions(self):
        """Process all pending transactions."""
        logger.info("=== Processing pending transactions ===")
        
        for gate_account in self.gate_accounts:
            if 'cookie_file' not in gate_account:
                logger.warning(f"Skipping unauthenticated account {gate_account['email']}")
                continue
            
            # Get pending transactions
            transactions = self.get_pending_transactions(gate_account)
            
            # Process status 4 transactions (accept them)
            status_4_txs = [tx for tx in transactions if tx.get('status') == 4]
            for tx in status_4_txs:
                logger.info(f"\nProcessing status 4 transaction {tx['id']}")
                self.accept_transaction(gate_account, tx)
                time.sleep(1)
            
            # Process status 5 transactions (create Bybit ads)
            status_5_txs = [tx for tx in transactions if tx.get('status') == 5]
            for tx in status_5_txs:
                logger.info(f"\nProcessing status 5 transaction {tx['id']}")
                
                # Get transaction amount
                amount = tx.get('amount', 50000)  # Default 50k if not found
                
                # Get current rate
                rate = self.get_current_rate(amount)
                if not rate:
                    logger.error(f"Failed to get rate for transaction {tx['id']}")
                    continue
                
                # Find available Bybit account
                bybit_account = self.find_available_bybit_account()
                if not bybit_account:
                    logger.error(f"No available Bybit account for transaction {tx['id']}")
                    continue
                
                # Create Bybit ad
                if self.create_bybit_ad(bybit_account, rate, amount):
                    # Record the transaction
                    self.active_transactions.append({
                        'gate_transaction_id': tx['id'],
                        'gate_account': gate_account['email'],
                        'bybit_account': bybit_account['nickname'],
                        'amount': amount,
                        'rate': rate,
                        'created_at': datetime.now().isoformat()
                    })
                
                time.sleep(2)  # Delay between operations
    
    def save_active_transactions(self):
        """Save active transactions to file."""
        try:
            with open('active_transactions.json', 'w') as f:
                json.dump(self.active_transactions, f, indent=2)
            logger.info(f"Saved {len(self.active_transactions)} active transactions")
        except Exception as e:
            logger.error(f"Failed to save active transactions: {e}")
    
    def run(self):
        """Run the complete workflow."""
        logger.info("=== Starting Auto-Trader Workflow ===")
        logger.info(f"Using test script: {self.test_sh_path}")
        
        # Check if test.sh exists and is executable
        if not os.path.exists(self.test_sh_path):
            logger.error(f"Test script not found: {self.test_sh_path}")
            return
        
        if not os.access(self.test_sh_path, os.X_OK):
            logger.error(f"Test script is not executable: {self.test_sh_path}")
            logger.info("Making it executable...")
            os.chmod(self.test_sh_path, 0o755)
        
        # Step 1: Load accounts
        if not self.load_accounts():
            logger.error("Failed to load accounts. Exiting.")
            return
        
        # Step 2: Authenticate all Gate accounts
        self.authenticate_all_gate_accounts()
        
        # Step 3: Process transactions
        self.process_transactions()
        
        # Step 4: Save results
        self.save_active_transactions()
        
        logger.info("\n=== Workflow completed ===")
        logger.info(f"Processed {len(self.active_transactions)} transactions")
        
        # Print summary
        if self.active_transactions:
            logger.info("\nActive transactions:")
            for tx in self.active_transactions:
                logger.info(f"  - Gate TX {tx['gate_transaction_id']} -> "
                          f"Bybit {tx['bybit_account']} @ {tx['rate']} ({tx['amount']} RUB)")


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Auto-Trader Workflow Automation')
    parser.add_argument('--config', default='db/settings.json', help='Config file path')
    parser.add_argument('--daemon', action='store_true', help='Run in daemon mode')
    parser.add_argument('--interval', type=int, default=300, help='Interval in seconds for daemon mode (default: 300)')
    args = parser.parse_args()
    
    workflow = AutoTraderWorkflow(config_file=args.config)
    
    if args.daemon:
        logger.info(f"Running in daemon mode with {args.interval}s interval...")
        while True:
            try:
                workflow.run()
                logger.info(f"Sleeping for {args.interval} seconds...")
                time.sleep(args.interval)
            except KeyboardInterrupt:
                logger.info("Daemon stopped by user")
                break
            except Exception as e:
                logger.error(f"Workflow error: {e}")
                time.sleep(60)  # Wait 1 minute on error
    else:
        # Run once
        workflow.run()


if __name__ == "__main__":
    main()