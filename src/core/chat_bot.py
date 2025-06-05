#!/usr/bin/env python3
"""
P2P Chat Bot
Handles chat interactions with buyers according to script
"""

import threading
import time
import re
import psycopg2
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple
from rich.console import Console
from rich.prompt import Confirm

# Import our modules
import sys
sys.path.append('.')
from scripts.bybit_p2p_order_manager import P2POrderManager

console = Console()

class P2PChatBot:
    def __init__(self, db_url: str, auto_mode: bool = False):
        self.db_url = db_url
        self.auto_mode = auto_mode
        self.running = False
        self.monitored_ads = {}  # {ad_id: transaction_id}
        
        # Chat stages
        self.STAGES = {
            'greeting': 1,
            'bank_confirm': 2,
            'receipt_confirm': 3,
            'kyc_confirm': 4,
            'reqs_sent': 5,
            'waiting_receipt': 6,
            'completed': 7
        }
        
        # Response patterns
        self.YES_PATTERNS = [
            r'\bда\b', r'\byes\b', r'\bдa\b', r'\bконечно\b', 
            r'\bсогласен\b', r'\bок\b', r'\bокей\b', r'\b\+\b'
        ]
        
        self.NO_PATTERNS = [
            r'\bнет\b', r'\bno\b', r'\bне\b', r'\bотказ\b',
            r'\bне согласен\b', r'\b\-\b'
        ]
        
        self.CONFIRM_PATTERNS = [
            r'\bподтверждаю\b', r'\bconfirm\b', r'\bпринимаю\b',
            r'\bсогласен\b', r'\bок\b'
        ]
    
    def get_db_connection(self):
        """Get database connection"""
        return psycopg2.connect(self.db_url)
    
    def start(self):
        """Start chat bot monitoring"""
        self.running = True
        monitor_thread = threading.Thread(target=self._monitor_orders, daemon=True)
        monitor_thread.start()
        console.print("[cyan]🤖 Chat bot started[/cyan]")
    
    def stop(self):
        """Stop chat bot"""
        self.running = False
    
    def monitor_ad(self, transaction_id: int, ad_id: str):
        """Start monitoring specific ad for orders"""
        self.monitored_ads[ad_id] = transaction_id
        console.print(f"[cyan]👀 Monitoring ad {ad_id} for transaction {transaction_id}[/cyan]")
    
    def _monitor_orders(self):
        """Monitor orders for all tracked ads"""
        while self.running:
            try:
                for ad_id, transaction_id in list(self.monitored_ads.items()):
                    self._check_ad_orders(ad_id, transaction_id)
                
                time.sleep(30)  # Check every 30 seconds
                
            except Exception as e:
                console.print(f"[red]Chat bot monitoring error: {e}[/red]")
                time.sleep(60)
    
    def _check_ad_orders(self, ad_id: str, transaction_id: int):
        """Check for new orders on specific ad"""
        try:
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            # Get Bybit credentials
            cur.execute("""
                SELECT b.api_key, b.api_secret, t.bybit_order_id
                FROM transactions t
                JOIN bybit_accounts b ON t.bybit_account_id = b.id
                WHERE t.id = %s
            """, (transaction_id,))
            
            result = cur.fetchone()
            if not result:
                return
            
            api_key, api_secret, current_order_id = result
            
            # Get orders for this ad
            manager = P2POrderManager(api_key, api_secret)
            orders = manager.get_orders(status=20)  # Waiting for seller to release
            
            for order in orders:
                order_id = order.get('id')
                
                # Check if this is a new order
                if order_id != current_order_id:
                    # New order found!
                    console.print(f"[green]🔔 New order {order_id} for ad {ad_id}[/green]")
                    
                    # Update transaction with order ID
                    cur.execute("""
                        UPDATE transactions 
                        SET bybit_order_id = %s, status = 'waiting_payment',
                            chat_stage = 'greeting'
                        WHERE id = %s
                    """, (order_id, transaction_id))
                    conn.commit()
                    
                    # Start chat interaction
                    self._handle_order_chat(transaction_id, order_id, api_key, api_secret)
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Error checking ad orders: {e}[/red]")
    
    def _handle_order_chat(self, transaction_id: int, order_id: str, 
                          api_key: str, api_secret: str):
        """Handle chat interaction for order"""
        try:
            manager = P2POrderManager(api_key, api_secret)
            
            # Get current chat stage
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            cur.execute("""
                SELECT chat_stage, wallet, bank_label, amount_rub
                FROM transactions
                WHERE id = %s
            """, (transaction_id,))
            
            result = cur.fetchone()
            if not result:
                return
            
            chat_stage, wallet, bank_label, amount_rub = result
            
            # Get chat messages
            messages = manager.get_chat_messages(order_id)
            
            # Process based on stage
            if chat_stage == 'greeting':
                self._send_greeting(transaction_id, order_id, manager)
                
            elif chat_stage == 'bank_confirm':
                self._check_bank_response(transaction_id, order_id, messages, manager)
                
            elif chat_stage == 'receipt_confirm':
                self._check_receipt_response(transaction_id, order_id, messages, manager)
                
            elif chat_stage == 'kyc_confirm':
                self._check_kyc_response(transaction_id, order_id, messages, manager)
                
            elif chat_stage == 'reqs_sent':
                self._send_requisites(transaction_id, order_id, wallet, 
                                    bank_label, amount_rub, manager)
                
            elif chat_stage == 'waiting_receipt':
                self._check_for_receipt(transaction_id, order_id, messages, manager)
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Error handling order chat: {e}[/red]")
    
    def _send_greeting(self, transaction_id: int, order_id: str, 
                      manager: P2POrderManager):
        """Send initial greeting"""
        message = "Здравствуйте!\nОплата будет с Т банка?\n(просто напишите да/нет)"
        
        if not self.auto_mode:
            console.print(f"\n[yellow]Sending greeting to order {order_id}[/yellow]")
            console.print(f"Message: {message}")
            if not Confirm.ask("Send message?"):
                return
        
        if manager.send_message(order_id, message):
            self._update_chat_stage(transaction_id, 'bank_confirm')
            self._log_message(transaction_id, order_id, 'out', message)
    
    def _check_bank_response(self, transaction_id: int, order_id: str,
                           messages: List[Dict], manager: P2POrderManager):
        """Check bank confirmation response"""
        # Find latest user message
        user_message = self._get_latest_user_message(messages)
        
        if not user_message:
            return
        
        text = user_message.get('message', '').lower()
        
        if self._matches_patterns(text, self.YES_PATTERNS):
            # Bank confirmed, move to receipt confirmation
            message = ("Чек в формате пдф с официальной почты Т банка сможете отправить?\n"
                      "(просто напишите да/нет)")
            
            if manager.send_message(order_id, message):
                self._update_chat_stage(transaction_id, 'receipt_confirm')
                self._log_message(transaction_id, order_id, 'out', message)
                
        elif self._matches_patterns(text, self.NO_PATTERNS):
            # Not T-Bank, move to fool pool
            self._move_to_fool_pool(transaction_id, "Not using T-Bank")
            
        else:
            # Unclear response, repeat question
            message = ("Я все делаю строго по инструкции.\n"
                      "Оплата будет с Т банка?\n(просто напишите да/нет)")
            manager.send_message(order_id, message)
    
    def _check_receipt_response(self, transaction_id: int, order_id: str,
                              messages: List[Dict], manager: P2POrderManager):
        """Check receipt confirmation response"""
        user_message = self._get_latest_user_message(messages)
        
        if not user_message:
            return
        
        text = user_message.get('message', '').lower()
        
        if self._matches_patterns(text, self.YES_PATTERNS):
            # Receipt confirmed, move to KYC warning
            message = ("При СБП, если оплата будет на неверный банк, деньги потеряны.\n"
                      "(просто напишите подтверждаю/не подтверждаю)")
            
            if manager.send_message(order_id, message):
                self._update_chat_stage(transaction_id, 'kyc_confirm')
                self._log_message(transaction_id, order_id, 'out', message)
                
        elif self._matches_patterns(text, self.NO_PATTERNS):
            # No receipt, move to fool pool
            self._move_to_fool_pool(transaction_id, "Cannot provide receipt")
            
        else:
            # Unclear response, repeat question
            message = ("Я все делаю строго по инструкции.\n"
                      "Чек в формате пдф с официальной почты Т банка сможете отправить?\n"
                      "(просто напишите да/нет)")
            manager.send_message(order_id, message)
    
    def _check_kyc_response(self, transaction_id: int, order_id: str,
                          messages: List[Dict], manager: P2POrderManager):
        """Check KYC confirmation response"""
        user_message = self._get_latest_user_message(messages)
        
        if not user_message:
            return
        
        text = user_message.get('message', '').lower()
        
        if self._matches_patterns(text, self.CONFIRM_PATTERNS):
            # KYC confirmed, send requisites
            self._update_chat_stage(transaction_id, 'reqs_sent')
            
        elif self._matches_patterns(text, ['не подтверждаю', 'не согласен', 'отказ']):
            # Not confirmed, move to fool pool
            self._move_to_fool_pool(transaction_id, "KYC not confirmed")
            
        else:
            # Unclear response, repeat question
            message = ("Я все делаю строго по инструкции.\n"
                      "При СБП, если оплата будет на неверный банк, деньги потеряны.\n"
                      "(просто напишите подтверждаю/не подтверждаю)")
            manager.send_message(order_id, message)
    
    def _send_requisites(self, transaction_id: int, order_id: str,
                       wallet: str, bank_label: str, amount_rub: float,
                       manager: P2POrderManager):
        """Send payment requisites"""
        message = (f"Реквизиты для оплаты:\n\n"
                  f"Банк: {bank_label}\n"
                  f"Получатель: {wallet}\n"
                  f"Сумма: {amount_rub} RUB\n\n"
                  f"После оплаты обязательно отправьте чек в формате PDF")
        
        if not self.auto_mode:
            console.print(f"\n[yellow]Sending requisites[/yellow]")
            console.print(f"Message: {message}")
            if not Confirm.ask("Send requisites?"):
                return
        
        if manager.send_message(order_id, message):
            self._update_chat_stage(transaction_id, 'waiting_receipt')
            self._log_message(transaction_id, order_id, 'out', message)
            
            # Set receipt timeout (10 minutes from now)
            conn = self.get_db_connection()
            cur = conn.cursor()
            cur.execute("""
                UPDATE transactions 
                SET updated_at = CURRENT_TIMESTAMP
                WHERE id = %s
            """, (transaction_id,))
            conn.commit()
            conn.close()
    
    def _check_for_receipt(self, transaction_id: int, order_id: str,
                         messages: List[Dict], manager: P2POrderManager):
        """Check if receipt was sent"""
        # Check for PDF attachments in recent messages
        for msg in messages[-5:]:  # Check last 5 messages
            if msg.get('contentType') == 'pdf':
                console.print(f"[green]📄 Receipt PDF found in chat[/green]")
                # Receipt processing will be handled by OCR module
                return
        
        # Check timeout (10 minutes)
        conn = self.get_db_connection()
        cur = conn.cursor()
        cur.execute("""
            SELECT updated_at FROM transactions WHERE id = %s
        """, (transaction_id,))
        
        updated_at = cur.fetchone()[0]
        conn.close()
        
        if datetime.now() - updated_at > timedelta(minutes=10):
            # Timeout, move to fool pool
            self._move_to_fool_pool(transaction_id, "Receipt timeout (10 minutes)")
    
    def _matches_patterns(self, text: str, patterns: List[str]) -> bool:
        """Check if text matches any pattern"""
        for pattern in patterns:
            if re.search(pattern, text, re.IGNORECASE):
                return True
        return False
    
    def _get_latest_user_message(self, messages: List[Dict]) -> Optional[Dict]:
        """Get latest message from user"""
        for msg in reversed(messages):
            if msg.get('msgType') in [1, 2, 7, 8]:  # User messages
                return msg
        return None
    
    def _update_chat_stage(self, transaction_id: int, stage: str):
        """Update chat stage in database"""
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                UPDATE transactions 
                SET chat_stage = %s, updated_at = CURRENT_TIMESTAMP
                WHERE id = %s
            """, (stage, transaction_id))
            conn.commit()
        finally:
            conn.close()
    
    def _log_message(self, transaction_id: int, order_id: str, 
                    direction: str, message: str):
        """Log chat message to database"""
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                INSERT INTO chat_messages 
                (transaction_id, order_id, direction, message)
                VALUES (%s, %s, %s, %s)
            """, (transaction_id, order_id, direction, message))
            conn.commit()
        finally:
            conn.close()
    
    def _move_to_fool_pool(self, transaction_id: int, reason: str):
        """Move transaction to fool pool"""
        console.print(f"[red]❌ Moving transaction {transaction_id} to fool pool: {reason}[/red]")
        
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                UPDATE transactions 
                SET status = 'fool_pool', error_reason = %s,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = %s
            """, (reason, transaction_id))
            conn.commit()
            
            # Remove from monitoring
            # Find and remove ad_id from monitored_ads
            for ad_id, tx_id in list(self.monitored_ads.items()):
                if tx_id == transaction_id:
                    del self.monitored_ads[ad_id]
                    break
                    
        finally:
            conn.close()