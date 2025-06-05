"""
Multi-Account Manager for Trading System
Manages multiple Gate.io and Bybit accounts
"""

import logging
import asyncio
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
import uuid
from concurrent.futures import ThreadPoolExecutor

from db_manager import DatabaseManager

logger = logging.getLogger(__name__)


class Account:
    """Base account class"""
    
    def __init__(self, account_id: str, account_type: str, credentials: Dict[str, Any]):
        self.id = account_id
        self.type = account_type
        self.credentials = credentials
        self.status = "inactive"
        self.last_login = None
        self.last_error = None
        self.client = None
        
    def to_dict(self) -> Dict[str, Any]:
        """Convert account to dictionary"""
        return {
            "id": self.id,
            "type": self.type,
            "status": self.status,
            "last_login": self.last_login.isoformat() if self.last_login else None,
            "last_error": self.last_error
        }


class GateAccount(Account):
    """Gate.io account"""
    
    def __init__(self, account_id: str, login: str, password: str):
        super().__init__(account_id, "gate", {"login": login, "password": password})
        self.cookies = None
        self.balance = 0
        self.last_balance_update = None
        

class BybitAccount(Account):
    """Bybit account"""
    
    def __init__(self, account_id: str, api_key: str, api_secret: str, testnet: bool = False):
        super().__init__(account_id, "bybit", {"api_key": api_key, "api_secret": api_secret})
        self.api_key = api_key
        self.api_secret = api_secret
        self.testnet = testnet
        self.active_ads = []
        self.active_orders = []


