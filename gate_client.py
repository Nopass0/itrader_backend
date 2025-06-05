"""
Gate.io API Client
Handles all interactions with Gate.io using cookies authentication
"""

import aiohttp
import asyncio
import json
import logging
from datetime import datetime, timezone
from decimal import Decimal
from typing import List, Dict, Any, Optional
from urllib.parse import urljoin

from models import Transaction
from utils import RateLimiter

logger = logging.getLogger(__name__)


class GateClient:
    """Gate.io API client with cookie authentication"""
    
    BASE_URL = "https://panel.gate.cx"
    API_BASE = "https://panel.gate.cx/api/v1"
    
    def __init__(self, config):
        self.config = config
        self.cookies = self._load_cookies()
        self.session: Optional[aiohttp.ClientSession] = None
        self.rate_limiter = RateLimiter(max_requests=240, window_seconds=60)  # 240 req/min
        self.user_info = None
        
    def _load_cookies(self) -> Dict[str, str]:
        """Load cookies from .gate_cookies.json"""
        try:
            with open('.gate_cookies.json', 'r') as f:
                cookies_data = json.load(f)
                
            cookies = {}
            for cookie in cookies_data:
                cookies[cookie['name']] = cookie['value']
            
            return cookies
        except Exception as e:
            logger.error(f"Failed to load Gate.io cookies: {e}")
            raise Exception("Failed to load Gate.io cookies. Please ensure .gate_cookies.json exists")
    
    async def _get_session(self) -> aiohttp.ClientSession:
        """Get or create HTTP session"""
        if not self.session:
            self.session = aiohttp.ClientSession(
                headers={
                    'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
                    'Accept': 'application/json',
                    'Accept-Language': 'en-US,en;q=0.9',
                    'Referer': 'https://panel.gate.cx/'
                },
                cookies=self.cookies
            )
        return self.session
    
    async def _request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        """Make rate-limited API request"""
        await self.rate_limiter.acquire()
        
        session = await self._get_session()
        url = urljoin(self.API_BASE, endpoint)
        
        try:
            async with session.request(method, url, **kwargs) as response:
                response.raise_for_status()
                return await response.json()
        except aiohttp.ClientError as e:
            logger.error(f"Gate.io API error: {e}")
            raise
    
    async def test_connection(self) -> bool:
        """Test connection to Gate.io"""
        try:
            # Get user info
            response = await self._request('GET', '/auth/me')
            if response and 'user' in response:
                self.user_info = response['user']
                logger.info(f"Connected to Gate.io as: {self.user_info.get('name', 'Unknown')}")
                return True
            return False
        except Exception as e:
            logger.error(f"Failed to connect to Gate.io: {e}")
            return False
    
    async def get_pending_transactions(self) -> List[Transaction]:
        """Get pending transactions from Gate.io"""
        try:
            # Get payouts with status 1 (pending)
            response = await self._request('GET', '/payouts', params={
                'status': 1,
                'page': 1,
                'per_page': 50
            })
            
            transactions = []
            if response and 'payouts' in response:
                for payout in response['payouts'].get('data', []):
                    tx = self._parse_transaction(payout)
                    if tx:
                        transactions.append(tx)
            
            return transactions
            
        except Exception as e:
            logger.error(f"Error getting pending transactions: {e}")
            return []
    
    def _parse_transaction(self, payout: Dict[str, Any]) -> Optional[Transaction]:
        """Parse payout data into Transaction object"""
        try:
            # Extract amounts
            amount_data = payout.get('amount', {}).get('trader', {})
            total_data = payout.get('total', {}).get('trader', {})
            
            # Get currency and amount
            currency = payout.get('wallet', 'USDT')
            amount = Decimal('0')
            fiat_amount = Decimal('0')
            fiat_currency = 'RUB'  # Default
            
            # Extract amount from trader data
            for curr, amt in amount_data.items():
                if curr == currency:
                    amount = Decimal(str(amt))
                elif curr in ['RUB', 'USD', 'EUR']:
                    fiat_currency = curr
                    fiat_amount = Decimal(str(amt))
            
            # If no fiat amount in amount_data, check total_data
            if fiat_amount == 0:
                for curr, amt in total_data.items():
                    if curr in ['RUB', 'USD', 'EUR']:
                        fiat_currency = curr
                        fiat_amount = Decimal(str(amt))
            
            # Calculate rate
            rate = fiat_amount / amount if amount > 0 else Decimal('0')
            
            # Get buyer info
            buyer_name = payout.get('trader', {}).get('name', 'Unknown')
            
            # Get payment method
            payment_method = payout.get('method', {}).get('label', 'Unknown')
            
            # Get bank info if available
            bank = payout.get('bank', {}).get('label', '')
            if bank:
                payment_method = f"{payment_method} ({bank})"
            
            # Create transaction
            tx = Transaction(
                id=str(payout.get('id')),
                order_id=f"GATE-{payout.get('id')}",
                amount=amount,
                currency=currency,
                fiat_amount=fiat_amount,
                fiat_currency=fiat_currency,
                rate=rate.quantize(Decimal('0.01')),
                status=payout.get('status', 1),
                buyer_name=buyer_name,
                payment_method=payment_method,
                created_at=datetime.fromisoformat(payout.get('created_at', '').replace('Z', '+00:00')),
                meta=payout.get('meta', {})
            )
            
            return tx
            
        except Exception as e:
            logger.error(f"Error parsing transaction: {e}")
            return None
    
    async def accept_transaction(self, transaction_id: str) -> bool:
        """Accept a pending transaction"""
        try:
            response = await self._request(
                'POST',
                f'/payouts/{transaction_id}/approve',
                json={'action': 'accept'}
            )
            
            if response and response.get('payout'):
                logger.info(f"Successfully accepted transaction {transaction_id}")
                return True
            return False
            
        except Exception as e:
            logger.error(f"Error accepting transaction {transaction_id}: {e}")
            return False
    
    async def complete_transaction(self, transaction_id: str, confirmation_code: Optional[str] = None) -> bool:
        """Complete a transaction"""
        try:
            data = {'action': 'complete'}
            if confirmation_code:
                data['confirmation_code'] = confirmation_code
            
            response = await self._request(
                'POST',
                f'/payouts/{transaction_id}/complete',
                json=data
            )
            
            if response and response.get('payout'):
                logger.info(f"Successfully completed transaction {transaction_id}")
                return True
            return False
            
        except Exception as e:
            logger.error(f"Error completing transaction {transaction_id}: {e}")
            return False
    
    async def get_transaction_details(self, transaction_id: str) -> Optional[Transaction]:
        """Get details of a specific transaction"""
        try:
            response = await self._request('GET', f'/payouts/{transaction_id}')
            
            if response and 'payout' in response:
                return self._parse_transaction(response['payout'])
            return None
            
        except Exception as e:
            logger.error(f"Error getting transaction details: {e}")
            return None
    
    async def get_balance(self, currency: str = 'USDT') -> Optional[Decimal]:
        """Get wallet balance"""
        try:
            if not self.user_info:
                await self.test_connection()
            
            if self.user_info:
                wallets = self.user_info.get('wallets', [])
                for wallet in wallets:
                    if wallet.get('currency', {}).get('code') == currency:
                        return Decimal(wallet.get('balance', '0'))
            
            return None
            
        except Exception as e:
            logger.error(f"Error getting balance: {e}")
            return None
    
    async def close(self):
        """Close the HTTP session"""
        if self.session:
            await self.session.close()
            self.session = None