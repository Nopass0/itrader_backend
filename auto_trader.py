"""
Auto Trader - Multi-Account P2P Trading System
Main application that orchestrates all components
"""

import asyncio
import logging
import signal
import sys
import os
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
import threading

# Import all components
from db_manager import DatabaseManager
from account_manager import AccountManager
from gmail_client import GmailClient
from gate_multi_client import GateMultiClient
from bybit_multi_client import BybitMultiClient
from chat_flow import ChatFlowManager
from transaction_manager import TransactionManager, TransactionStatus
from websocket_admin import WebSocketAdminServer

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class AutoTrader:
    """Main auto-trader application"""
    
    def __init__(self, config_path: Optional[str] = None):
        """Initialize auto trader with configuration"""
        self.running = False
        self.start_time = None
        
        # Initialize database manager
        self.db_manager = DatabaseManager("db")
        
        # Load settings
        self.settings = self.db_manager.load_settings()
        self.balance_update_interval = self.settings.get("balance_update_interval", 14400)  # 4 hours
        self.gate_relogin_interval = self.settings.get("gate_relogin_interval", 1800)  # 30 minutes
        
        # Initialize managers
        self.account_manager = AccountManager(self.db_manager)
        self.transaction_manager = TransactionManager(self.db_manager)
        self.chat_flow_manager = ChatFlowManager(self.db_manager, self.transaction_manager)
        
        # Initialize multi-clients
        self.gate_multi_client = GateMultiClient(self.account_manager, self.db_manager)
        self.bybit_multi_client = BybitMultiClient(self.account_manager, self.db_manager)
        
        # Initialize Gmail client
        self.gmail_client = GmailClient(self.db_manager)
        
        # Initialize WebSocket admin server
        admin_token = self.settings.get("admin_token")
        self.admin_server = WebSocketAdminServer(admin_token=admin_token)
        self.admin_server.set_app_instance(self)
        
        # Background tasks
        self.background_tasks = []
        
        logger.info("Auto Trader initialized")
        
    async def start(self):
        """Start the auto trader"""
        logger.info("Starting Auto Trader...")
        self.running = True
        self.start_time = datetime.now()
        
        # Authenticate Gmail (skip if disabled)
        self.gmail_enabled = os.getenv("DISABLE_GMAIL", "false").lower() != "true"
        if self.gmail_enabled:
            if not self.gmail_client.authenticate():
                logger.warning("Gmail authentication failed. Email monitoring disabled.")
                self.gmail_enabled = False
            else:
                logger.info("Gmail authenticated successfully")
        else:
            logger.info("Gmail monitoring disabled by environment variable")
            
        # Login all Gate.io accounts
        logger.info("Logging in to Gate.io accounts...")
        gate_login_results = await self.gate_multi_client.login_all()
        logger.info(f"Gate.io login results: {gate_login_results}")
        
        # Test Bybit connections
        logger.info("Testing Bybit connections...")
        bybit_test_results = await self.bybit_multi_client.test_all_connections()
        logger.info(f"Bybit connection results: {bybit_test_results}")
        
        # Start background tasks
        self.background_tasks = [
            asyncio.create_task(self.monitor_transactions()),
            asyncio.create_task(self.monitor_bybit_orders()),
            asyncio.create_task(self.periodic_balance_update()),
            asyncio.create_task(self.periodic_gate_relogin()),
            asyncio.create_task(self.cleanup_old_data()),
        ]
        
        # Add email monitoring if enabled
        if self.gmail_enabled:
            self.background_tasks.append(asyncio.create_task(self.monitor_emails()))
        
        # Start WebSocket admin server in separate thread
        admin_thread = threading.Thread(target=self.admin_server.run, daemon=True)
        admin_thread.start()
        
        logger.info("Auto Trader started successfully")
        
        # Broadcast startup event
        await self.admin_server.broadcast_event("system_started", {
            "start_time": self.start_time.isoformat(),
            "accounts": self.account_manager.get_account_stats()
        })
        
    async def stop(self):
        """Stop the auto trader"""
        logger.info("Stopping Auto Trader...")
        self.running = False
        
        # Cancel background tasks
        for task in self.background_tasks:
            task.cancel()
            
        # Wait for tasks to complete
        await asyncio.gather(*self.background_tasks, return_exceptions=True)
        
        # Cleanup
        self.gate_multi_client.cleanup()
        self.bybit_multi_client.cleanup()
        self.account_manager.cleanup()
        
        logger.info("Auto Trader stopped")
        
    async def monitor_transactions(self):
        """Monitor Gate.io transactions"""
        while self.running:
            try:
                # Get pending transactions from all accounts
                all_transactions = await self.gate_multi_client.get_all_pending_transactions()
                
                for account_id, transactions in all_transactions.items():
                    for gate_tx in transactions:
                        # Check if we already have this transaction
                        tx_id = gate_tx.get("id")
                        existing_tx = await self.transaction_manager.get_transaction(tx_id)
                        
                        if not existing_tx:
                            # New transaction - create and start chat flow
                            transaction = self.transaction_manager.create_transaction(
                                platform="gate",
                                account_id=account_id,
                                buyer_id=gate_tx.get("buyer_id"),
                                amount=float(gate_tx.get("amount", 0)),
                                price=float(gate_tx.get("price", 0))
                            )
                            
                            # Start chat flow
                            message, session = await self.chat_flow_manager.process_new_buyer(
                                transaction.id,
                                transaction.buyer_id,
                                "gate"
                            )
                            
                            # Send initial message
                            client = self.gate_multi_client.get_client(account_id)
                            if client:
                                await asyncio.get_event_loop().run_in_executor(
                                    None,
                                    client.send_chat_message,
                                    transaction.id,
                                    message
                                )
                            
                            # Broadcast new transaction event
                            await self.admin_server.broadcast_event("new_transaction", {
                                "transaction": transaction.to_dict(),
                                "account_id": account_id
                            })
                        else:
                            # Check for new messages
                            client = self.gate_multi_client.get_client(account_id)
                            if client:
                                messages = await asyncio.get_event_loop().run_in_executor(
                                    None,
                                    client.get_chat_messages,
                                    tx_id
                                )
                                
                                # Process new messages
                                session = self.chat_flow_manager.get_session(tx_id)
                                if session:
                                    for msg in messages:
                                        if msg.get("userId") != account_id:  # Buyer message
                                            response, _ = await self.chat_flow_manager.process_buyer_response(
                                                tx_id,
                                                msg.get("content", "")
                                            )
                                            
                                            if response:
                                                # Send response
                                                await asyncio.get_event_loop().run_in_executor(
                                                    None,
                                                    client.send_chat_message,
                                                    tx_id,
                                                    response
                                                )
                
            except Exception as e:
                logger.error(f"Error monitoring transactions: {e}")
                
            # Wait before next check
            await asyncio.sleep(30)
            
    async def monitor_bybit_orders(self):
        """Monitor Bybit P2P orders"""
        async def handle_order(order_data: Dict[str, Any]):
            account_id = order_data["account_id"]
            order = order_data["order"]
            order_id = order.get("orderId")
            
            # Check if we already have this order
            tx_id = f"bybit_{order_id}"
            existing_tx = await self.transaction_manager.get_transaction(tx_id)
            
            if not existing_tx:
                # New order - create transaction
                transaction = self.transaction_manager.create_transaction(
                    platform="bybit",
                    account_id=account_id,
                    buyer_id=order.get("buyerUserId"),
                    amount=float(order.get("amount", 0)),
                    price=float(order.get("price", 0))
                )
                
                # Start chat flow
                message, session = await self.chat_flow_manager.process_new_buyer(
                    transaction.id,
                    transaction.buyer_id,
                    "bybit"
                )
                
                # Send initial message
                await self.bybit_multi_client.send_chat_message(
                    account_id,
                    order_id,
                    message
                )
                
                # Broadcast new order event
                await self.admin_server.broadcast_event("new_bybit_order", {
                    "transaction": transaction.to_dict(),
                    "account_id": account_id,
                    "order": order
                })
        
        # Start monitoring
        await self.bybit_multi_client.monitor_orders(handle_order, interval=30)
        
    async def periodic_balance_update(self):
        """Periodically update Gate.io balances"""
        while self.running:
            try:
                # Update balances
                results = await self.gate_multi_client.update_all_balances()
                
                logger.info(f"Balance update completed: {results}")
                
                # Broadcast update event
                await self.admin_server.broadcast_event("balance_update", {
                    "results": results,
                    "timestamp": datetime.now().isoformat()
                })
                
            except Exception as e:
                logger.error(f"Error updating balances: {e}")
                
            # Wait for next update
            await asyncio.sleep(self.balance_update_interval)
            
    async def periodic_gate_relogin(self):
        """Periodically re-login to Gate.io accounts"""
        while self.running:
            try:
                # Re-login accounts that need it
                results = await self.gate_multi_client.relogin_needed_accounts()
                
                if results:
                    logger.info(f"Re-login completed: {results}")
                    
                    # Broadcast relogin event
                    await self.admin_server.broadcast_event("gate_relogin", {
                        "results": results,
                        "timestamp": datetime.now().isoformat()
                    })
                    
            except Exception as e:
                logger.error(f"Error during re-login: {e}")
                
            # Wait for next check
            await asyncio.sleep(self.gate_relogin_interval)
            
    async def monitor_emails(self):
        """Monitor Gmail for receipts"""
        if not self.gmail_client.is_authenticated():
            logger.warning("Gmail not authenticated, skipping email monitoring")
            return
            
        def handle_receipt(receipt_data: Dict[str, Any]):
            """Handle new receipt"""
            asyncio.create_task(self.process_receipt(receipt_data))
            
        # Start monitoring in thread
        email_thread = threading.Thread(
            target=self.gmail_client.monitor_new_receipts,
            args=(handle_receipt, 60),
            daemon=True
        )
        email_thread.start()
        
        # Keep task alive
        while self.running:
            await asyncio.sleep(60)
            
    async def process_receipt(self, receipt_data: Dict[str, Any]):
        """Process received receipt"""
        try:
            # Extract transaction ID from email or subject
            # This is a simplified example - you'd need to parse the actual format
            subject = receipt_data.get("subject", "")
            
            # Find matching transaction
            # In reality, you'd match by amount, phone, date, etc.
            pending_transactions = self.transaction_manager.get_transactions_by_status(
                TransactionStatus.WAITING_PAYMENT
            )
            
            for transaction in pending_transactions:
                # Try to match transaction
                # This is where you'd use OCR and validation
                
                # Process receipt
                result = await self.transaction_manager.process_receipt(
                    transaction.id,
                    receipt_data["filename"],
                    receipt_data["pdf_data"],
                    {
                        "email_id": receipt_data["message_id"],
                        "from": receipt_data["from"],
                        "subject": receipt_data["subject"],
                        "date": receipt_data["date"].isoformat() if receipt_data.get("date") else None
                    }
                )
                
                if result["success"]:
                    # TODO: Perform OCR validation here
                    # For now, just approve
                    self.transaction_manager.update_transaction_status(
                        transaction.id,
                        TransactionStatus.APPROVED
                    )
                    
                    # Approve on platform
                    if transaction.platform == "gate":
                        client = self.gate_multi_client.get_client(transaction.account_id)
                        if client:
                            await asyncio.get_event_loop().run_in_executor(
                                None,
                                client.approve_transaction,
                                transaction.id
                            )
                    
                    # Broadcast receipt processed event
                    await self.admin_server.broadcast_event("receipt_processed", {
                        "transaction_id": transaction.id,
                        "filename": receipt_data["filename"],
                        "result": "approved"
                    })
                    
                    break
                    
        except Exception as e:
            logger.error(f"Error processing receipt: {e}")
            
    async def cleanup_old_data(self):
        """Periodically clean up old data"""
        while self.running:
            try:
                # Clean up old transactions
                tx_cleaned = self.transaction_manager.cleanup_old_transactions(days=30)
                
                # Clean up old chat sessions
                chat_cleaned = self.chat_flow_manager.cleanup_old_sessions(days=7)
                
                # Clean up database files
                self.db_manager.cleanup_old_data(days=30)
                
                logger.info(f"Cleanup completed: {tx_cleaned} transactions, {chat_cleaned} chat sessions")
                
            except Exception as e:
                logger.error(f"Error during cleanup: {e}")
                
            # Run daily
            await asyncio.sleep(86400)  # 24 hours
            
    async def run(self):
        """Main run loop"""
        await self.start()
        
        try:
            # Keep running until stopped
            while self.running:
                await asyncio.sleep(1)
                
        except KeyboardInterrupt:
            logger.info("Received interrupt signal")
        finally:
            await self.stop()


def main():
    """Main entry point"""
    # Setup signal handlers
    def signal_handler(sig, frame):
        logger.info("Received signal to stop")
        sys.exit(0)
        
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Create and run auto trader
    trader = AutoTrader()
    
    try:
        asyncio.run(trader.run())
    except KeyboardInterrupt:
        logger.info("Shutting down...")
    except Exception as e:
        logger.error(f"Fatal error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()