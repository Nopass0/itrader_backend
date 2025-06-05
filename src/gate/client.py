#!/usr/bin/env python3
"""
Gate.io Client - Python Implementation
Complete port from Rust with all API functionality
"""

import json
import time
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
from decimal import Decimal
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

logger = logging.getLogger(__name__)

class Cookie:
    """Cookie model"""
    def __init__(self, name: str, value: str, domain: str = ".panel.gate.cx", 
                 path: str = "/", secure: bool = True, http_only: bool = True,
                 expiration_date: Optional[float] = None):
        self.name = name
        self.value = value
        self.domain = domain
        self.path = path
        self.secure = secure
        self.http_only = http_only
        self.same_site = None
        self.session = False
        self.host_only = False
        self.store_id = None
        self.expiration_date = expiration_date or (time.time() + 30 * 24 * 60 * 60)  # 30 days
    
    def to_dict(self):
        return {
            "name": self.name,
            "value": self.value,
            "domain": self.domain,
            "path": self.path,
            "secure": self.secure,
            "httpOnly": self.http_only,
            "sameSite": self.same_site,
            "session": self.session,
            "hostOnly": self.host_only,
            "storeId": self.store_id,
            "expirationDate": self.expiration_date
        }
    
    @classmethod
    def from_dict(cls, data: dict):
        cookie = cls(
            name=data["name"],
            value=data["value"],
            domain=data.get("domain", ".panel.gate.cx"),
            path=data.get("path", "/"),
            secure=data.get("secure", True),
            http_only=data.get("httpOnly", True),
            expiration_date=data.get("expirationDate")
        )
        cookie.same_site = data.get("sameSite")
        cookie.session = data.get("session", False)
        cookie.host_only = data.get("hostOnly", False)
        cookie.store_id = data.get("storeId")
        return cookie