class AccountManager:
    """Manages multiple trading accounts"""
    
    def __init__(self, db_manager: DatabaseManager):
        self.db = db_manager
        self.gate_accounts: Dict[str, GateAccount] = {}
        self.bybit_accounts: Dict[str, BybitAccount] = {}
        self.executor = ThreadPoolExecutor(max_workers=10)
        self._load_accounts()
        
    def _load_accounts(self):
        """Load all accounts from database"""
        # Load Gate.io accounts
        for account_id in self.db.list_gate_accounts():
            account_data = self.db.load_gate_account(account_id)
            if account_data:
                account = GateAccount(
                    account_id,
                    account_data["login"],
                    account_data["password"]
                )
                account.status = account_data.get("status", "inactive")
                
                # Load cookies if available
                cookies = self.db.load_gate_cookies(account_id)
                if cookies:
                    account.cookies = cookies
                    
                self.gate_accounts[account_id] = account
                logger.info(f"Loaded Gate.io account: {account_id}")
        
        # Load Bybit accounts
        for account_id in self.db.list_bybit_accounts():
            account_data = self.db.load_bybit_account(account_id)
            if account_data:
                account = BybitAccount(
                    account_id,
                    account_data["api_key"],
                    account_data["api_secret"],
                    account_data.get("testnet", False)
                )
                account.status = account_data.get("status", "inactive")
                self.bybit_accounts[account_id] = account
                logger.info(f"Loaded Bybit account: {account_id}")
    
    # Gate.io Account Management
    def add_gate_account(self, login: str, password: str, account_id: Optional[str] = None) -> str:
        """Add new Gate.io account"""
        account_id = account_id or str(uuid.uuid4())
        
        # Save to database
        if self.db.save_gate_account(account_id, login, password):
            # Create account object
            account = GateAccount(account_id, login, password)
            self.gate_accounts[account_id] = account
            
            logger.info(f"Added Gate.io account: {account_id}")
            return account_id
        else:
            raise Exception("Failed to save Gate.io account")
    
    def remove_gate_account(self, account_id: str) -> bool:
        """Remove Gate.io account"""
        if account_id in self.gate_accounts:
            # Remove from database
            if self.db.delete_gate_account(account_id):
                # Remove from memory
                del self.gate_accounts[account_id]
                logger.info(f"Removed Gate.io account: {account_id}")
                return True
        return False
    
    def get_gate_account(self, account_id: str) -> Optional[GateAccount]:
        """Get Gate.io account by ID"""
        return self.gate_accounts.get(account_id)
    
    def list_gate_accounts(self) -> List[str]:
        """List all Gate.io account IDs"""
        return list(self.gate_accounts.keys())
    
    def list_gate_accounts_full(self) -> List[Dict[str, Any]]:
        """List all Gate.io accounts with full details"""
        return [acc.to_dict() for acc in self.gate_accounts.values()]
    
    def update_gate_cookies(self, account_id: str, cookies: Dict[str, Any]) -> bool:
        """Update Gate.io account cookies"""
        account = self.gate_accounts.get(account_id)
        if account:
            account.cookies = cookies
            account.last_login = datetime.now()
            account.status = "active"
            
            # Save to database
            return self.db.save_gate_cookies(account_id, cookies)
        return False
    
    def update_gate_balance(self, account_id: str, balance: float) -> bool:
        """Update Gate.io account balance"""
        account = self.gate_accounts.get(account_id)
        if account:
            account.balance = balance
            account.last_balance_update = datetime.now()
            
            # Save to database
            account_data = self.db.load_gate_account(account_id)
            if account_data:
                account_data["balance"] = balance
                account_data["last_balance_update"] = account.last_balance_update.isoformat()
                return self.db.save_gate_account(
                    account_id,
                    account.credentials["login"],
                    account.credentials["password"],
                    account_data
                )
        return False
    
    def get_gate_accounts_needing_balance_update(self, hours: int = 4) -> List[str]:
        """Get Gate.io accounts that need balance update"""
        accounts_to_update = []
        cutoff_time = datetime.now() - timedelta(hours=hours)
        
        for account_id, account in self.gate_accounts.items():
            if account.status == "active":
                if not account.last_balance_update or account.last_balance_update < cutoff_time:
                    accounts_to_update.append(account_id)
                    
        return accounts_to_update
    
    def get_gate_accounts_needing_relogin(self, minutes: int = 30) -> List[str]:
        """Get Gate.io accounts that need re-login"""
        accounts_to_relogin = []
        cutoff_time = datetime.now() - timedelta(minutes=minutes)
        
        for account_id, account in self.gate_accounts.items():
            if account.status == "active" or account.status == "inactive":
                if not account.last_login or account.last_login < cutoff_time:
                    accounts_to_relogin.append(account_id)
                    
        return accounts_to_relogin
    
    # Bybit Account Management
    def add_bybit_account(self, api_key: str, api_secret: str, testnet: bool = False, account_id: Optional[str] = None) -> str:
        """Add new Bybit account"""
        account_id = account_id or str(uuid.uuid4())
        
        # Save to database
        if self.db.save_bybit_account(account_id, api_key, api_secret, {"testnet": testnet}):
            # Create account object
            account = BybitAccount(account_id, api_key, api_secret, testnet)
            self.bybit_accounts[account_id] = account
            
            logger.info(f"Added Bybit account: {account_id}")
            return account_id
        else:
            raise Exception("Failed to save Bybit account")
    
    def remove_bybit_account(self, account_id: str) -> bool:
        """Remove Bybit account"""
        if account_id in self.bybit_accounts:
            # Remove from database
            if self.db.delete_bybit_account(account_id):
                # Remove from memory
                del self.bybit_accounts[account_id]
                logger.info(f"Removed Bybit account: {account_id}")
                return True
        return False
    
    def get_bybit_account(self, account_id: str) -> Optional[BybitAccount]:
        """Get Bybit account by ID"""
        return self.bybit_accounts.get(account_id)
    
    def list_bybit_accounts(self) -> List[str]:
        """List all Bybit account IDs"""
        return list(self.bybit_accounts.keys())
    
    def list_bybit_accounts_full(self) -> List[Dict[str, Any]]:
        """List all Bybit accounts with full details"""
        return [acc.to_dict() for acc in self.bybit_accounts.values()]
    
    def update_bybit_status(self, account_id: str, status: str, error: Optional[str] = None) -> bool:
        """Update Bybit account status"""
        account = self.bybit_accounts.get(account_id)
        if account:
            account.status = status
            account.last_error = error
            if status == "active":
                account.last_login = datetime.now()
                
            # Save to database
            account_data = self.db.load_bybit_account(account_id)
            if account_data:
                account_data["status"] = status
                account_data["last_error"] = error
                account_data["last_login"] = account.last_login.isoformat() if account.last_login else None
                return self.db.save_bybit_account(
                    account_id,
                    account.credentials["api_key"],
                    account.credentials["api_secret"],
                    account_data
                )
        return False
    
    # Utility Methods
    def get_all_active_accounts(self) -> Dict[str, List[str]]:
        """Get all active accounts by type"""
        active_accounts = {
            "gate": [],
            "bybit": []
        }
        
        for account_id, account in self.gate_accounts.items():
            if account.status == "active":
                active_accounts["gate"].append(account_id)
                
        for account_id, account in self.bybit_accounts.items():
            if account.status == "active":
                active_accounts["bybit"].append(account_id)
                
        return active_accounts
    
    def get_account_stats(self) -> Dict[str, Any]:
        """Get statistics for all accounts"""
        stats = {
            "gate": {
                "total": len(self.gate_accounts),
                "active": sum(1 for acc in self.gate_accounts.values() if acc.status == "active"),
                "inactive": sum(1 for acc in self.gate_accounts.values() if acc.status == "inactive"),
                "error": sum(1 for acc in self.gate_accounts.values() if acc.status == "error")
            },
            "bybit": {
                "total": len(self.bybit_accounts),
                "active": sum(1 for acc in self.bybit_accounts.values() if acc.status == "active"),
                "inactive": sum(1 for acc in self.bybit_accounts.values() if acc.status == "inactive"),
                "error": sum(1 for acc in self.bybit_accounts.values() if acc.status == "error")
            }
        }
        
        return stats
    
    async def parallel_operation(self, account_ids: List[str], operation_func, *args, **kwargs) -> Dict[str, Any]:
        """Execute operation on multiple accounts in parallel"""
        tasks = []
        results = {}
        
        for account_id in account_ids:
            task = asyncio.create_task(
                self._run_account_operation(account_id, operation_func, *args, **kwargs)
            )
            tasks.append((account_id, task))
            
        for account_id, task in tasks:
            try:
                result = await task
                results[account_id] = {"success": True, "result": result}
            except Exception as e:
                logger.error(f"Operation failed for account {account_id}: {e}")
                results[account_id] = {"success": False, "error": str(e)}
                
        return results
    
    async def _run_account_operation(self, account_id: str, operation_func, *args, **kwargs):
        """Run operation for a single account"""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            operation_func,
            account_id,
            *args,
            **kwargs
        )
    
    def set_account_client(self, account_id: str, client: Any, account_type: str = "gate"):
        """Set client instance for an account"""
        if account_type == "gate" and account_id in self.gate_accounts:
            self.gate_accounts[account_id].client = client
        elif account_type == "bybit" and account_id in self.bybit_accounts:
            self.bybit_accounts[account_id].client = client
    
    def get_account_client(self, account_id: str, account_type: str = "gate") -> Optional[Any]:
        """Get client instance for an account"""
        if account_type == "gate" and account_id in self.gate_accounts:
            return self.gate_accounts[account_id].client
        elif account_type == "bybit" and account_id in self.bybit_accounts:
            return self.bybit_accounts[account_id].client
        return None
    
    def cleanup(self):
        """Cleanup resources"""
        self.executor.shutdown(wait=True)
        logger.info("Account manager cleaned up")