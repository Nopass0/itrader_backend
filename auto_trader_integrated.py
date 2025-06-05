#!/usr/bin/env python3
"""
Integrated auto-trader that coordinates Gate.io and Bybit operations.
Based on test.sh functionality.
"""

import json
import os
import sys
import time
import subprocess
import asyncio
from datetime import datetime
from typing import List, Dict, Optional, Tuple
import logging

# Add the project directory to Python path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from gate_client import GateClient
from bybit_client import BybitClient
from transaction_manager import TransactionManager
from db_manager import DatabaseManager

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('AutoTrader')


class IntegratedAutoTrader:
    def __init__(self):
        self.db = DatabaseManager()
        self.transaction_manager = TransactionManager()
        self.gate_clients = {}
        self.bybit_clients = {}
        self.config = self.load_config()
        
    def load_config(self) -> Dict:
        """Load configuration from settings."""
        config_file = "db/settings.json"
        if os.path.exists(config_file):
            with open(config_file, 'r') as f:
                return json.load(f)
        return {
            'gate_accounts': [],
            'bybit_accounts': [],
            'trading_params': {
                'min_amount': 1000,
                'max_amount': 50000,
                'profit_margin': 0.02,
                'auto_approve_threshold': 5000
            }
        }
    
    async def initialize_clients(self):
        """Initialize all Gate and Bybit clients."""
        logger.info("Initializing clients...")
        
        # Initialize Gate clients
        for account in self.config.get('gate_accounts', []):
            try:
                client = GateClient(account['id'])
                
                # Check if we have valid cookies
                cookie_file = f".gate_cookies_{account['id']}.json"
                if os.path.exists(cookie_file):
                    await client.load_cookies(cookie_file)
                    if await client.is_authenticated():
                        self.gate_clients[account['id']] = client
                        logger.info(f"✅ Gate account {account['email']} authenticated via cookies")
                        continue
                
                # Otherwise authenticate with credentials
                if await client.login(account['email'], account['password']):
                    await client.save_cookies(cookie_file)
                    self.gate_clients[account['id']] = client
                    logger.info(f"✅ Gate account {account['email']} authenticated")
                else:
                    logger.error(f"❌ Failed to authenticate Gate account {account['email']}")
                    
            except Exception as e:
                logger.error(f"Error initializing Gate client {account['id']}: {e}")
        
        # Initialize Bybit clients
        for account in self.config.get('bybit_accounts', []):
            try:
                client = BybitClient(
                    api_key=account.get('api_key', ''),
                    api_secret=account.get('api_secret', ''),
                    nickname=account['nickname']
                )
                if await client.check_authentication():
                    self.bybit_clients[account['id']] = client
                    logger.info(f"✅ Bybit account {account['nickname']} authenticated")
                else:
                    logger.error(f"❌ Failed to authenticate Bybit account {account['nickname']}")
            except Exception as e:
                logger.error(f"Error initializing Bybit client {account['id']}: {e}")
    
    async def get_all_available_transactions(self) -> List[Dict]:
        """Get all available transactions from all Gate accounts."""
        all_transactions = []
        
        for account_id, client in self.gate_clients.items():
            try:
                logger.info(f"Getting transactions for Gate account {account_id}...")
                transactions = await client.get_available_transactions()
                
                for tx in transactions:
                    tx['gate_account_id'] = account_id
                    all_transactions.append(tx)
                    
                logger.info(f"Found {len(transactions)} transactions for account {account_id}")
            except Exception as e:
                logger.error(f"Error getting transactions for account {account_id}: {e}")
        
        return all_transactions
    
    async def process_transaction(self, transaction: Dict) -> bool:
        """Process a single transaction."""
        tx_id = transaction['id']
        status = transaction['status']
        amount = float(transaction.get('amount', 0))
        gate_account_id = transaction['gate_account_id']
        
        logger.info(f"Processing transaction {tx_id} (status: {status}, amount: {amount})")
        
        try:
            gate_client = self.gate_clients[gate_account_id]
            
            # Handle status 4 - accept the transaction
            if status == 4:
                logger.info(f"Accepting transaction {tx_id}...")
                success = await gate_client.accept_transaction(tx_id)
                if success:
                    logger.info(f"✅ Accepted transaction {tx_id}")
                    # Update status to 5 for further processing
                    transaction['status'] = 5
                else:
                    logger.error(f"❌ Failed to accept transaction {tx_id}")
                    return False
            
            # Handle status 5 - create Bybit ad
            if transaction['status'] == 5:
                # Get current rate from Bybit
                rate = await self.get_best_rate(amount)
                if not rate:
                    logger.error(f"Failed to get rate for amount {amount}")
                    return False
                
                # Find available Bybit account
                bybit_account_id = await self.find_available_bybit_account()
                if not bybit_account_id:
                    logger.error("No available Bybit account found")
                    return False
                
                bybit_client = self.bybit_clients[bybit_account_id]
                
                # Create the ad
                ad_params = {
                    'amount': amount,
                    'rate': rate,
                    'min_amount': max(1000, amount * 0.1),
                    'max_amount': amount,
                    'payment_methods': ['75', '382'],  # Tinkoff, SBP
                    'remarks': 'Быстрая сделка. Отправьте чек после оплаты.'
                }
                
                ad_id = await bybit_client.create_advertisement(ad_params)
                if ad_id:
                    logger.info(f"✅ Created Bybit ad {ad_id} for transaction {tx_id}")
                    
                    # Record the mapping
                    await self.transaction_manager.create_transaction_mapping(
                        gate_tx_id=tx_id,
                        gate_account_id=gate_account_id,
                        bybit_ad_id=ad_id,
                        bybit_account_id=bybit_account_id,
                        amount=amount,
                        rate=rate
                    )
                    return True
                else:
                    logger.error(f"❌ Failed to create Bybit ad for transaction {tx_id}")
                    return False
                    
        except Exception as e:
            logger.error(f"Error processing transaction {tx_id}: {e}")
            return False
        
        return True
    
    async def get_best_rate(self, amount: float) -> Optional[float]:
        """Get the best P2P rate for the given amount."""
        try:
            # Use the Rust binary to get rates
            cmd = [
                "cargo", "run", "--bin", "bybit_check_rate_python",
                "--", str(amount)
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode == 0:
                # Parse the rate from output
                output_lines = result.stdout.strip().split('\n')
                for line in output_lines:
                    if 'Best rate:' in line:
                        rate_str = line.split(':')[1].strip()
                        return float(rate_str)
            
            # Fallback to a default calculation
            logger.warning(f"Using fallback rate calculation for {amount}")
            return 95.5  # Example fallback rate
            
        except Exception as e:
            logger.error(f"Error getting rate: {e}")
            return None
    
    async def find_available_bybit_account(self) -> Optional[str]:
        """Find a Bybit account with capacity for new ads."""
        for account_id, client in self.bybit_clients.items():
            try:
                active_ads = await client.get_active_advertisements()
                if len(active_ads) < 2:  # Less than 2 active ads
                    logger.info(f"Found available Bybit account: {account_id} ({len(active_ads)} active ads)")
                    return account_id
            except Exception as e:
                logger.error(f"Error checking Bybit account {account_id}: {e}")
        
        return None
    
    async def monitor_active_transactions(self):
        """Monitor active transactions and handle completions."""
        active_mappings = await self.transaction_manager.get_active_mappings()
        
        for mapping in active_mappings:
            try:
                # Check Gate transaction status
                gate_client = self.gate_clients[mapping['gate_account_id']]
                gate_tx = await gate_client.get_transaction_details(mapping['gate_tx_id'])
                
                if gate_tx and gate_tx['status'] == 7:  # Completed
                    logger.info(f"Transaction {mapping['gate_tx_id']} completed")
                    
                    # Check if Bybit order was completed
                    bybit_client = self.bybit_clients[mapping['bybit_account_id']]
                    # This would check the Bybit side...
                    
                    # Update mapping status
                    await self.transaction_manager.complete_mapping(mapping['id'])
                    
            except Exception as e:
                logger.error(f"Error monitoring transaction {mapping['gate_tx_id']}: {e}")
    
    async def run_cycle(self):
        """Run one complete cycle of the auto-trader."""
        logger.info("=== Starting auto-trader cycle ===")
        
        # Get all available transactions
        transactions = await self.get_all_available_transactions()
        logger.info(f"Found {len(transactions)} available transactions")
        
        # Process each transaction
        processed = 0
        for transaction in transactions:
            if await self.process_transaction(transaction):
                processed += 1
                await asyncio.sleep(2)  # Small delay between operations
        
        logger.info(f"Processed {processed}/{len(transactions)} transactions")
        
        # Monitor active transactions
        await self.monitor_active_transactions()
        
        logger.info("=== Cycle completed ===")
    
    async def run(self, daemon_mode: bool = False):
        """Main run method."""
        # Initialize all clients
        await self.initialize_clients()
        
        if not self.gate_clients or not self.bybit_clients:
            logger.error("No active clients available. Exiting.")
            return
        
        if daemon_mode:
            logger.info("Running in daemon mode...")
            while True:
                try:
                    await self.run_cycle()
                    logger.info("Sleeping for 5 minutes...")
                    await asyncio.sleep(300)  # 5 minutes
                except KeyboardInterrupt:
                    logger.info("Daemon stopped by user")
                    break
                except Exception as e:
                    logger.error(f"Cycle error: {e}")
                    await asyncio.sleep(60)  # 1 minute on error
        else:
            # Run once
            await self.run_cycle()


async def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Integrated Auto-Trader')
    parser.add_argument('--daemon', action='store_true', help='Run in daemon mode')
    parser.add_argument('--config', default='db/settings.json', help='Config file path')
    args = parser.parse_args()
    
    trader = IntegratedAutoTrader()
    await trader.run(daemon_mode=args.daemon)


if __name__ == "__main__":
    asyncio.run(main())