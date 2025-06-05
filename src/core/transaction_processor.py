#!/usr/bin/env python3
"""
Transaction Processor
Handles transaction processing, ad creation, and order management
"""

import queue
import threading
import time
import psycopg2
from datetime import datetime, timedelta
from typing import Dict, Optional, Tuple
from rich.console import Console
from rich.prompt import Confirm

# Import our modules
import sys
sys.path.append('.')
from scripts.bybit_smart_ad_creator import SmartAdCreator
from scripts.bybit_p2p_order_manager import P2POrderManager
from src.core.chat_bot import P2PChatBot

console = Console()

class TransactionProcessor:
    def __init__(self, db_url: str, auto_mode: bool = False):
        self.db_url = db_url
        self.auto_mode = auto_mode
        self.processing_queue = queue.Queue()
        self.running = False
        self.chat_bot = P2PChatBot(db_url, auto_mode)
        
    def get_db_connection(self):
        """Get database connection"""
        return psycopg2.connect(self.db_url)
    
    def add_to_queue(self, transaction_id: int):
        """Add transaction to processing queue"""
        self.processing_queue.put(transaction_id)
    
    def process_queue(self):
        """Process transactions from queue"""
        self.running = True
        console.print("[cyan]📦 Transaction processor started[/cyan]")
        
        while self.running:
            try:
                # Get transaction from queue (wait up to 1 second)
                transaction_id = self.processing_queue.get(timeout=1)
                self.process_transaction(transaction_id)
                
            except queue.Empty:
                continue
            except Exception as e:
                console.print(f"[red]Transaction processor error: {e}[/red]")
    
    def stop(self):
        """Stop processor"""
        self.running = False
        self.chat_bot.stop()
    
    def process_transaction(self, transaction_id: int):
        """Process a single transaction"""
        try:
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            # Get transaction details
            cur.execute("""
                SELECT t.*, g.login as gate_login, g.password as gate_password
                FROM transactions t
                JOIN gate_accounts g ON t.gate_account_id = g.id
                WHERE t.id = %s
            """, (transaction_id,))
            
            tx = cur.fetchone()
            if not tx:
                console.print(f"[red]Transaction {transaction_id} not found[/red]")
                return
            
            # Update status to processing
            self.update_transaction_status(transaction_id, 'processing')
            
            # Find available Bybit account (max 1 active ad)
            bybit_account = self.find_available_bybit_account()
            
            if not bybit_account:
                console.print("[red]No available Bybit accounts[/red]")
                self.update_transaction_status(transaction_id, 'error', 
                                             'No available Bybit accounts')
                return
            
            # Create P2P ad
            ad_id = self.create_p2p_ad(transaction_id, tx, bybit_account)
            
            if ad_id:
                # Update transaction with Bybit account and ad ID
                cur.execute("""
                    UPDATE transactions 
                    SET bybit_account_id = %s, bybit_ad_id = %s,
                        status = 'waiting_response'
                    WHERE id = %s
                """, (bybit_account['id'], ad_id, transaction_id))
                conn.commit()
                
                # Start monitoring for orders on this ad
                self.chat_bot.monitor_ad(transaction_id, ad_id)
                
            else:
                self.update_transaction_status(transaction_id, 'error',
                                             'Failed to create P2P ad')
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Error processing transaction {transaction_id}: {e}[/red]")
            self.update_transaction_status(transaction_id, 'error', str(e))
    
    def find_available_bybit_account(self) -> Optional[Dict]:
        """Find Bybit account with ≤1 active ad"""
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            
            # Get all active Bybit accounts
            cur.execute("""
                SELECT b.id, b.name, b.api_key, b.api_secret
                FROM bybit_accounts b
                WHERE b.is_active = true
            """)
            
            accounts = cur.fetchall()
            
            for acc_id, name, api_key, api_secret in accounts:
                # Check active ads count
                creator = SmartAdCreator(api_key, api_secret)
                active_ads = creator.get_active_ads()
                
                if len(active_ads) <= 1:
                    return {
                        'id': acc_id,
                        'name': name,
                        'api_key': api_key,
                        'api_secret': api_secret
                    }
            
            return None
            
        finally:
            conn.close()
    
    def create_p2p_ad(self, transaction_id: int, tx_data: tuple, 
                     bybit_account: Dict) -> Optional[str]:
        """Create P2P ad on Bybit"""
        try:
            amount_rub = float(tx_data[4])  # amount_rub from transaction
            
            if not self.auto_mode:
                console.print(f"\n[yellow]📝 Creating P2P ad[/yellow]")
                console.print(f"Amount: {amount_rub} RUB")
                console.print(f"Bybit account: {bybit_account['name']}")
                
                if not Confirm.ask("Create ad?"):
                    return None
            
            # Create ad using smart creator
            creator = SmartAdCreator(
                bybit_account['api_key'],
                bybit_account['api_secret']
            )
            
            result = creator.create_smart_ad({
                "amount": str(amount_rub),
                "remark": "Быстрая продажа USDT. Оплата Т-Банк"
            })
            
            if result.get("ret_code") == 0:
                ad_id = result.get("result", {}).get("itemId")
                console.print(f"[green]✅ P2P ad created: {ad_id}[/green]")
                return ad_id
            else:
                console.print(f"[red]❌ Failed to create ad: {result.get('ret_msg')}[/red]")
                return None
                
        except Exception as e:
            console.print(f"[red]Error creating P2P ad: {e}[/red]")
            return None
    
    def update_transaction_status(self, transaction_id: int, status: str, 
                                error_reason: str = None):
        """Update transaction status in database"""
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            
            if error_reason:
                cur.execute("""
                    UPDATE transactions 
                    SET status = %s, error_reason = %s, 
                        updated_at = CURRENT_TIMESTAMP
                    WHERE id = %s
                """, (status, error_reason, transaction_id))
            else:
                cur.execute("""
                    UPDATE transactions 
                    SET status = %s, updated_at = CURRENT_TIMESTAMP
                    WHERE id = %s
                """, (status, transaction_id))
            
            conn.commit()
            
        except Exception as e:
            conn.rollback()
            console.print(f"[red]Error updating transaction status: {e}[/red]")
        finally:
            conn.close()
    
    def process_receipt(self, receipt_id: int):
        """Process receipt with OCR and matching"""
        # This will be called from monitoring when new receipt arrives
        # OCR processing will be implemented in separate module
        pass
    
    def release_funds(self, transaction_id: int, order_id: str):
        """Release funds for approved transaction"""
        try:
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            # Get Bybit account credentials
            cur.execute("""
                SELECT b.api_key, b.api_secret
                FROM transactions t
                JOIN bybit_accounts b ON t.bybit_account_id = b.id
                WHERE t.id = %s
            """, (transaction_id,))
            
            result = cur.fetchone()
            if not result:
                console.print(f"[red]Transaction {transaction_id} not found[/red]")
                return
            
            api_key, api_secret = result
            
            # Release assets
            manager = P2POrderManager(api_key, api_secret)
            success = manager.release_assets(order_id)
            
            if success:
                # Update transaction
                cur.execute("""
                    UPDATE transactions 
                    SET status = 'released', released_at = CURRENT_TIMESTAMP
                    WHERE id = %s
                """, (transaction_id,))
                
                # Send final message
                manager.send_message(
                    order_id,
                    "Переходи в закрытый чат https://t.me/+nIB6kP22KmhlMmQy\n\n"
                    "Всегда есть большой объем ЮСДТ по хорошему курсу, работаем оперативно."
                )
                
                conn.commit()
                console.print(f"[green]✅ Funds released for transaction {transaction_id}[/green]")
            else:
                console.print(f"[red]Failed to release funds for transaction {transaction_id}[/red]")
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Error releasing funds: {e}[/red]")