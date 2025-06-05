#!/usr/bin/env python3
"""
Transaction and Email Monitoring System
"""

import time
import threading
import json
import psycopg2
from datetime import datetime, timedelta
from typing import List, Dict, Optional
import base64
from email.mime.text import MIMEText
from googleapiclient.errors import HttpError

# Import our modules
import sys
sys.path.append('.')
from src.gmail.auth import GmailAuthManager
from src.gate.client import GateClient
from src.core.transaction_processor import TransactionProcessor
from rich.console import Console

console = Console()

class MonitoringSystem:
    def __init__(self, db_url: str, auto_mode: bool = False):
        self.db_url = db_url
        self.auto_mode = auto_mode
        self.running = False
        self.gmail_manager = GmailAuthManager(db_url)
        self.transaction_processor = TransactionProcessor(db_url, auto_mode)
        
        # Threading events
        self.stop_event = threading.Event()
        self.gate_check_event = threading.Event()
        
    def get_db_connection(self):
        """Get database connection"""
        return psycopg2.connect(self.db_url)
    
    def start(self):
        """Start monitoring threads"""
        self.running = True
        console.print("[green]🚀 Starting monitoring system...[/green]")
        
        # Start email monitoring thread
        email_thread = threading.Thread(target=self.monitor_emails, daemon=True)
        email_thread.start()
        
        # Start Gate.io transaction monitoring thread
        gate_thread = threading.Thread(target=self.monitor_gate_transactions, daemon=True)
        gate_thread.start()
        
        # Start transaction processor thread
        processor_thread = threading.Thread(target=self.transaction_processor.process_queue, daemon=True)
        processor_thread.start()
        
        # Start release scheduler thread
        release_thread = threading.Thread(target=self.release_scheduler, daemon=True)
        release_thread.start()
        
        console.print("[green]✅ All monitoring threads started[/green]")
        
        # Keep main thread alive
        try:
            while self.running:
                time.sleep(1)
        except KeyboardInterrupt:
            self.stop()
    
    def stop(self):
        """Stop monitoring"""
        console.print("\n[yellow]⏹️  Stopping monitoring system...[/yellow]")
        self.running = False
        self.stop_event.set()
        self.transaction_processor.stop()
        time.sleep(2)
        console.print("[green]✅ Monitoring stopped[/green]")
    
    def monitor_emails(self):
        """Monitor Gmail for new receipts"""
        console.print("[cyan]📧 Email monitoring started[/cyan]")
        
        while self.running:
            try:
                # Get Gmail service
                service = self.gmail_manager.get_gmail_service()
                
                # Query for recent emails from T Bank
                query = 'from:(noreply@tinkoff.ru OR noreply@tbank.ru) subject:(чек OR квитанция) is:unread'
                
                results = service.users().messages().list(
                    userId='me',
                    q=query,
                    maxResults=10
                ).execute()
                
                messages = results.get('messages', [])
                
                for msg in messages:
                    # Get full message
                    message = service.users().messages().get(
                        userId='me',
                        id=msg['id']
                    ).execute()
                    
                    # Process receipt
                    self.process_email_receipt(message)
                    
                    # Mark as read
                    service.users().messages().modify(
                        userId='me',
                        id=msg['id'],
                        body={'removeLabelIds': ['UNREAD']}
                    ).execute()
                
                # Check every 30 seconds
                time.sleep(30)
                
            except Exception as e:
                console.print(f"[red]Email monitoring error: {e}[/red]")
                time.sleep(60)  # Wait longer on error
    
    def process_email_receipt(self, message: dict):
        """Process email receipt"""
        try:
            # Extract email data
            headers = message['payload']['headers']
            subject = next(h['value'] for h in headers if h['name'] == 'Subject')
            sender = next(h['value'] for h in headers if h['name'] == 'From')
            email_id = message['id']
            
            # Find PDF attachment
            pdf_data = None
            for part in message['payload'].get('parts', []):
                if part['filename'] and part['filename'].endswith('.pdf'):
                    attachment_id = part['body']['attachmentId']
                    
                    # Get attachment
                    service = self.gmail_manager.get_gmail_service()
                    att = service.users().messages().attachments().get(
                        userId='me',
                        messageId=email_id,
                        id=attachment_id
                    ).execute()
                    
                    pdf_data = base64.urlsafe_b64decode(att['data'])
                    break
            
            if pdf_data:
                # Save receipt to database
                self.save_receipt(email_id, sender, subject, pdf_data)
                console.print(f"[green]📧 New receipt processed: {subject}[/green]")
            
        except Exception as e:
            console.print(f"[red]Error processing email: {e}[/red]")
    
    def save_receipt(self, email_id: str, sender: str, subject: str, pdf_data: bytes):
        """Save receipt to database"""
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            
            # Save receipt
            cur.execute("""
                INSERT INTO receipts (email_id, sender_email, subject, pdf_content)
                VALUES (%s, %s, %s, %s)
                RETURNING id
            """, (email_id, sender, subject, pdf_data))
            
            receipt_id = cur.fetchone()[0]
            conn.commit()
            
            # Trigger OCR processing
            self.transaction_processor.process_receipt(receipt_id)
            
        except Exception as e:
            conn.rollback()
            console.print(f"[red]Error saving receipt: {e}[/red]")
        finally:
            conn.close()
    
    def monitor_gate_transactions(self):
        """Monitor Gate.io pending transactions every 5 minutes"""
        console.print("[cyan]🏦 Gate.io transaction monitoring started[/cyan]")
        
        # First check immediately
        self.check_gate_transactions()
        
        while self.running:
            # Wait exactly 5 minutes
            if self.stop_event.wait(300):  # 300 seconds = 5 minutes
                break
            
            self.check_gate_transactions()
    
    def check_gate_transactions(self):
        """Check Gate.io for pending transactions"""
        try:
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            # Get active Gate accounts
            cur.execute("""
                SELECT id, login, password, uid 
                FROM gate_accounts 
                WHERE is_active = true
            """)
            
            accounts = cur.fetchall()
            
            for account_id, login, password, uid in accounts:
                try:
                    # Get Gate client
                    gate_client = GateClient(login, password)
                    
                    # Get pending transactions
                    pending_txs = gate_client.get_pending_transactions()
                    
                    console.print(f"[cyan]Gate account {login}: Found {len(pending_txs)} pending transactions[/cyan]")
                    
                    for tx in pending_txs:
                        self.process_gate_transaction(account_id, tx)
                    
                except Exception as e:
                    console.print(f"[red]Error checking Gate account {login}: {e}[/red]")
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Gate monitoring error: {e}[/red]")
    
    def process_gate_transaction(self, gate_account_id: int, tx: dict):
        """Process Gate.io transaction"""
        try:
            # Extract transaction data
            gate_tx_id = tx.get('id', '')
            amount_data = tx.get('amount', {}).get('trader', {})
            amount_rub = float(amount_data.get('643', 0))  # 643 is RUB code
            wallet = tx.get('wallet', '')
            bank_data = tx.get('bank', {})
            
            if amount_rub <= 0:
                return
            
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            # Check if transaction already exists
            cur.execute("""
                SELECT id FROM transactions 
                WHERE gate_transaction_id = %s
            """, (gate_tx_id,))
            
            if cur.fetchone():
                conn.close()
                return  # Already processed
            
            # Create new transaction record
            cur.execute("""
                INSERT INTO transactions (
                    gate_transaction_id, status, gate_account_id,
                    amount_rub, wallet, bank_label, bank_code
                ) VALUES (%s, %s, %s, %s, %s, %s, %s)
                RETURNING id
            """, (
                gate_tx_id, 'pending', gate_account_id,
                amount_rub, wallet, 
                bank_data.get('label', ''),
                bank_data.get('code', '')
            ))
            
            transaction_id = cur.fetchone()[0]
            conn.commit()
            conn.close()
            
            console.print(f"[green]💰 New transaction: {amount_rub} RUB from {wallet}[/green]")
            
            # Add to processing queue
            self.transaction_processor.add_to_queue(transaction_id)
            
        except Exception as e:
            console.print(f"[red]Error processing Gate transaction: {e}[/red]")
    
    def release_scheduler(self):
        """Monitor and release funds when scheduled"""
        console.print("[cyan]⏰ Release scheduler started[/cyan]")
        
        while self.running:
            try:
                conn = self.get_db_connection()
                cur = conn.cursor()
                
                # Find transactions ready for release
                cur.execute("""
                    SELECT id, bybit_order_id 
                    FROM transactions 
                    WHERE status = 'approved' 
                    AND release_scheduled_at <= CURRENT_TIMESTAMP
                    AND released_at IS NULL
                """)
                
                ready_transactions = cur.fetchall()
                
                for tx_id, order_id in ready_transactions:
                    if order_id:
                        console.print(f"[yellow]💸 Releasing funds for order {order_id}[/yellow]")
                        self.transaction_processor.release_funds(tx_id, order_id)
                
                conn.close()
                
                # Check every 30 seconds
                time.sleep(30)
                
            except Exception as e:
                console.print(f"[red]Release scheduler error: {e}[/red]")
                time.sleep(60)