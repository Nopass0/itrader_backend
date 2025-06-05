"""
Bybit Multi-Account Client
Manages multiple Bybit accounts for P2P trading
"""

import sys
import os
import json
import logging
import asyncio
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor

# Add python_modules to path for bybit_wrapper
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python_modules'))

from python_modules.bybit_wrapper import BybitP2PWrapper

logger = logging.getLogger(__name__)


class BybitMultiClient:
    """Manages multiple Bybit P2P clients"""
    
    def __init__(self, account_manager, db_manager):
        self.account_manager = account_manager
        self.db_manager = db_manager
        self.clients: Dict[str, BybitP2PWrapper] = {}
        self.executor = ThreadPoolExecutor(max_workers=10)
        self._initialize_clients()
        
    def _initialize_clients(self):
        """Initialize clients for all accounts"""
        for account_id in self.account_manager.list_bybit_accounts():
            self.add_client(account_id)
    
    def add_client(self, account_id: str) -> Optional[BybitP2PWrapper]:
        """Add a new client for an account"""
        account = self.account_manager.get_bybit_account(account_id)
        if not account:
            logger.error(f"Bybit account not found: {account_id}")
            return None
        
        try:
            # Create client
            client = BybitP2PWrapper(
                api_key=account.api_key,
                api_secret=account.api_secret,
                testnet=account.testnet
            )
            
            # Test connection
            server_time = client.get_server_time()
            logger.info(f"Connected to Bybit for account {account_id}, server time: {server_time}")
            
            # Store client
            self.clients[account_id] = client
            self.account_manager.set_account_client(account_id, client, "bybit")
            
            # Update account status
            self.account_manager.update_bybit_status(account_id, "active")
            
            return client
            
        except Exception as e:
            logger.error(f"Failed to create Bybit client for {account_id}: {e}")
            self.account_manager.update_bybit_status(account_id, "error", str(e))
            return None
    
    def remove_client(self, account_id: str):
        """Remove a client"""
        if account_id in self.clients:
            del self.clients[account_id]
            logger.info(f"Removed Bybit client: {account_id}")
    
    def get_client(self, account_id: str) -> Optional[BybitP2PWrapper]:
        """Get a client by account ID"""
        return self.clients.get(account_id)
    
    async def test_all_connections(self) -> Dict[str, Dict[str, Any]]:
        """Test connections for all accounts"""
        results = {}
        
        for account_id, client in self.clients.items():
            try:
                # Get account info
                loop = asyncio.get_event_loop()
                account_info = await loop.run_in_executor(
                    self.executor,
                    client.get_account_info
                )
                
                results[account_id] = {
                    "success": True,
                    "info": account_info
                }
                
                # Update status
                self.account_manager.update_bybit_status(account_id, "active")
                
            except Exception as e:
                logger.error(f"Connection test failed for {account_id}: {e}")
                results[account_id] = {
                    "success": False,
                    "error": str(e)
                }
                
                # Update status
                self.account_manager.update_bybit_status(account_id, "error", str(e))
        
        return results
    
    async def fetch_rates_all_accounts(self, params: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
        """Fetch P2P rates from all accounts"""
        results = {}
        active_accounts = [
            acc_id for acc_id, client in self.clients.items()
            if self.account_manager.get_bybit_account(acc_id).status == "active"
        ]
        
        for account_id in active_accounts:
            try:
                rates = await self._fetch_rates(account_id, params)
                results[account_id] = rates
            except Exception as e:
                logger.error(f"Failed to fetch rates for {account_id}: {e}")
                results[account_id] = {"success": False, "error": str(e)}
        
        return results
    
    async def _fetch_rates(self, account_id: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Fetch rates for a single account"""
        client = self.get_client(account_id)
        if not client:
            return {"success": False, "error": "Client not found"}
        
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            client.fetch_p2p_rates,
            params
        )
    
    async def get_best_rate(self, token_id: str = "USDT", currency_id: str = "RUB", 
                           side: int = 0) -> Optional[Dict[str, Any]]:
        """Get best P2P rate across all accounts"""
        params = {
            "token_id": token_id,
            "currency_id": currency_id,
            "side": side,
            "payment": ["382", "75"],  # СБП and Тинькофф
            "size": 5
        }
        
        all_rates = await self.fetch_rates_all_accounts(params)
        
        best_rate = None
        best_price = None
        
        for account_id, rate_data in all_rates.items():
            if rate_data.get("success") and rate_data.get("result"):
                items = rate_data["result"].get("items", [])
                
                for item in items:
                    price = float(item.get("price", 0))
                    
                    # For buying USDT (side=0), we want the lowest price
                    # For selling USDT (side=1), we want the highest price
                    if side == 0:  # Buy
                        if best_price is None or price < best_price:
                            best_price = price
                            best_rate = {
                                "account_id": account_id,
                                "price": price,
                                "item": item
                            }
                    else:  # Sell
                        if best_price is None or price > best_price:
                            best_price = price
                            best_rate = {
                                "account_id": account_id,
                                "price": price,
                                "item": item
                            }
        
        return best_rate
    
    async def create_advertisement_all(self, params: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
        """Create advertisement on all active accounts"""
        results = {}
        active_accounts = [
            acc_id for acc_id, client in self.clients.items()
            if self.account_manager.get_bybit_account(acc_id).status == "active"
        ]
        
        for account_id in active_accounts:
            try:
                result = await self._create_advertisement(account_id, params)
                results[account_id] = result
            except Exception as e:
                logger.error(f"Failed to create ad for {account_id}: {e}")
                results[account_id] = {"success": False, "error": str(e)}
        
        return results
    
    async def _create_advertisement(self, account_id: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Create advertisement for a single account"""
        client = self.get_client(account_id)
        if not client:
            return {"success": False, "error": "Client not found"}
        
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            client.create_advertisement,
            params
        )
    
    async def get_all_active_orders(self) -> Dict[str, List[Dict[str, Any]]]:
        """Get active orders from all accounts"""
        results = {}
        
        for account_id, client in self.clients.items():
            if self.account_manager.get_bybit_account(account_id).status == "active":
                try:
                    orders = await self._get_active_orders(account_id)
                    results[account_id] = orders
                except Exception as e:
                    logger.error(f"Failed to get orders for {account_id}: {e}")
                    results[account_id] = []
        
        return results
    
    async def _get_active_orders(self, account_id: str) -> List[Dict[str, Any]]:
        """Get active orders for a single account"""
        client = self.get_client(account_id)
        if not client:
            return []
        
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            client.get_active_orders
        )
    
    async def handle_order(self, account_id: str, order_id: str, action: str) -> bool:
        """Handle order action (confirm_payment, release, etc.)"""
        client = self.get_client(account_id)
        if not client:
            return False
        
        loop = asyncio.get_event_loop()
        
        try:
            if action == "confirm_payment":
                return await loop.run_in_executor(
                    self.executor,
                    client.confirm_payment,
                    order_id
                )
            elif action == "release":
                return await loop.run_in_executor(
                    self.executor,
                    client.release_order,
                    order_id
                )
            else:
                logger.error(f"Unknown order action: {action}")
                return False
                
        except Exception as e:
            logger.error(f"Failed to {action} order {order_id} for {account_id}: {e}")
            return False
    
    async def send_chat_message(self, account_id: str, order_id: str, message: str) -> bool:
        """Send chat message in order"""
        client = self.get_client(account_id)
        if not client:
            return False
        
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            client.send_chat_message,
            order_id,
            message
        )
    
    async def get_chat_messages(self, account_id: str, order_id: str) -> List[Dict[str, Any]]:
        """Get chat messages for order"""
        client = self.get_client(account_id)
        if not client:
            return []
        
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            client.get_chat_messages,
            order_id
        )
    
    async def monitor_orders(self, callback, interval: int = 30):
        """Monitor orders across all accounts"""
        while True:
            try:
                all_orders = await self.get_all_active_orders()
                
                for account_id, orders in all_orders.items():
                    for order in orders:
                        # Check if this is a new order or status changed
                        order_data = {
                            "account_id": account_id,
                            "order": order
                        }
                        
                        # Call the callback
                        await callback(order_data)
                
            except Exception as e:
                logger.error(f"Error monitoring orders: {e}")
            
            # Wait before next check
            await asyncio.sleep(interval)
    
    def get_account_stats(self) -> Dict[str, Any]:
        """Get statistics for all Bybit accounts"""
        stats = {
            "total_accounts": len(self.clients),
            "active_accounts": sum(
                1 for acc_id in self.clients
                if self.account_manager.get_bybit_account(acc_id).status == "active"
            ),
            "error_accounts": sum(
                1 for acc_id in self.clients
                if self.account_manager.get_bybit_account(acc_id).status == "error"
            ),
            "accounts": {}
        }
        
        for account_id, client in self.clients.items():
            account = self.account_manager.get_bybit_account(account_id)
            if account:
                stats["accounts"][account_id] = {
                    "status": account.status,
                    "last_error": account.last_error,
                    "active_ads": len(account.active_ads),
                    "active_orders": len(account.active_orders)
                }
        
        return stats
    
    def cleanup(self):
        """Cleanup resources"""
        self.executor.shutdown(wait=True)
        self.clients.clear()
        logger.info("Bybit multi-client cleaned up")