class GateClient:
    """Gate.io API Client"""
    
    def __init__(self, login: str = None, password: str = None, 
                 base_url: str = "https://panel.gate.cx/api/v1"):
        self.login_email = login
        self.password = password
        self.base_url = base_url
        self.cookies: List[Cookie] = []
        
        # Setup session with retry strategy
        self.session = requests.Session()
        retry_strategy = Retry(
            total=3,
            status_forcelist=[429, 500, 502, 503, 504],
            method_whitelist=["HEAD", "GET", "OPTIONS", "POST"],
            backoff_factor=1
        )
        adapter = HTTPAdapter(max_retries=retry_strategy)
        self.session.mount("http://", adapter)
        self.session.mount("https://", adapter)
        
        # Set default headers
        self.session.headers.update({
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "Accept": "application/json, text/plain, */*",
            "Accept-Language": "en-US,en;q=0.9",
            "Accept-Encoding": "identity",
            "Referer": "https://panel.gate.cx/",
            "Origin": "https://panel.gate.cx",
            "DNT": "1"
        })
    
    def _update_cookies(self):
        """Update session cookies from stored cookies"""
        cookie_dict = {}
        for cookie in self.cookies:
            cookie_dict[cookie.name] = cookie.value
        self.session.cookies.update(cookie_dict)
    
    def _parse_set_cookies(self, response):
        """Parse Set-Cookie headers from response"""
        new_cookies = []
        for cookie_str in response.headers.get("Set-Cookie", "").split("\n"):
            if not cookie_str:
                continue
            
            parts = cookie_str.split(";")
            if not parts:
                continue
            
            # Parse name=value
            name_value = parts[0].strip().split("=", 1)
            if len(name_value) != 2:
                continue
            
            cookie = Cookie(name=name_value[0], value=name_value[1])
            
            # Parse attributes
            for part in parts[1:]:
                part = part.strip()
                if "=" in part:
                    key, val = part.split("=", 1)
                    key = key.lower()
                    if key == "domain":
                        cookie.domain = val
                    elif key == "path":
                        cookie.path = val
                    elif key == "max-age":
                        try:
                            cookie.expiration_date = time.time() + int(val)
                        except:
                            pass
                else:
                    if part.lower() == "secure":
                        cookie.secure = True
                    elif part.lower() == "httponly":
                        cookie.http_only = True
            
            new_cookies.append(cookie)
        
        return new_cookies
    
    async def login(self, email: str = None, password: str = None) -> Dict[str, Any]:
        """Login to Gate.io"""
        email = email or self.login_email
        password = password or self.password
        
        if not email or not password:
            raise ValueError("Email and password required")
        
        url = f"{self.base_url}/auth/basic/login"
        data = {
            "login": email,
            "password": password
        }
        
        logger.info(f"Attempting login to: {url}")
        
        response = self.session.post(url, json=data)
        
        # Check for Cloudflare block
        if response.status_code == 403:
            logger.warning("Cloudflare challenge detected")
            raise Exception("Cloudflare block detected")
        
        # Parse cookies from response
        new_cookies = self._parse_set_cookies(response)
        if new_cookies:
            self.cookies = new_cookies
            self._update_cookies()
            logger.info(f"Stored {len(new_cookies)} cookies from login response")
        
        # If we got cookies, consider it successful
        if self.cookies:
            logger.info(f"Successfully logged into Gate.io account: {email}")
            return {
                "success": True,
                "user_id": "unknown",
                "session_id": "from_cookies",
                "expires_at": datetime.utcnow() + timedelta(days=1)
            }
        
        # Try to parse response
        try:
            data = response.json()
            if not data.get("success"):
                error = data.get("error", "Unknown error")
                raise Exception(f"Login failed: {error}")
            
            return data.get("response", {})
        except json.JSONDecodeError:
            if response.status_code == 200:
                # Sometimes login succeeds without proper JSON response
                return {
                    "success": True,
                    "user_id": "unknown",
                    "session_id": "unknown"
                }
            raise Exception(f"Login failed with status {response.status_code}")
    
    def set_cookies(self, cookies: List[Dict[str, Any]]):
        """Set cookies from list of cookie dicts"""
        self.cookies = [Cookie.from_dict(c) for c in cookies]
        self._update_cookies()
        logger.info(f"Set {len(self.cookies)} cookies for Gate.io client")
    
    def get_cookies(self) -> List[Dict[str, Any]]:
        """Get current cookies as list of dicts"""
        return [c.to_dict() for c in self.cookies]
    
    def load_cookies(self, file_path: str):
        """Load cookies from file"""
        with open(file_path, 'r') as f:
            cookies_data = json.load(f)
        self.set_cookies(cookies_data)
    
    def save_cookies(self, file_path: str):
        """Save cookies to file"""
        with open(file_path, 'w') as f:
            json.dump(self.get_cookies(), f, indent=2)
    
    async def get_balance(self, currency: str = "RUB") -> Dict[str, Any]:
        """Get balance for specified currency"""
        url = f"{self.base_url}/auth/me"
        response = self.session.get(url)
        
        if response.status_code == 401:
            raise Exception("Session expired")
        elif response.status_code == 403:
            raise Exception("Cloudflare block")
        
        data = response.json()
        if not data.get("success"):
            raise Exception(f"Failed to get balance: {data.get('error', 'Unknown error')}")
        
        user_info = data.get("response", {}).get("user", {})
        wallets = user_info.get("wallets", [])
        
        # Find wallet for requested currency
        for wallet in wallets:
            wallet_currency = wallet.get("currency", {}).get("code", "")
            if wallet_currency.upper() == currency.upper() or \
               (wallet_currency == "643" and currency.upper() == "RUB"):
                balance = Decimal(wallet.get("balance", "0"))
                return {
                    "currency": currency,
                    "balance": float(balance),
                    "available": float(balance),
                    "locked": 0.0
                }
        
        raise Exception(f"Wallet for currency {currency} not found")
    
    async def set_balance(self, amount: float) -> float:
        """Set balance (update balance on Gate.io)"""
        url = f"{self.base_url}/payments/payouts/balance"
        data = {"amount": str(amount)}
        
        response = self.session.post(url, json=data)
        
        if not response.ok:
            raise Exception(f"Failed to set balance: HTTP {response.status_code}")
        
        result = response.json()
        if not result.get("success"):
            raise Exception(f"Failed to set balance: {result.get('error', 'Unknown error')}")
        
        logger.info(f"Successfully set balance to {amount}")
        return amount
    
    def get_pending_transactions(self) -> List[Dict[str, Any]]:
        """Get pending transactions (status 4 or 5)"""
        return self.get_available_transactions()
    
    def get_available_transactions(self) -> List[Dict[str, Any]]:
        """Get available transactions with status 4 or 5"""
        url = f"{self.base_url}/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1"
        
        logger.debug(f"Getting available transactions from: {url}")
        
        response = self.session.get(url)
        
        if not response.ok:
            logger.warning(f"Failed to get transactions: {response.status_code}")
            return []
        
        try:
            data = response.json()
            if data.get("success"):
                payouts = data.get("response", {}).get("payouts", {}).get("data", [])
                
                # Filter and format transactions
                transactions = []
                for payout in payouts:
                    # Skip if empty amounts
                    if not payout.get("amount", {}).get("trader") or \
                       not payout.get("total", {}).get("trader"):
                        continue
                    
                    # Extract RUB amount
                    rub_amount = payout.get("amount", {}).get("trader", {}).get("643", 0)
                    rub_total = payout.get("total", {}).get("trader", {}).get("643", 0)
                    
                    if rub_amount > 0:
                        tx = {
                            "id": str(payout.get("id")),
                            "status": payout.get("status"),
                            "amount": {
                                "trader": {
                                    "643": rub_amount
                                }
                            },
                            "total": {
                                "trader": {
                                    "643": rub_total
                                }
                            },
                            "wallet": payout.get("wallet", ""),
                            "method": payout.get("method", {}),
                            "bank": payout.get("bank", {}),
                            "created_at": payout.get("created_at"),
                            "updated_at": payout.get("updated_at"),
                            "meta": payout.get("meta", {})
                        }
                        transactions.append(tx)
                
                logger.info(f"Found {len(transactions)} available transactions")
                return transactions
            
        except Exception as e:
            logger.error(f"Error parsing transactions: {e}")
        
        return []
    
    async def get_transactions(self, page: int = 1, per_page: int = 30) -> List[Dict[str, Any]]:
        """Get all transactions with pagination"""
        url = f"{self.base_url}/payments/payouts"
        params = {
            "page": page,
            "per_page": per_page
        }
        
        response = self.session.get(url, params=params)
        
        if not response.ok:
            raise Exception(f"Failed to get transactions: HTTP {response.status_code}")
        
        data = response.json()
        if not data.get("success"):
            raise Exception(f"Failed to get transactions: {data.get('error', 'Unknown error')}")
        
        payouts = data.get("response", {}).get("payouts", {}).get("data", [])
        return self._format_transactions(payouts)
    
    async def get_in_progress_transactions(self) -> List[Dict[str, Any]]:
        """Get transactions with status 5 (in progress)"""
        all_transactions = await self.get_transactions()
        return [tx for tx in all_transactions if tx.get("status") == 5]
    
    async def get_history_transactions(self, page: int = 1) -> List[Dict[str, Any]]:
        """Get completed/history transactions"""
        all_transactions = await self.get_transactions(page=page)
        # Status 7 with approved_at or status 9
        return [
            tx for tx in all_transactions 
            if (tx.get("status") == 7 and tx.get("approved_at")) or tx.get("status") == 9
        ]
    
    async def accept_transaction(self, transaction_id: str) -> bool:
        """Accept a transaction (change status from 4 to 5)"""
        url = f"{self.base_url}/payments/payouts/{transaction_id}/show"
        
        logger.info(f"Accepting transaction {transaction_id} via /show endpoint")
        
        response = self.session.post(url)
        
        # Handle various response codes
        if response.status_code in [409, 422, 400]:
            # Transaction might already be accepted
            try:
                data = response.json()
                error_desc = data.get("response", {}).get("error_description", "")
                message = data.get("message", "")
                
                if "incorrect_status" in error_desc or "already" in message:
                    logger.warning(f"Transaction {transaction_id} already processed")
                    return True
            except:
                pass
        
        if not response.ok:
            raise Exception(f"Failed to accept transaction: HTTP {response.status_code}")
        
        logger.info(f"Successfully accepted transaction: {transaction_id}")
        return True
    
    async def approve_transaction(self, transaction_id: str, pdf_path: str = None) -> Dict[str, Any]:
        """Approve transaction with or without receipt"""
        url = f"{self.base_url}/payments/payouts/{transaction_id}/approve"
        
        if pdf_path:
            # Multipart upload with PDF
            logger.info(f"Approving transaction {transaction_id} with receipt: {pdf_path}")
            
            with open(pdf_path, 'rb') as f:
                files = {'attachments[]': (pdf_path.split('/')[-1], f, 'application/pdf')}
                response = self.session.post(url, files=files)
        else:
            # Simple approval without receipt
            logger.info(f"Approving transaction {transaction_id} without receipt")
            response = self.session.post(url)
        
        if not response.ok:
            raise Exception(f"Failed to approve transaction: HTTP {response.status_code}")
        
        data = response.json()
        if not data.get("success"):
            raise Exception(f"Failed to approve transaction: {data.get('error', 'Unknown error')}")
        
        return data.get("response", {}).get("payout", {})
    
    async def cancel_order(self, transaction_id: str) -> Dict[str, Any]:
        """Cancel an order"""
        url = f"{self.base_url}/payments/payouts/{transaction_id}/cancel"
        
        logger.info(f"Cancelling order {transaction_id}")
        
        response = self.session.post(url)
        
        if not response.ok:
            raise Exception(f"Failed to cancel order: HTTP {response.status_code}")
        
        data = response.json()
        if not data.get("success"):
            raise Exception(f"Failed to cancel order: {data.get('error', 'Unknown error')}")
        
        return data.get("response", {}).get("payout", {})
    
    async def get_transaction_details(self, transaction_id: str) -> Dict[str, Any]:
        """Get detailed information about a transaction"""
        # Try multiple endpoints
        for endpoint in [f"/payments/payouts/{transaction_id}/", f"/payments/payouts/{transaction_id}"]:
            url = f"{self.base_url}{endpoint}"
            response = self.session.get(url)
            
            if response.ok:
                data = response.json()
                if data.get("success"):
                    # Try different response formats
                    if "payout" in data.get("response", {}):
                        return data["response"]["payout"]
                    elif "response" in data and isinstance(data["response"], dict):
                        return data["response"]
        
        raise Exception(f"Failed to get transaction details for {transaction_id}")
    
    async def search_transaction_by_id(self, transaction_id: str) -> Optional[Dict[str, Any]]:
        """Search for transaction by ID"""
        url = f"{self.base_url}/payments/payouts"
        params = {
            "search[id]": transaction_id,
            "filters[status][]": [4, 5],
            "page": 1
        }
        
        response = self.session.get(url, params=params)
        
        if response.ok:
            data = response.json()
            if data.get("success"):
                payouts = data.get("response", {}).get("payouts", {}).get("data", [])
                if payouts:
                    return payouts[0]
        
        return None
    
    async def update_balance(self, amount: float) -> Dict[str, Any]:
        """Update account balance"""
        url = f"{self.base_url}/account/balance/update"
        data = {
            "amount": str(amount),
            "currency": "RUB"
        }
        
        logger.info(f"Updating balance to {amount}")
        
        response = self.session.post(url, json=data)
        
        if not response.ok:
            raise Exception(f"Failed to update balance: HTTP {response.status_code}")
        
        return response.json()
    
    async def is_authenticated(self) -> bool:
        """Check if client is authenticated"""
        if not self.cookies:
            return False
        
        try:
            # Try to get balance as auth check
            await self.get_balance("RUB")
            return True
        except:
            return False
    
    def _format_transactions(self, payouts: List[Dict]) -> List[Dict[str, Any]]:
        """Format payout data to transaction format"""
        transactions = []
        
        for payout in payouts:
            # Skip empty transactions
            if not payout.get("amount", {}).get("trader"):
                continue
            
            # Extract RUB amounts
            rub_amount = payout.get("amount", {}).get("trader", {}).get("643", 0)
            rub_total = payout.get("total", {}).get("trader", {}).get("643", 0)
            
            tx = {
                "id": str(payout.get("id")),
                "order_id": str(payout.get("id")),
                "amount": float(rub_amount) if rub_amount else 0.0,
                "currency": "RUB",
                "fiat_currency": "RUB",
                "fiat_amount": float(rub_total) if rub_total else 0.0,
                "rate": 1.0,
                "status": payout.get("status"),
                "buyer_name": payout.get("trader", {}).get("name", "Unknown"),
                "payment_method": payout.get("method", {}).get("label", ""),
                "created_at": payout.get("created_at"),
                "updated_at": payout.get("updated_at"),
                "wallet": payout.get("wallet", ""),
                "bank": payout.get("bank", {}),
                "meta": payout.get("meta", {}),
                "approved_at": payout.get("approved_at"),
                "attachments": payout.get("attachments", [])
            }
            
            transactions.append(tx)
        
        return transactions


# Async wrapper for synchronous client
class AsyncGateClient:
    """Async wrapper for GateClient to maintain compatibility"""
    
    def __init__(self, login: str = None, password: str = None, base_url: str = None):
        self.client = GateClient(login, password, base_url or "https://panel.gate.cx/api/v1")
    
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        pass
    
    def __getattr__(self, name):
        # Delegate all method calls to the sync client
        return getattr(self.client, name)