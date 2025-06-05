"""
Bybit SDK wrapper for P2P operations
This module provides a Python interface for Bybit P2P operations
that can be called from Rust via PyO3
"""

import asyncio
import json
from typing import Dict, List, Optional, Any
from datetime import datetime
from pybit.unified_trading import HTTP
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class BybitP2PWrapper:
    """Wrapper for Bybit P2P operations using the official SDK"""
    
    def __init__(self, api_key: str, api_secret: str, testnet: bool = False):
        """
        Initialize Bybit P2P client
        
        Args:
            api_key: Bybit API key
            api_secret: Bybit API secret
            testnet: Whether to use testnet (default: False)
        """
        self.api_key = api_key
        self.api_secret = api_secret
        self.testnet = testnet
        self.base_url = "https://api-testnet.bybit.com" if testnet else "https://api.bybit.com"
        
        # Initialize the HTTP client with extended recv_window to handle time sync issues
        self.session = HTTP(
            testnet=testnet,
            api_key=api_key,
            api_secret=api_secret,
            recv_window=5000000,  # Increase recv_window to 5000 seconds (83 minutes) to handle large time differences
        )
        
    def get_server_time(self) -> int:
        """Get server time in milliseconds"""
        try:
            response = self.session.get_server_time()
            if response['retCode'] == 0:
                return int(response['result']['timeSecond']) * 1000
            else:
                raise Exception(f"Failed to get server time: {response['retMsg']}")
        except Exception as e:
            logger.error(f"Error getting server time: {e}")
            raise
    
    def get_account_info(self) -> Dict[str, Any]:
        """Get P2P account information"""
        try:
            # Get API info to retrieve UID
            api_info = self.session.get_api_key_information()
            
            if api_info.get('retCode') == 0:
                # Log full API response for debugging
                logger.info(f"Full API info response: {api_info}")
                
                result = api_info.get('result', {})
                uid = result.get('userID')
                
                # Try to get user info from SDK
                try:
                    # Check if account has email in other endpoints
                    if hasattr(self.session, 'get_account_info'):
                        sdk_account_info = self.session.get_account_info()
                        logger.info(f"SDK account info: {sdk_account_info}")
                except Exception as e:
                    logger.debug(f"SDK account info not available: {e}")
                
                # Try P2P endpoints to get email
                email_found = None
                nickname_found = None
                try:
                    import requests
                    
                    # Try P2P personal info endpoint (POST)
                    # For empty body, don't send json={}, send data=""
                    headers = self._get_auth_headers("/v5/p2p/user/personal/info", {}, "POST", None)
                    response = requests.post(
                        f"{self.base_url}/v5/p2p/user/personal/info",
                        headers=headers,
                        data=""  # Empty string for empty body
                    )
                    logger.info(f"P2P personal info status: {response.status_code}")
                    if response.status_code == 200:
                        p2p_user_data = response.json()
                        logger.info(f"P2P personal info response: {p2p_user_data}")
                        if p2p_user_data.get('ret_code') == 0:
                            p2p_result = p2p_user_data.get('result', {})
                            email_found = p2p_result.get('email')
                            nickname_found = p2p_result.get('nickName') or p2p_result.get('nickname')
                            # Also get userId and realName
                            p2p_user_id = p2p_result.get('userId')
                            real_name = p2p_result.get('realName')
                            logger.info(f"P2P User ID: {p2p_user_id}, Nickname: {nickname_found}, Email: {email_found}, Real Name: {real_name}")
                    else:
                        logger.info(f"P2P personal info error: {response.text}")
                    
                    
                except Exception as e:
                    logger.warning(f"Could not get P2P info: {e}")
                
                # Return account info with all available fields
                final_email = email_found or result.get('email', '')
                final_nickname = nickname_found or f"user_{uid}"
                
                return {
                    "id": str(uid),
                    "uid": str(uid),  # Also return as uid
                    "email": final_email if final_email else None,  # Return None for Option<String>
                    "nickname": final_nickname,
                    "status": "active",
                    "activeAds": 0
                }
            else:
                raise Exception(f"Failed to get API info: {api_info.get('retMsg', 'Unknown error')}")
        except Exception as e:
            logger.error(f"Error getting account info: {e}")
            raise
    
    def _get_auth_headers(self, endpoint: str, params: Dict[str, Any], method: str = "GET", body: Dict[str, Any] = None) -> Dict[str, str]:
        """Generate authentication headers for direct API calls"""
        import time
        import hmac
        import hashlib
        import json
        
        timestamp = str(int(time.time() * 1000))
        recv_window = "5000"
        
        # For GET requests, use query string
        if method == "GET":
            param_str = ""
            if params:
                param_str = "&".join([f"{k}={v}" for k, v in sorted(params.items())])
            pre_sign = f"{timestamp}{self.api_key}{recv_window}{param_str}"
        else:
            # For POST requests, use JSON body
            body_str = ""
            if body is not None:
                body_str = json.dumps(body, separators=(',', ':'))
            # For empty body, use empty string (not "{}")
            pre_sign = f"{timestamp}{self.api_key}{recv_window}{body_str}"
        
        signature = hmac.new(
            self.api_secret.encode('utf-8'),
            pre_sign.encode('utf-8'),
            hashlib.sha256
        ).hexdigest()
        
        # Log signature details for debugging
        logger.debug(f"Pre-sign string: {pre_sign}")
        logger.debug(f"Generated signature: {signature}")
        
        return {
            "X-BAPI-API-KEY": self.api_key,
            "X-BAPI-TIMESTAMP": timestamp,
            "X-BAPI-RECV-WINDOW": recv_window,
            "X-BAPI-SIGN": signature,
            "Content-Type": "application/json"
        }
    
    def create_advertisement(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a new P2P advertisement
        
        Args:
            params: Advertisement parameters
                - asset: Cryptocurrency (e.g., "USDT")
                - fiat: Fiat currency (e.g., "RUB")
                - price: Price per unit
                - amount: Total amount
                - min_amount: Minimum order amount
                - max_amount: Maximum order amount
                - payment_methods: List of payment method IDs
                - remarks: Optional remarks
        """
        try:
            # Use direct API call to create P2P advertisement
            import requests
            import json
            
            # Prepare request body
            body = {
                "tokenId": params.get("asset", "USDT"),
                "currencyId": params.get("fiat", "RUB"),
                "side": "1",  # 1 = Sell
                "priceType": "1",  # 1 = Fixed price
                "price": str(params.get("price", "100")),
                "quantity": str(params.get("amount", "1000")),
                "minAmount": str(params.get("min_amount", "100")),
                "maxAmount": str(params.get("max_amount", "10000")),
                "payments": params.get("payment_methods", ["1"]),  # Payment method IDs
                "remarks": params.get("remarks", ""),
                "isOnline": "1"  # 1 = Online
            }
            
            headers = self._get_auth_headers("/v5/p2p/advertiser/create-ad", {}, "POST", body)
            
            response = requests.post(
                f"{self.base_url}/v5/p2p/advertiser/create-ad",
                headers=headers,
                json=body
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('retCode') == 0:
                    result = data.get('result', {})
                    # Transform to our expected format
                    return {
                        "id": result.get("adId", ""),
                        "asset": params.get("asset", "USDT"),
                        "fiat": params.get("fiat", "RUB"),
                        "price": str(params.get("price", "100")),
                        "amount": str(params.get("amount", "1000")),
                        "min_amount": str(params.get("min_amount", "100")),
                        "max_amount": str(params.get("max_amount", "10000")),
                        "status": "1",
                        "payment_methods": [{"id": str(pm), "name": "Payment"} for pm in params.get("payment_methods", ["1"])],
                        "remarks": params.get("remarks", ""),
                        "created_at": str(result.get("createdAt", ""))
                    }
                else:
                    raise Exception(f"Failed to create ad: {data.get('retMsg')}")
            else:
                raise Exception(f"Failed to create ad: HTTP {response.status_code}")
        except Exception as e:
            logger.error(f"Error creating advertisement: {e}")
            raise
    
    def get_my_advertisements(self) -> List[Dict[str, Any]]:
        """Get user's P2P advertisements"""
        try:
            # Use direct API call for P2P ads
            import requests
            
            # According to docs, this is a POST endpoint
            body = {
                "status": 1,  # 1 = Active ads
                "limit": 50
            }
            
            headers = self._get_auth_headers("/v5/p2p/advertise/list", {}, "POST", body)
            
            response = requests.post(
                f"{self.base_url}/v5/p2p/advertise/list",
                headers=headers,
                json=body
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('retCode') == 0:
                    return data.get('result', {}).get('list', [])
                else:
                    logger.warning(f"P2P API returned error: {data.get('retMsg')}")
                    return []
            else:
                logger.warning(f"Failed to get advertisements: HTTP {response.status_code}")
                return []
        except Exception as e:
            logger.error(f"Error getting advertisements: {e}")
            return []  # Return empty list on error
    
    def get_active_orders(self) -> List[Dict[str, Any]]:
        """Get active P2P orders"""
        try:
            # Use direct API call for P2P orders
            import requests
            
            params = {
                "orderStatus": "10,20,30",  # Active statuses
                "limit": "50"
            }
            
            headers = self._get_auth_headers("/v5/p2p/order/list", params, "GET")
            
            response = requests.get(
                f"{self.base_url}/v5/p2p/order/list",
                headers=headers,
                params=params
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('retCode') == 0:
                    return data.get('result', {}).get('list', [])
                else:
                    logger.warning(f"P2P orders API returned error: {data.get('retMsg')}")
                    return []
            else:
                logger.warning(f"Failed to get orders: HTTP {response.status_code}")
                return []
        except Exception as e:
            logger.error(f"Error getting orders: {e}")
            return []  # Return empty list on error
    
    def get_order_details(self, order_id: str) -> Dict[str, Any]:
        """Get specific order details"""
        try:
            # Direct API call to get P2P order details
            import requests
            
            params = {
                "orderId": order_id
            }
            
            headers = self._get_auth_headers("/v5/p2p/order/detail", params, "GET")
            
            response = requests.get(
                f"{self.base_url}/v5/p2p/order/detail",
                headers=headers,
                params=params
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('retCode') == 0:
                    order = data.get('result', {})
                    return {
                        "id": order.get('orderId', order_id),
                        "status": order.get('orderStatus', '10'),
                        "amount": order.get('amount', '0'),
                        "price": order.get('price', '0'),
                        "tokenId": order.get('tokenId', 'USDT'),
                        "currencyId": order.get('currencyId', 'RUB'),
                        "side": order.get('side', '1'),
                        "createdAt": order.get('createdAt', ''),
                        "paymentInfo": order.get('paymentInfo', {}),
                        "buyerUserId": order.get('buyerUserId', ''),
                        "sellerUserId": order.get('sellerUserId', '')
                    }
                else:
                    logger.warning(f"P2P order detail API returned error: {data.get('retMsg')}")
                    return {"id": order_id, "error": data.get('retMsg', 'Failed to get order details')}
            else:
                logger.warning(f"Failed to get order details: HTTP {response.status_code}")
                return {"id": order_id, "error": f"HTTP {response.status_code}"}
        except Exception as e:
            logger.error(f"Error getting order details: {e}")
            return {"id": order_id, "error": str(e)}
    
    def confirm_payment(self, order_id: str) -> bool:
        """Confirm payment for an order"""
        try:
            # Direct API call to confirm P2P payment
            import requests
            import json
            
            body = {
                "orderId": order_id
            }
            
            headers = self._get_auth_headers("/v5/p2p/order/confirm-payment", {}, "POST", body)
            
            response = requests.post(
                f"{self.base_url}/v5/p2p/order/confirm-payment",
                headers=headers,
                json=body
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('retCode') == 0:
                    logger.info(f"Successfully confirmed payment for order: {order_id}")
                    return True
                else:
                    logger.warning(f"Failed to confirm payment: {data.get('retMsg')}")
                    return False
            else:
                logger.warning(f"Failed to confirm payment: HTTP {response.status_code}")
                return False
        except Exception as e:
            logger.error(f"Error confirming payment: {e}")
            return False
    
    def release_order(self, order_id: str) -> bool:
        """Release funds for an order"""
        try:
            # Direct API call to release P2P order
            import requests
            import json
            
            body = {
                "orderId": order_id
            }
            
            headers = self._get_auth_headers("/v5/p2p/order/release", {}, "POST", body)
            
            response = requests.post(
                f"{self.base_url}/v5/p2p/order/release",
                headers=headers,
                json=body
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('retCode') == 0:
                    logger.info(f"Successfully released order: {order_id}")
                    return True
                else:
                    logger.warning(f"Failed to release order: {data.get('retMsg')}")
                    return False
            else:
                logger.warning(f"Failed to release order: HTTP {response.status_code}")
                return False
        except Exception as e:
            logger.error(f"Error releasing order: {e}")
            return False
    
    def get_chat_messages(self, order_id: str) -> List[Dict[str, Any]]:
        """Get chat messages for an order"""
        try:
            # Direct API call to get P2P chat messages
            import requests
            
            params = {
                "orderId": order_id,
                "limit": "50"
            }
            
            headers = self._get_auth_headers("/v5/p2p/order/chat/messages", params, "GET")
            
            response = requests.get(
                f"{self.base_url}/v5/p2p/order/chat/messages",
                headers=headers,
                params=params
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('retCode') == 0:
                    messages = data.get('result', {}).get('list', [])
                    # Transform messages to our format
                    return [{
                        "id": msg.get('messageId', ''),
                        "orderId": order_id,
                        "userId": msg.get('userId', ''),
                        "content": msg.get('content', ''),
                        "timestamp": msg.get('createdAt', ''),
                        "type": msg.get('messageType', 'text')
                    } for msg in messages]
                else:
                    logger.warning(f"Failed to get chat messages: {data.get('retMsg')}")
                    return []
            else:
                logger.warning(f"Failed to get chat messages: HTTP {response.status_code}")
                return []
        except Exception as e:
            logger.error(f"Error getting chat messages: {e}")
            return []
    
    def send_chat_message(self, order_id: str, message: str) -> bool:
        """Send a chat message in an order"""
        try:
            # Direct API call to send P2P chat message
            import requests
            import json
            
            body = {
                "orderId": order_id,
                "content": message,
                "messageType": "text"
            }
            
            headers = self._get_auth_headers("/v5/p2p/order/chat/send", {}, "POST", body)
            
            response = requests.post(
                f"{self.base_url}/v5/p2p/order/chat/send",
                headers=headers,
                json=body
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('retCode') == 0:
                    logger.info(f"Successfully sent message to order {order_id}")
                    return True
                else:
                    logger.warning(f"Failed to send chat message: {data.get('retMsg')}")
                    return False
            else:
                logger.warning(f"Failed to send chat message: HTTP {response.status_code}")
                return False
        except Exception as e:
            logger.error(f"Error sending chat message: {e}")
            return False
    
    def fetch_p2p_rates(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """
        Fetch P2P trading rates/advertisements
        
        Args:
            params: Request parameters
                - token_id: Cryptocurrency (e.g., "USDT")
                - currency_id: Fiat currency (e.g., "RUB")
                - side: 0=Buy, 1=Sell (from user perspective)
                - payment: List of payment method IDs
                - page: Page number (default: 1)
                - size: Page size (default: 10)
                - amount: Optional filter by amount
        
        Returns:
            Dictionary with P2P listings
        """
        try:
            import requests
            
            # This is a public endpoint - no auth required
            url = "https://api2.bybit.com/fiat/otc/item/online"
            
            # Prepare request body
            body = {
                "userId": params.get("user_id", 431812707),  # Can be any user ID for public data
                "tokenId": params.get("token_id", "USDT"),
                "currencyId": params.get("currency_id", "RUB"),
                "payment": params.get("payment", ["382", "75"]),  # СБП and Тинькофф
                "side": str(params.get("side", 0)),  # 0 = Buy
                "size": str(params.get("size", 10)),
                "page": str(params.get("page", 1)),
                "amount": params.get("amount", ""),
                "vaMaker": False,
                "bulkMaker": False,
                "canTrade": False,
                "verificationFilter": 0,
                "sortType": "TRADE_PRICE",
                "paymentPeriod": [],
                "itemRegion": 1
            }
            
            # Headers to mimic browser request
            headers = {
                "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
                "Accept": "application/json",
                "Accept-Language": "en-US,en;q=0.9",
                "Content-Type": "application/json;charset=UTF-8",
                "Origin": "https://www.bybit.com",
                "Referer": "https://www.bybit.com/"
            }
            
            response = requests.post(url, json=body, headers=headers, timeout=30)
            
            if response.status_code == 200:
                data = response.json()
                if data.get('ret_code') == 0:
                    return {
                        "success": True,
                        "result": data.get('result', {}),
                        "ret_code": data.get('ret_code'),
                        "ret_msg": data.get('ret_msg', 'Success')
                    }
                else:
                    return {
                        "success": False,
                        "error": data.get('ret_msg', 'Failed to fetch P2P rates'),
                        "ret_code": data.get('ret_code')
                    }
            else:
                return {
                    "success": False,
                    "error": f"HTTP {response.status_code}: {response.text}",
                    "ret_code": response.status_code
                }
        except Exception as e:
            logger.error(f"Error fetching P2P rates: {e}")
            return {
                "success": False,
                "error": str(e),
                "ret_code": -1
            }


# Async wrapper functions for Rust integration
async def create_client(api_key: str, api_secret: str, testnet: bool = False) -> BybitP2PWrapper:
    """Create a new Bybit P2P client"""
    return BybitP2PWrapper(api_key, api_secret, testnet)


async def get_server_time(client: BybitP2PWrapper) -> int:
    """Get server time asynchronously"""
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, client.get_server_time)


async def get_account_info(client: BybitP2PWrapper) -> str:
    """Get account info as JSON string"""
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(None, client.get_account_info)
    return json.dumps(result)


async def get_my_advertisements(client: BybitP2PWrapper) -> str:
    """Get advertisements as JSON string"""
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(None, client.get_my_advertisements)
    return json.dumps(result)


async def get_active_orders(client: BybitP2PWrapper) -> str:
    """Get active orders as JSON string"""
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(None, client.get_active_orders)
    return json.dumps(result)


async def create_advertisement(client: BybitP2PWrapper, params: str) -> str:
    """Create advertisement from JSON params"""
    loop = asyncio.get_event_loop()
    params_dict = json.loads(params)
    result = await loop.run_in_executor(None, client.create_advertisement, params_dict)
    return json.dumps(result)


async def fetch_p2p_rates(client: BybitP2PWrapper, params: str) -> str:
    """Fetch P2P rates from JSON params"""
    loop = asyncio.get_event_loop()
    params_dict = json.loads(params) if params else {}
    result = await loop.run_in_executor(None, client.fetch_p2p_rates, params_dict)
    return json.dumps(result)


# For testing
if __name__ == "__main__":
    import os
    from dotenv import load_dotenv
    
    load_dotenv()
    
    # Test the wrapper
    api_key = os.getenv("BYBIT_API_KEY", "test_key")
    api_secret = os.getenv("BYBIT_API_SECRET", "test_secret")
    
    client = BybitP2PWrapper(api_key, api_secret, testnet=True)
    
    try:
        # Test server time
        server_time = client.get_server_time()
        print(f"Server time: {server_time}")
        
        # Test account info
        account = client.get_account_info()
        print(f"Account info: {account}")
        
        # Test P2P rate fetching
        rate_params = {
            "token_id": "USDT",
            "currency_id": "RUB",
            "side": 0,  # Buy
            "payment": ["382", "75"],  # СБП and Тинькофф
            "page": 1,
            "size": 10
        }
        rates = client.fetch_p2p_rates(rate_params)
        print(f"\nP2P Rates: {json.dumps(rates, indent=2)}")
    except Exception as e:
        print(f"Error: {e}")