"""
Gate.io Multi-Account Client
Manages multiple Gate.io accounts with cookie-based authentication
"""

import requests
import json
import time
import logging
import asyncio
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
import urllib.parse
from concurrent.futures import ThreadPoolExecutor

logger = logging.getLogger(__name__)


class GateClient:
    """Single Gate.io account client"""
    
    def __init__(self, account_id: str, cookies: Optional[Dict[str, Any]] = None):
        self.account_id = account_id
        self.cookies = cookies or {}
        self.session = requests.Session()
        self.base_url = "https://www.gate.io"
        self.api_base = "https://www.gate.io/api/v1"
        self.last_request_time = 0
        self.min_request_interval = 0.25  # 240 requests per minute = 0.25s per request
        
        # Set default headers
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            'Accept': 'application/json, text/plain, */*',
            'Accept-Language': 'en-US,en;q=0.9',
            'Accept-Encoding': 'gzip, deflate, br',
            'Origin': 'https://www.gate.io',
            'Referer': 'https://www.gate.io/',
        })
        
        # Set cookies if available
        if self.cookies:
            self.set_cookies(self.cookies)
    
    def set_cookies(self, cookies: Dict[str, Any]):
        """Set session cookies"""
        self.cookies = cookies
        for name, value in cookies.items():
            self.session.cookies.set(name, value, domain='.gate.io')
    
    def get_cookies(self) -> Dict[str, str]:
        """Get current cookies"""
        return {cookie.name: cookie.value for cookie in self.session.cookies}
    
    def _rate_limit(self):
        """Enforce rate limiting"""
        current_time = time.time()
        time_since_last = current_time - self.last_request_time
        
        if time_since_last < self.min_request_interval:
            sleep_time = self.min_request_interval - time_since_last
            time.sleep(sleep_time)
        
        self.last_request_time = time.time()
    
    def _make_request(self, method: str, url: str, **kwargs) -> requests.Response:
        """Make rate-limited request"""
        self._rate_limit()
        
        try:
            response = self.session.request(method, url, **kwargs)
            response.raise_for_status()
            return response
        except requests.exceptions.RequestException as e:
            logger.error(f"Request failed for account {self.account_id}: {e}")
            raise
    
    def login(self, username: str, password: str) -> bool:
        """Login to Gate.io"""
        try:
            # Get login page to get CSRF token
            login_page = self._make_request('GET', f"{self.base_url}/login")
            
            # Extract CSRF token (this is a simplified example)
            # In reality, you'd need to parse the HTML or check cookies
            csrf_token = self.session.cookies.get('csrftoken', '')
            
            # Prepare login data
            login_data = {
                'email': username,
                'password': password,
                'csrfmiddlewaretoken': csrf_token,
                'remember': '1'
            }
            
            # Submit login
            headers = {
                'Content-Type': 'application/x-www-form-urlencoded',
                'X-CSRFToken': csrf_token
            }
            
            response = self._make_request(
                'POST',
                f"{self.base_url}/login",
                data=login_data,
                headers=headers,
                allow_redirects=False
            )
            
            # Check if login was successful
            if response.status_code in [302, 200]:
                # Save cookies
                self.cookies = self.get_cookies()
                logger.info(f"Successfully logged in account: {self.account_id}")
                return True
            else:
                logger.error(f"Login failed for account {self.account_id}")
                return False
                
        except Exception as e:
            logger.error(f"Login error for account {self.account_id}: {e}")
            return False
    
    def get_balance(self) -> Dict[str, float]:
        """Get account balance"""
        try:
            response = self._make_request('GET', f"{self.api_base}/private/balances")
            data = response.json()
            
            if data.get('result'):
                balances = {}
                for currency, info in data['result'].items():
                    available = float(info.get('available', 0))
                    if available > 0:
                        balances[currency] = available
                return balances
            else:
                logger.error(f"Failed to get balance for account {self.account_id}")
                return {}
                
        except Exception as e:
            logger.error(f"Error getting balance for account {self.account_id}: {e}")
            return {}
    
    def set_balance(self, currency: str, amount: float) -> bool:
        """Set virtual balance (for demo/testing)"""
        try:
            # This would be the endpoint for setting virtual balance
            # The actual endpoint would depend on Gate.io's API
            data = {
                'currency': currency,
                'amount': str(amount)
            }
            
            response = self._make_request(
                'POST',
                f"{self.api_base}/private/set_virtual_balance",
                json=data
            )
            
            result = response.json()
            if result.get('success'):
                logger.info(f"Set balance for {self.account_id}: {currency} = {amount}")
                return True
            else:
                logger.error(f"Failed to set balance for {self.account_id}")
                return False
                
        except Exception as e:
            logger.error(f"Error setting balance for account {self.account_id}: {e}")
            return False
    
    def get_pending_transactions(self) -> List[Dict[str, Any]]:
        """Get pending P2P transactions"""
        try:
            response = self._make_request(
                'GET',
                f"{self.api_base}/p2p/transactions",
                params={'status': 'pending'}
            )
            
            data = response.json()
            if data.get('success'):
                return data.get('transactions', [])
            else:
                logger.error(f"Failed to get pending transactions for {self.account_id}")
                return []
                
        except Exception as e:
            logger.error(f"Error getting pending transactions for {self.account_id}: {e}")
            return []
    
    def get_transaction_details(self, transaction_id: str) -> Optional[Dict[str, Any]]:
        """Get transaction details"""
        try:
            response = self._make_request(
                'GET',
                f"{self.api_base}/p2p/transactions/{transaction_id}"
            )
            
            data = response.json()
            if data.get('success'):
                return data.get('transaction')
            else:
                logger.error(f"Failed to get transaction {transaction_id} for {self.account_id}")
                return None
                
        except Exception as e:
            logger.error(f"Error getting transaction {transaction_id} for {self.account_id}: {e}")
            return None
    
    def approve_transaction(self, transaction_id: str) -> bool:
        """Approve a P2P transaction"""
        try:
            response = self._make_request(
                'POST',
                f"{self.api_base}/p2p/transactions/{transaction_id}/approve"
            )
            
            data = response.json()
            if data.get('success'):
                logger.info(f"Approved transaction {transaction_id} for {self.account_id}")
                return True
            else:
                logger.error(f"Failed to approve transaction {transaction_id} for {self.account_id}")
                return False
                
        except Exception as e:
            logger.error(f"Error approving transaction {transaction_id} for {self.account_id}: {e}")
            return False
    
    def reject_transaction(self, transaction_id: str, reason: str) -> bool:
        """Reject a P2P transaction"""
        try:
            data = {'reason': reason}
            response = self._make_request(
                'POST',
                f"{self.api_base}/p2p/transactions/{transaction_id}/reject",
                json=data
            )
            
            result = response.json()
            if result.get('success'):
                logger.info(f"Rejected transaction {transaction_id} for {self.account_id}")
                return True
            else:
                logger.error(f"Failed to reject transaction {transaction_id} for {self.account_id}")
                return False
                
        except Exception as e:
            logger.error(f"Error rejecting transaction {transaction_id} for {self.account_id}: {e}")
            return False
    
    def send_chat_message(self, transaction_id: str, message: str) -> bool:
        """Send chat message in transaction"""
        try:
            data = {'message': message}
            response = self._make_request(
                'POST',
                f"{self.api_base}/p2p/transactions/{transaction_id}/chat",
                json=data
            )
            
            result = response.json()
            if result.get('success'):
                logger.info(f"Sent message in transaction {transaction_id} for {self.account_id}")
                return True
            else:
                logger.error(f"Failed to send message in transaction {transaction_id}")
                return False
                
        except Exception as e:
            logger.error(f"Error sending message in transaction {transaction_id}: {e}")
            return False
    
    def get_chat_messages(self, transaction_id: str) -> List[Dict[str, Any]]:
        """Get chat messages for transaction"""
        try:
            response = self._make_request(
                'GET',
                f"{self.api_base}/p2p/transactions/{transaction_id}/chat"
            )
            
            data = response.json()
            if data.get('success'):
                return data.get('messages', [])
            else:
                logger.error(f"Failed to get chat messages for transaction {transaction_id}")
                return []
                
        except Exception as e:
            logger.error(f"Error getting chat messages for transaction {transaction_id}: {e}")
            return []
    
    def is_logged_in(self) -> bool:
        """Check if currently logged in"""
        try:
            response = self._make_request('GET', f"{self.api_base}/private/account")
            data = response.json()
            return data.get('success', False)
        except:
            return False


