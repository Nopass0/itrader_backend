"""
WebSocket Admin API for Trading System
Provides real-time control and monitoring
"""

import asyncio
import json
import logging
import secrets
from typing import Dict, List, Optional, Any, Set
from datetime import datetime
import websockets
from websockets.server import WebSocketServerProtocol

logger = logging.getLogger(__name__)


class AdminClient:
    """Connected admin client"""
    
    def __init__(self, websocket: WebSocketServerProtocol, client_id: str):
        self.websocket = websocket
        self.client_id = client_id
        self.authenticated = False
        self.connected_at = datetime.now()
        self.last_activity = datetime.now()
        
    async def send_message(self, message: Dict[str, Any]):
        """Send message to client"""
        try:
            await self.websocket.send(json.dumps(message))
            self.last_activity = datetime.now()
        except Exception as e:
            logger.error(f"Failed to send message to client {self.client_id}: {e}")


class WebSocketAdminServer:
    """WebSocket server for admin control"""
    
    def __init__(self, host: str = "localhost", port: int = 8765, admin_token: Optional[str] = None):
        self.host = host
        self.port = port
        self.admin_token = admin_token or secrets.token_urlsafe(32)
        self.clients: Dict[str, AdminClient] = {}
        self.handlers: Dict[str, Any] = {}
        self.app_instance = None  # Will be set by main app
        
        logger.info(f"Admin token: {self.admin_token}")
        
    def set_app_instance(self, app):
        """Set the main application instance"""
        self.app_instance = app
        
    def register_handler(self, command: str, handler):
        """Register command handler"""
        self.handlers[command] = handler
        
    async def handle_client(self, websocket: WebSocketServerProtocol, path: str):
        """Handle new client connection"""
        client_id = secrets.token_urlsafe(16)
        client = AdminClient(websocket, client_id)
        self.clients[client_id] = client
        
        logger.info(f"New admin client connected: {client_id}")
        
        try:
            # Send welcome message
            await client.send_message({
                "type": "welcome",
                "message": "Connected to Trading System Admin API. Please authenticate.",
                "client_id": client_id,
                "timestamp": datetime.now().isoformat()
            })
            
            # Handle messages
            async for message in websocket:
                await self.handle_message(client, message)
                
        except websockets.exceptions.ConnectionClosed:
            logger.info(f"Client disconnected: {client_id}")
        except Exception as e:
            logger.error(f"Error handling client {client_id}: {e}")
        finally:
            # Clean up
            if client_id in self.clients:
                del self.clients[client_id]
                
    async def handle_message(self, client: AdminClient, message: str):
        """Handle incoming message from client"""
        try:
            data = json.loads(message)
            command = data.get("command")
            
            # Check authentication
            if command == "auth":
                await self.handle_auth(client, data)
                return
                
            # All other commands require authentication
            if not client.authenticated:
                await client.send_message({
                    "type": "error",
                    "error": "Not authenticated",
                    "message": "Please authenticate first"
                })
                return
                
            # Handle command
            if command in self.handlers:
                handler = self.handlers[command]
                response = await handler(data)
                await client.send_message(response)
            else:
                # Built-in commands
                await self.handle_builtin_command(client, command, data)
                
        except json.JSONDecodeError:
            await client.send_message({
                "type": "error",
                "error": "Invalid JSON",
                "message": "Failed to parse message"
            })
        except Exception as e:
            logger.error(f"Error handling message: {e}")
            await client.send_message({
                "type": "error",
                "error": "Internal error",
                "message": str(e)
            })
            
    async def handle_auth(self, client: AdminClient, data: Dict[str, Any]):
        """Handle authentication"""
        token = data.get("token")
        
        if token == self.admin_token:
            client.authenticated = True
            await client.send_message({
                "type": "auth_success",
                "message": "Authentication successful",
                "timestamp": datetime.now().isoformat()
            })
            logger.info(f"Client authenticated: {client.client_id}")
        else:
            await client.send_message({
                "type": "auth_failed",
                "message": "Invalid token"
            })
            logger.warning(f"Authentication failed for client: {client.client_id}")
            
    async def handle_builtin_command(self, client: AdminClient, command: str, data: Dict[str, Any]):
        """Handle built-in commands"""
        
        if command == "status":
            await self.handle_status(client)
            
        elif command == "add_account":
            await self.handle_add_account(client, data)
            
        elif command == "remove_account":
            await self.handle_remove_account(client, data)
            
        elif command == "list_accounts":
            await self.handle_list_accounts(client, data)
            
        elif command == "list_transactions":
            await self.handle_list_transactions(client, data)
            
        elif command == "get_transaction":
            await self.handle_get_transaction(client, data)
            
        elif command == "approve_transaction":
            await self.handle_approve_transaction(client, data)
            
        elif command == "reject_transaction":
            await self.handle_reject_transaction(client, data)
            
        elif command == "force_balance_update":
            await self.handle_force_balance_update(client, data)
            
        elif command == "force_relogin":
            await self.handle_force_relogin(client, data)
            
        elif command == "get_statistics":
            await self.handle_get_statistics(client)
            
        elif command == "set_settings":
            await self.handle_set_settings(client, data)
            
        elif command == "get_settings":
            await self.handle_get_settings(client)
            
        else:
            await client.send_message({
                "type": "error",
                "error": "Unknown command",
                "message": f"Command '{command}' not recognized"
            })
            
    async def handle_status(self, client: AdminClient):
        """Get system status"""
        if not self.app_instance:
            await client.send_message({
                "type": "error",
                "error": "App not initialized"
            })
            return
            
        status = {
            "type": "status",
            "running": self.app_instance.running,
            "start_time": self.app_instance.start_time.isoformat() if hasattr(self.app_instance, 'start_time') else None,
            "accounts": self.app_instance.account_manager.get_account_stats(),
            "transactions": self.app_instance.transaction_manager.get_statistics(),
            "chat_sessions": self.app_instance.chat_flow_manager.get_session_stats(),
            "connected_admins": len(self.clients),
            "timestamp": datetime.now().isoformat()
        }
        
        await client.send_message(status)
        
    async def handle_add_account(self, client: AdminClient, data: Dict[str, Any]):
        """Add new account"""
        account_type = data.get("type")  # gate or bybit
        
        try:
            if account_type == "gate":
                login = data.get("login")
                password = data.get("password")
                
                if not login or not password:
                    raise ValueError("Login and password required for Gate.io account")
                    
                account_id = self.app_instance.account_manager.add_gate_account(login, password)
                
                # Try to login
                success = await self.app_instance.gate_multi_client._login_account(account_id)
                
                await client.send_message({
                    "type": "account_added",
                    "account_type": "gate",
                    "account_id": account_id,
                    "login_success": success,
                    "timestamp": datetime.now().isoformat()
                })
                
            elif account_type == "bybit":
                api_key = data.get("api_key")
                api_secret = data.get("api_secret")
                
                if not api_key or not api_secret:
                    raise ValueError("API key and secret required for Bybit account")
                    
                account_id = self.app_instance.account_manager.add_bybit_account(api_key, api_secret)
                
                # Test connection
                client_obj = self.app_instance.bybit_multi_client.add_client(account_id)
                success = client_obj is not None
                
                await client.send_message({
                    "type": "account_added",
                    "account_type": "bybit",
                    "account_id": account_id,
                    "connection_success": success,
                    "timestamp": datetime.now().isoformat()
                })
                
            else:
                raise ValueError(f"Invalid account type: {account_type}")
                
        except Exception as e:
            await client.send_message({
                "type": "error",
                "error": "Failed to add account",
                "message": str(e)
            })
            
    async def handle_remove_account(self, client: AdminClient, data: Dict[str, Any]):
        """Remove account"""
        account_id = data.get("account_id")
        account_type = data.get("type")  # gate or bybit
        
        try:
            if account_type == "gate":
                success = self.app_instance.account_manager.remove_gate_account(account_id)
                if success:
                    self.app_instance.gate_multi_client.remove_client(account_id)
                    
            elif account_type == "bybit":
                success = self.app_instance.account_manager.remove_bybit_account(account_id)
                if success:
                    self.app_instance.bybit_multi_client.remove_client(account_id)
                    
            else:
                raise ValueError(f"Invalid account type: {account_type}")
                
            await client.send_message({
                "type": "account_removed",
                "account_id": account_id,
                "success": success,
                "timestamp": datetime.now().isoformat()
            })
            
        except Exception as e:
            await client.send_message({
                "type": "error",
                "error": "Failed to remove account",
                "message": str(e)
            })
            
    async def handle_list_accounts(self, client: AdminClient, data: Dict[str, Any]):
        """List accounts"""
        account_type = data.get("type")  # gate, bybit, or all
        
        accounts = {
            "type": "accounts_list",
            "accounts": {}
        }
        
        if account_type in ["gate", "all", None]:
            accounts["accounts"]["gate"] = self.app_instance.account_manager.list_gate_accounts()
            
        if account_type in ["bybit", "all", None]:
            accounts["accounts"]["bybit"] = self.app_instance.account_manager.list_bybit_accounts()
            
        accounts["timestamp"] = datetime.now().isoformat()
        
        await client.send_message(accounts)
        
    async def handle_list_transactions(self, client: AdminClient, data: Dict[str, Any]):
        """List transactions"""
        status = data.get("status")  # pending, approved, etc.
        limit = data.get("limit", 100)
        
        if status:
            from transaction_manager import TransactionStatus
            try:
                status_enum = TransactionStatus(status)
                transactions = self.app_instance.transaction_manager.get_transactions_by_status(status_enum)
            except ValueError:
                transactions = []
        else:
            transactions = list(self.app_instance.transaction_manager.transactions.values())
            
        # Convert to dict and limit
        transactions_data = [tx.to_dict() for tx in transactions[:limit]]
        
        await client.send_message({
            "type": "transactions_list",
            "count": len(transactions_data),
            "transactions": transactions_data,
            "timestamp": datetime.now().isoformat()
        })
        
    async def handle_get_transaction(self, client: AdminClient, data: Dict[str, Any]):
        """Get transaction details"""
        transaction_id = data.get("transaction_id")
        
        transaction = await self.app_instance.transaction_manager.get_transaction(transaction_id)
        
        if transaction:
            await client.send_message({
                "type": "transaction_details",
                "transaction": transaction.to_dict(),
                "timestamp": datetime.now().isoformat()
            })
        else:
            await client.send_message({
                "type": "error",
                "error": "Transaction not found",
                "transaction_id": transaction_id
            })
            
    async def handle_approve_transaction(self, client: AdminClient, data: Dict[str, Any]):
        """Manually approve transaction"""
        transaction_id = data.get("transaction_id")
        
        try:
            from transaction_manager import TransactionStatus
            success = self.app_instance.transaction_manager.update_transaction_status(
                transaction_id,
                TransactionStatus.APPROVED
            )
            
            await client.send_message({
                "type": "transaction_approved",
                "transaction_id": transaction_id,
                "success": success,
                "timestamp": datetime.now().isoformat()
            })
            
        except Exception as e:
            await client.send_message({
                "type": "error",
                "error": "Failed to approve transaction",
                "message": str(e)
            })
            
    async def handle_reject_transaction(self, client: AdminClient, data: Dict[str, Any]):
        """Manually reject transaction"""
        transaction_id = data.get("transaction_id")
        reason = data.get("reason", "Manual rejection")
        
        try:
            from transaction_manager import TransactionStatus
            
            # Update transaction
            transaction = await self.app_instance.transaction_manager.get_transaction(transaction_id)
            if transaction:
                transaction.status = TransactionStatus.REJECTED
                transaction.metadata["rejection_reason"] = reason
                transaction.updated_at = datetime.now()
                
                self.app_instance.db_manager.save_transaction(transaction_id, transaction.to_dict())
                
                await client.send_message({
                    "type": "transaction_rejected",
                    "transaction_id": transaction_id,
                    "success": True,
                    "timestamp": datetime.now().isoformat()
                })
            else:
                raise ValueError("Transaction not found")
                
        except Exception as e:
            await client.send_message({
                "type": "error",
                "error": "Failed to reject transaction",
                "message": str(e)
            })
            
    async def handle_force_balance_update(self, client: AdminClient, data: Dict[str, Any]):
        """Force balance update for all accounts"""
        try:
            results = await self.app_instance.gate_multi_client.update_all_balances()
            
            await client.send_message({
                "type": "balance_update_complete",
                "results": results,
                "timestamp": datetime.now().isoformat()
            })
            
        except Exception as e:
            await client.send_message({
                "type": "error",
                "error": "Failed to update balances",
                "message": str(e)
            })
            
    async def handle_force_relogin(self, client: AdminClient, data: Dict[str, Any]):
        """Force re-login for accounts"""
        account_id = data.get("account_id")  # Optional, if not provided, relogin all
        
        try:
            if account_id:
                success = await self.app_instance.gate_multi_client._login_account(account_id)
                results = {account_id: success}
            else:
                results = await self.app_instance.gate_multi_client.login_all()
                
            await client.send_message({
                "type": "relogin_complete",
                "results": results,
                "timestamp": datetime.now().isoformat()
            })
            
        except Exception as e:
            await client.send_message({
                "type": "error",
                "error": "Failed to relogin",
                "message": str(e)
            })
            
    async def handle_get_statistics(self, client: AdminClient):
        """Get system statistics"""
        stats = {
            "type": "statistics",
            "transactions": self.app_instance.transaction_manager.get_statistics(),
            "accounts": self.app_instance.account_manager.get_account_stats(),
            "chat_sessions": self.app_instance.chat_flow_manager.get_session_stats(),
            "bybit_stats": self.app_instance.bybit_multi_client.get_account_stats(),
            "timestamp": datetime.now().isoformat()
        }
        
        await client.send_message(stats)
        
    async def handle_set_settings(self, client: AdminClient, data: Dict[str, Any]):
        """Update system settings"""
        settings = data.get("settings", {})
        
        try:
            current_settings = self.app_instance.db_manager.load_settings()
            current_settings.update(settings)
            
            success = self.app_instance.db_manager.save_settings(current_settings)
            
            # Apply settings if needed
            if "balance_update_interval" in settings:
                self.app_instance.balance_update_interval = settings["balance_update_interval"]
            if "gate_relogin_interval" in settings:
                self.app_instance.gate_relogin_interval = settings["gate_relogin_interval"]
                
            await client.send_message({
                "type": "settings_updated",
                "success": success,
                "settings": current_settings,
                "timestamp": datetime.now().isoformat()
            })
            
        except Exception as e:
            await client.send_message({
                "type": "error",
                "error": "Failed to update settings",
                "message": str(e)
            })
            
    async def handle_get_settings(self, client: AdminClient):
        """Get current settings"""
        settings = self.app_instance.db_manager.load_settings()
        
        await client.send_message({
            "type": "current_settings",
            "settings": settings,
            "timestamp": datetime.now().isoformat()
        })
        
    async def broadcast_event(self, event_type: str, data: Dict[str, Any]):
        """Broadcast event to all authenticated clients"""
        message = {
            "type": "event",
            "event_type": event_type,
            "data": data,
            "timestamp": datetime.now().isoformat()
        }
        
        # Send to all authenticated clients
        tasks = []
        for client in self.clients.values():
            if client.authenticated:
                tasks.append(client.send_message(message))
                
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)
            
    async def start_server(self):
        """Start WebSocket server"""
        logger.info(f"Starting WebSocket admin server on {self.host}:{self.port}")
        
        async with websockets.serve(self.handle_client, self.host, self.port):
            logger.info(f"WebSocket admin server running on ws://{self.host}:{self.port}")
            logger.info(f"Admin token: {self.admin_token}")
            
            # Keep server running
            await asyncio.Future()  # Run forever
            
    def run(self):
        """Run server in event loop"""
        asyncio.run(self.start_server())