#!/usr/bin/env python3
"""
Main Auto-Trader System Application
Monitors Gate.io for pending transactions and creates Bybit P2P advertisements
"""

import asyncio
import signal
import sys
import logging
from datetime import datetime, timezone
from decimal import Decimal
from typing import Optional, Dict, Any, List
from colorama import init, Fore, Style
import click

from config import Config
from gate_client import GateClient
from bybit_client import BybitClient
from chat_manager import ChatManager
from email_monitor import EmailMonitor
from models import Transaction, Advertisement, Order, ReceiptValidation
from utils import setup_logging, colored_print, confirm_action

# Initialize colorama for cross-platform colored output
init(autoreset=True)

# Setup logging
logger = setup_logging()


class TraderSystem:
    """Main auto-trader system orchestrator"""
    
    def __init__(self, config: Config):
        self.config = config
        self.gate_client = GateClient(config)
        self.bybit_client = BybitClient(config)
        self.chat_manager = ChatManager(self.bybit_client)
        self.email_monitor = EmailMonitor(config)
        self.running = False
        self.mode = "manual"  # manual or automatic
        self.active_ads: Dict[str, Advertisement] = {}
        self.pending_transactions: List[Transaction] = []
        
    async def start(self, mode: str = "manual"):
        """Start the trader system"""
        self.mode = mode
        self.running = True
        
        colored_print(f"🚀 Starting Trader System in {mode.upper()} mode", Fore.GREEN)
        
        try:
            # Initialize components
            await self.initialize()
            
            # Start monitoring tasks
            tasks = [
                self.monitor_gate_transactions(),
                self.monitor_email_receipts(),
                self.monitor_active_orders(),
            ]
            
            if mode == "automatic":
                tasks.append(self.auto_process_transactions())
            
            # Run all tasks concurrently
            await asyncio.gather(*tasks)
            
        except Exception as e:
            logger.error(f"System error: {e}")
            colored_print(f"❌ System error: {e}", Fore.RED)
        finally:
            await self.shutdown()
    
    async def initialize(self):
        """Initialize all components"""
        colored_print("🔧 Initializing components...", Fore.YELLOW)
        
        # Test Gate.io connection
        if not await self.gate_client.test_connection():
            raise Exception("Failed to connect to Gate.io")
        
        # Test Bybit connection
        account_info = await self.bybit_client.get_account_info()
        if not account_info:
            raise Exception("Failed to connect to Bybit")
            
        colored_print(f"✅ Connected to Bybit as: {account_info.get('nickname', 'Unknown')}", Fore.GREEN)
        
        # Initialize email monitor
        if self.config.email_monitoring_enabled:
            await self.email_monitor.connect()
            colored_print("✅ Email monitoring initialized", Fore.GREEN)
        
        colored_print("✅ All components initialized successfully", Fore.GREEN)
    
    async def monitor_gate_transactions(self):
        """Monitor Gate.io for pending transactions"""
        colored_print("👀 Starting Gate.io transaction monitoring", Fore.CYAN)
        
        while self.running:
            try:
                # Get pending transactions
                transactions = await self.gate_client.get_pending_transactions()
                
                if transactions:
                    colored_print(f"📦 Found {len(transactions)} pending transactions", Fore.YELLOW)
                    
                    for tx in transactions:
                        if tx.id not in [t.id for t in self.pending_transactions]:
                            self.pending_transactions.append(tx)
                            colored_print(
                                f"🆕 New transaction: {tx.amount} {tx.currency} → "
                                f"{tx.fiat_amount} {tx.fiat_currency} @ {tx.rate}",
                                Fore.GREEN
                            )
                            
                            if self.mode == "manual":
                                await self.handle_transaction_manual(tx)
                
                # Rate limiting - Gate.io allows 240 req/min
                await asyncio.sleep(self.config.gate_poll_interval)
                
            except Exception as e:
                logger.error(f"Error monitoring Gate transactions: {e}")
                await asyncio.sleep(30)  # Wait before retry
    
    async def handle_transaction_manual(self, transaction: Transaction):
        """Handle transaction in manual mode"""
        colored_print("\n" + "="*60, Fore.BLUE)
        colored_print("📊 TRANSACTION DETAILS", Fore.YELLOW, Style.BRIGHT)
        colored_print("="*60, Fore.BLUE)
        
        transaction.display()
        
        # Calculate recommended rate
        recommended_rate = await self.calculate_rate(transaction)
        colored_print(f"\n💡 Recommended rate: {recommended_rate}", Fore.CYAN)
        
        if confirm_action("Create Bybit P2P advertisement for this transaction?"):
            # Get custom rate if desired
            custom_rate = click.prompt(
                "Enter custom rate (or press Enter for recommended)",
                default=str(recommended_rate),
                type=float
            )
            
            # Create advertisement
            ad = await self.create_advertisement(transaction, Decimal(str(custom_rate)))
            if ad:
                colored_print(f"✅ Advertisement created: ID {ad.id}", Fore.GREEN)
                self.active_ads[ad.id] = ad
                
                # Start chat flow
                await self.chat_manager.start_chat_flow(ad)
    
    async def auto_process_transactions(self):
        """Automatically process pending transactions"""
        while self.running:
            try:
                # Process pending transactions
                for tx in self.pending_transactions[:]:  # Copy list to avoid modification issues
                    if tx.id not in [ad.transaction_id for ad in self.active_ads.values()]:
                        # Calculate rate
                        rate = await self.calculate_rate(tx)
                        
                        # Create advertisement
                        ad = await self.create_advertisement(tx, rate)
                        if ad:
                            colored_print(f"🤖 Auto-created ad {ad.id} for transaction {tx.id}", Fore.GREEN)
                            self.active_ads[ad.id] = ad
                            
                            # Start chat flow
                            await self.chat_manager.start_chat_flow(ad)
                            
                            # Remove from pending
                            self.pending_transactions.remove(tx)
                
                await asyncio.sleep(10)  # Check every 10 seconds
                
            except Exception as e:
                logger.error(f"Error in auto-processing: {e}")
                await asyncio.sleep(30)
    
    async def calculate_rate(self, transaction: Transaction) -> Decimal:
        """Calculate optimal rate based on market conditions"""
        try:
            # Get current market rates from Bybit
            market_rates = await self.bybit_client.get_market_rates(
                token=transaction.currency,
                fiat=transaction.fiat_currency
            )
            
            if market_rates:
                # Get average of top 3 rates
                top_rates = [Decimal(str(r['price'])) for r in market_rates[:3]]
                avg_rate = sum(top_rates) / len(top_rates)
                
                # Apply profit margin
                margin = Decimal(str(self.config.profit_margin_percent / 100))
                calculated_rate = avg_rate * (Decimal('1') - margin)
                
                return calculated_rate.quantize(Decimal('0.01'))
            else:
                # Fallback to transaction rate with margin
                margin = Decimal(str(self.config.profit_margin_percent / 100))
                return (transaction.rate * (Decimal('1') - margin)).quantize(Decimal('0.01'))
                
        except Exception as e:
            logger.error(f"Error calculating rate: {e}")
            return transaction.rate
    
    async def create_advertisement(self, transaction: Transaction, rate: Decimal) -> Optional[Advertisement]:
        """Create Bybit P2P advertisement"""
        try:
            params = {
                'asset': transaction.currency,
                'fiat': transaction.fiat_currency,
                'price': str(rate),
                'amount': str(transaction.amount),
                'min_amount': str(self.config.min_order_amount),
                'max_amount': str(min(transaction.fiat_amount, self.config.max_order_amount)),
                'payment_methods': self.config.payment_method_ids,
                'remarks': self.config.ad_remarks,
            }
            
            result = await self.bybit_client.create_advertisement(params)
            
            if result:
                ad = Advertisement(
                    id=result['id'],
                    transaction_id=transaction.id,
                    asset=transaction.currency,
                    fiat=transaction.fiat_currency,
                    price=rate,
                    amount=transaction.amount,
                    status='active',
                    created_at=datetime.now(timezone.utc)
                )
                return ad
            
        except Exception as e:
            logger.error(f"Error creating advertisement: {e}")
            colored_print(f"❌ Failed to create advertisement: {e}", Fore.RED)
        
        return None
    
    async def monitor_active_orders(self):
        """Monitor active orders on Bybit"""
        while self.running:
            try:
                for ad_id, ad in list(self.active_ads.items()):
                    # Get orders for this ad
                    orders = await self.bybit_client.get_ad_orders(ad_id)
                    
                    for order_data in orders:
                        order = Order.from_dict(order_data)
                        
                        # Check if buyer has agreed to terms
                        if await self.chat_manager.check_buyer_agreement(order):
                            # Show payment details
                            await self.chat_manager.send_payment_details(order)
                            
                            # Update order status
                            order.payment_shown = True
                            order.payment_shown_at = datetime.now(timezone.utc)
                
                await asyncio.sleep(30)  # Check every 30 seconds
                
            except Exception as e:
                logger.error(f"Error monitoring orders: {e}")
                await asyncio.sleep(60)
    
    async def monitor_email_receipts(self):
        """Monitor email for payment receipts"""
        if not self.config.email_monitoring_enabled:
            return
            
        while self.running:
            try:
                # Check for new receipts
                receipts = await self.email_monitor.check_receipts()
                
                for receipt in receipts:
                    colored_print(f"📧 New receipt from {receipt['from']}: {receipt['subject']}", Fore.YELLOW)
                    
                    # Validate receipt
                    validation = await self.validate_receipt(receipt)
                    if validation.is_valid:
                        colored_print(f"✅ Valid receipt for {validation.amount} {validation.currency}", Fore.GREEN)
                        
                        # Find matching order
                        order = await self.find_order_by_amount(validation.amount)
                        if order:
                            # Complete the transaction
                            await self.complete_transaction(order, validation)
                    else:
                        colored_print(f"❌ Invalid receipt: {validation.error}", Fore.RED)
                
                await asyncio.sleep(self.config.email_check_interval)
                
            except Exception as e:
                logger.error(f"Error monitoring emails: {e}")
                await asyncio.sleep(60)
    
    async def validate_receipt(self, receipt: Dict[str, Any]) -> ReceiptValidation:
        """Validate payment receipt"""
        try:
            # Check sender
            if receipt['from'] != self.config.required_receipt_sender:
                return ReceiptValidation(
                    is_valid=False,
                    error=f"Invalid sender: {receipt['from']}"
                )
            
            # Process attachments
            for attachment in receipt.get('attachments', []):
                if attachment['filename'].lower().endswith('.pdf'):
                    # Process PDF receipt
                    validation = await self.email_monitor.process_pdf_receipt(
                        attachment['content'],
                        expected_amount=None  # Will match against active orders
                    )
                    if validation.is_valid:
                        return validation
            
            return ReceiptValidation(
                is_valid=False,
                error="No valid PDF receipt found"
            )
            
        except Exception as e:
            logger.error(f"Error validating receipt: {e}")
            return ReceiptValidation(
                is_valid=False,
                error=str(e)
            )
    
    async def find_order_by_amount(self, amount: Decimal) -> Optional[Order]:
        """Find order matching the payment amount"""
        for ad in self.active_ads.values():
            orders = await self.bybit_client.get_ad_orders(ad.id)
            for order_data in orders:
                order = Order.from_dict(order_data)
                if abs(order.amount - amount) < Decimal('0.01'):
                    return order
        return None
    
    async def complete_transaction(self, order: Order, validation: ReceiptValidation):
        """Complete the transaction after receipt validation"""
        colored_print(f"💰 Completing transaction for order {order.id}", Fore.GREEN)
        
        try:
            # Release funds on Bybit
            if await self.bybit_client.release_order(order.id):
                colored_print(f"✅ Funds released on Bybit", Fore.GREEN)
            
            # Complete transaction on Gate.io
            # Find the original transaction
            for ad in self.active_ads.values():
                if any(o.id == order.id for o in await self.bybit_client.get_ad_orders(ad.id)):
                    tx_id = ad.transaction_id
                    if await self.gate_client.complete_transaction(tx_id):
                        colored_print(f"✅ Transaction completed on Gate.io", Fore.GREEN)
                    break
            
            # Send completion message
            await self.chat_manager.send_completion_message(order)
            
        except Exception as e:
            logger.error(f"Error completing transaction: {e}")
            colored_print(f"❌ Error completing transaction: {e}", Fore.RED)
    
    async def shutdown(self):
        """Shutdown the system gracefully"""
        colored_print("\n🛑 Shutting down trader system...", Fore.YELLOW)
        self.running = False
        
        # Close connections
        await self.email_monitor.disconnect()
        
        colored_print("👋 Trader system stopped", Fore.GREEN)


@click.command()
@click.option('--mode', type=click.Choice(['manual', 'automatic']), default='manual', help='Operation mode')
@click.option('--config', default='config.toml', help='Configuration file path')
def main(mode: str, config: str):
    """Auto-Trader System - Monitor Gate.io and create Bybit P2P ads"""
    
    # Load configuration
    try:
        cfg = Config.load(config)
    except Exception as e:
        colored_print(f"❌ Failed to load config: {e}", Fore.RED)
        sys.exit(1)
    
    # Create and run trader system
    trader = TraderSystem(cfg)
    
    # Setup signal handlers
    def signal_handler(sig, frame):
        colored_print("\n🛑 Received interrupt signal...", Fore.YELLOW)
        trader.running = False
        sys.exit(0)
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Run the system
    try:
        asyncio.run(trader.start(mode))
    except KeyboardInterrupt:
        colored_print("\n👋 Bye!", Fore.GREEN)
    except Exception as e:
        colored_print(f"❌ Fatal error: {e}", Fore.RED)
        sys.exit(1)


if __name__ == "__main__":
    main()