class GateMultiClient:
    """Manages multiple Gate.io clients"""
    
    def __init__(self, account_manager, db_manager):
        self.account_manager = account_manager
        self.db_manager = db_manager
        self.clients: Dict[str, GateClient] = {}
        self.executor = ThreadPoolExecutor(max_workers=10)
        
    def add_client(self, account_id: str) -> Optional[GateClient]:
        """Add a new client for an account"""
        account = self.account_manager.get_gate_account(account_id)
        if not account:
            logger.error(f"Account not found: {account_id}")
            return None
        
        # Create client with saved cookies
        client = GateClient(account_id, account.cookies)
        self.clients[account_id] = client
        
        # Store client in account manager
        self.account_manager.set_account_client(account_id, client, "gate")
        
        return client
    
    def remove_client(self, account_id: str):
        """Remove a client"""
        if account_id in self.clients:
            del self.clients[account_id]
            logger.info(f"Removed Gate.io client: {account_id}")
    
    def get_client(self, account_id: str) -> Optional[GateClient]:
        """Get a client by account ID"""
        return self.clients.get(account_id)
    
    async def login_all(self) -> Dict[str, bool]:
        """Login all accounts"""
        results = {}
        tasks = []
        
        for account_id in self.account_manager.list_gate_accounts():
            account = self.account_manager.get_gate_account(account_id)
            if account and account.status != "active":
                task = asyncio.create_task(self._login_account(account_id))
                tasks.append((account_id, task))
        
        for account_id, task in tasks:
            try:
                success = await task
                results[account_id] = success
            except Exception as e:
                logger.error(f"Failed to login account {account_id}: {e}")
                results[account_id] = False
        
        return results
    
    async def _login_account(self, account_id: str) -> bool:
        """Login a single account"""
        account = self.account_manager.get_gate_account(account_id)
        if not account:
            return False
        
        # Get or create client
        client = self.get_client(account_id)
        if not client:
            client = self.add_client(account_id)
            if not client:
                return False
        
        # Login
        loop = asyncio.get_event_loop()
        success = await loop.run_in_executor(
            self.executor,
            client.login,
            account.credentials["login"],
            account.credentials["password"]
        )
        
        if success:
            # Save cookies
            cookies = client.get_cookies()
            self.account_manager.update_gate_cookies(account_id, cookies)
            self.account_manager.update_gate_status(account_id, "active")
        else:
            self.account_manager.update_gate_status(account_id, "error", "Login failed")
        
        return success
    
    async def relogin_needed_accounts(self) -> Dict[str, bool]:
        """Re-login accounts that need it"""
        accounts_to_relogin = self.account_manager.get_gate_accounts_needing_relogin()
        results = {}
        
        for account_id in accounts_to_relogin:
            try:
                success = await self._login_account(account_id)
                results[account_id] = success
            except Exception as e:
                logger.error(f"Failed to relogin account {account_id}: {e}")
                results[account_id] = False
        
        return results
    
    async def update_all_balances(self, amount: float = 10_000_000) -> Dict[str, bool]:
        """Update balance for all accounts to specified amount"""
        accounts_to_update = self.account_manager.get_gate_accounts_needing_balance_update()
        results = {}
        
        for account_id in accounts_to_update:
            try:
                success = await self._update_balance(account_id, "RUB", amount)
                results[account_id] = success
            except Exception as e:
                logger.error(f"Failed to update balance for {account_id}: {e}")
                results[account_id] = False
        
        return results
    
    async def _update_balance(self, account_id: str, currency: str, amount: float) -> bool:
        """Update balance for a single account"""
        client = self.get_client(account_id)
        if not client:
            client = self.add_client(account_id)
            if not client:
                return False
        
        # Update balance
        loop = asyncio.get_event_loop()
        success = await loop.run_in_executor(
            self.executor,
            client.set_balance,
            currency,
            amount
        )
        
        if success:
            self.account_manager.update_gate_balance(account_id, amount)
        
        return success
    
    async def get_all_pending_transactions(self) -> Dict[str, List[Dict[str, Any]]]:
        """Get pending transactions from all accounts"""
        active_accounts = [
            acc_id for acc_id in self.account_manager.list_gate_accounts()
            if self.account_manager.get_gate_account(acc_id).status == "active"
        ]
        
        results = {}
        
        for account_id in active_accounts:
            try:
                transactions = await self._get_pending_transactions(account_id)
                results[account_id] = transactions
            except Exception as e:
                logger.error(f"Failed to get transactions for {account_id}: {e}")
                results[account_id] = []
        
        return results
    
    async def _get_pending_transactions(self, account_id: str) -> List[Dict[str, Any]]:
        """Get pending transactions for a single account"""
        client = self.get_client(account_id)
        if not client:
            return []
        
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            client.get_pending_transactions
        )
    
    def cleanup(self):
        """Cleanup resources"""
        self.executor.shutdown(wait=True)
        self.clients.clear()
        logger.info("Gate.io multi-client cleaned up")