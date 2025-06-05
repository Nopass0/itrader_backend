"""
Bybit P2P API Client
Handles all interactions with Bybit P2P using the Python SDK wrapper
"""

import asyncio
import json
import logging
import sys
from datetime import datetime
from decimal import Decimal
from typing import List, Dict, Any, Optional
from pathlib import Path

# Add python_modules to path
sys.path.insert(0, str(Path(__file__).parent / 'python_modules'))

from bybit_wrapper import BybitP2PWrapper

logger = logging.getLogger(__name__)


class BybitClient:
    """Bybit P2P API client"""
    
    def __init__(self, config):
        self.config = config
        self.client = BybitP2PWrapper(
            api_key=config.bybit_api_key,
            api_secret=config.bybit_api_secret,
            testnet=config.bybit_testnet
        )
        self.account_info = None
        
    async def get_account_info(self) -> Optional[Dict[str, Any]]:
        """Get account information"""
        try:
            loop = asyncio.get_event_loop()
            self.account_info = await loop.run_in_executor(None, self.client.get_account_info)
            return self.account_info
        except Exception as e:
            logger.error(f"Error getting Bybit account info: {e}")
            return None
    
    async def get_market_rates(self, token: str = 'USDT', fiat: str = 'RUB') -> List[Dict[str, Any]]:
        """Get current P2P market rates"""
        try:
            params = {
                'token_id': token,
                'currency_id': fiat,
                'side': 0,  # 0 = Buy (from user perspective, so we're selling)
                'payment': self.config.payment_method_ids,
                'page': 1,
                'size': 10
            }
            
            loop = asyncio.get_event_loop()
            result = await loop.run_in_executor(None, self.client.fetch_p2p_rates, params)
            
            if result.get('success'):
                items = result.get('result', {}).get('items', [])
                return items
            else:
                logger.error(f"Failed to fetch P2P rates: {result.get('error')}")
                return []
                
        except Exception as e:
            logger.error(f"Error fetching market rates: {e}")
            return []
    
    async def create_advertisement(self, params: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Create a P2P advertisement"""
        try:
            loop = asyncio.get_event_loop()
            result = await loop.run_in_executor(None, self.client.create_advertisement, params)
            
            if result:
                logger.info(f"Created advertisement: {result.get('id')}")
                return result
            return None
            
        except Exception as e:
            logger.error(f"Error creating advertisement: {e}")
            return None
    
    async def get_my_advertisements(self) -> List[Dict[str, Any]]:
        """Get user's active advertisements"""
        try:
            loop = asyncio.get_event_loop()
            ads = await loop.run_in_executor(None, self.client.get_my_advertisements)
            return ads or []
        except Exception as e:
            logger.error(f"Error getting advertisements: {e}")
            return []
    
    async def get_active_orders(self) -> List[Dict[str, Any]]:
        """Get all active orders"""
        try:
            loop = asyncio.get_event_loop()
            orders = await loop.run_in_executor(None, self.client.get_active_orders)
            return orders or []
        except Exception as e:
            logger.error(f"Error getting active orders: {e}")
            return []
    
    async def get_ad_orders(self, ad_id: str) -> List[Dict[str, Any]]:
        """Get orders for a specific advertisement"""
        try:
            # Get all active orders and filter by ad ID
            orders = await self.get_active_orders()
            ad_orders = [o for o in orders if o.get('adId') == ad_id]
            return ad_orders
        except Exception as e:
            logger.error(f"Error getting ad orders: {e}")
            return []
    
    async def get_order_details(self, order_id: str) -> Optional[Dict[str, Any]]:
        """Get specific order details"""
        try:
            loop = asyncio.get_event_loop()
            details = await loop.run_in_executor(None, self.client.get_order_details, order_id)
            return details
        except Exception as e:
            logger.error(f"Error getting order details: {e}")
            return None
    
    async def get_chat_messages(self, order_id: str) -> List[Dict[str, Any]]:
        """Get chat messages for an order"""
        try:
            loop = asyncio.get_event_loop()
            messages = await loop.run_in_executor(None, self.client.get_chat_messages, order_id)
            return messages or []
        except Exception as e:
            logger.error(f"Error getting chat messages: {e}")
            return []
    
    async def send_chat_message(self, order_id: str, message: str) -> bool:
        """Send a chat message in an order"""
        try:
            loop = asyncio.get_event_loop()
            success = await loop.run_in_executor(None, self.client.send_chat_message, order_id, message)
            return success
        except Exception as e:
            logger.error(f"Error sending chat message: {e}")
            return False
    
    async def confirm_payment(self, order_id: str) -> bool:
        """Confirm payment received for an order"""
        try:
            loop = asyncio.get_event_loop()
            success = await loop.run_in_executor(None, self.client.confirm_payment, order_id)
            return success
        except Exception as e:
            logger.error(f"Error confirming payment: {e}")
            return False
    
    async def release_order(self, order_id: str) -> bool:
        """Release funds for an order"""
        try:
            loop = asyncio.get_event_loop()
            success = await loop.run_in_executor(None, self.client.release_order, order_id)
            return success
        except Exception as e:
            logger.error(f"Error releasing order: {e}")
            return False
    
    async def cancel_advertisement(self, ad_id: str) -> bool:
        """Cancel an advertisement"""
        try:
            # Note: This might need to be implemented in the wrapper
            logger.warning("Advertisement cancellation not yet implemented")
            return False
        except Exception as e:
            logger.error(f"Error canceling advertisement: {e}")
            return False