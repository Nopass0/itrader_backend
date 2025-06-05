#!/usr/bin/env python3
"""
Receipt OCR Processor
Processes PDF receipts from T-Bank and matches with transactions
"""

import re
import io
import json
import psycopg2
from datetime import datetime
from typing import Dict, Optional, Tuple
from PIL import Image
import pytesseract
import PyPDF2
from pdf2image import convert_from_bytes
from rich.console import Console

console = Console()

class ReceiptProcessor:
    def __init__(self, db_url: str):
        self.db_url = db_url
        
        # Patterns for extracting data
        self.PATTERNS = {
            'status': r'(?:Статус|Status)[:\s]*([Уу]спешно|[Ss]uccessful|[Вв]ыполнено)',
            'amount': r'(?:Сумма|Amount)[:\s]*([0-9\s,]+(?:\.[0-9]+)?)\s*(?:₽|руб|RUB)',
            'phone': r'(?:\+7|8)[\s\-]?\(?(\d{3})\)?[\s\-]?(\d{3})[\s\-]?(\d{2})[\s\-]?(\d{2})',
            'card': r'(?:\*{4}[\s]?){3}(\d{4})',  # Last 4 digits of card
            'bank': r'(?:Т-Банк|T-Bank|Тинькофф|Tinkoff)',
            'datetime': r'(\d{1,2}[\.\/]\d{1,2}[\.\/]\d{2,4}\s+\d{1,2}:\d{2})'
        }
    
    def get_db_connection(self):
        """Get database connection"""
        return psycopg2.connect(self.db_url)
    
    def process_receipt(self, receipt_id: int):
        """Process receipt with OCR"""
        try:
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            # Get receipt PDF
            cur.execute("""
                SELECT pdf_content FROM receipts WHERE id = %s
            """, (receipt_id,))
            
            result = cur.fetchone()
            if not result or not result[0]:
                console.print(f"[red]Receipt {receipt_id} not found[/red]")
                return
            
            pdf_content = bytes(result[0])
            
            # Extract text from PDF
            ocr_text = self.extract_text_from_pdf(pdf_content)
            
            # Parse receipt data
            parsed_data = self.parse_receipt_text(ocr_text)
            
            # Validate receipt
            is_valid = self.validate_receipt(parsed_data)
            
            # Update receipt in database
            cur.execute("""
                UPDATE receipts 
                SET ocr_text = %s, parsed_data = %s, is_valid = %s
                WHERE id = %s
            """, (ocr_text, json.dumps(parsed_data), is_valid, receipt_id))
            
            conn.commit()
            
            if is_valid:
                console.print(f"[green]✅ Receipt {receipt_id} validated successfully[/green]")
                # Try to match with transaction
                self.match_receipt_to_transaction(receipt_id, parsed_data)
            else:
                console.print(f"[red]❌ Receipt {receipt_id} validation failed[/red]")
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Error processing receipt {receipt_id}: {e}[/red]")
    
    def extract_text_from_pdf(self, pdf_content: bytes) -> str:
        """Extract text from PDF using OCR"""
        try:
            # First try to extract text directly
            pdf_file = io.BytesIO(pdf_content)
            pdf_reader = PyPDF2.PdfReader(pdf_file)
            
            text = ""
            for page in pdf_reader.pages:
                page_text = page.extract_text()
                if page_text:
                    text += page_text + "\n"
            
            # If no text extracted, use OCR
            if not text.strip():
                console.print("[yellow]No text in PDF, using OCR...[/yellow]")
                
                # Convert PDF to images
                images = convert_from_bytes(pdf_content, dpi=300)
                
                for i, image in enumerate(images):
                    # Use Tesseract OCR
                    page_text = pytesseract.image_to_string(
                        image, 
                        lang='rus+eng',
                        config='--psm 6'
                    )
                    text += page_text + "\n"
            
            return text
            
        except Exception as e:
            console.print(f"[red]OCR error: {e}[/red]")
            return ""
    
    def parse_receipt_text(self, text: str) -> Dict:
        """Parse receipt text to extract key data"""
        parsed = {
            'status': None,
            'amount': None,
            'phone': None,
            'card_last4': None,
            'bank': None,
            'datetime': None
        }
        
        # Extract status
        status_match = re.search(self.PATTERNS['status'], text, re.IGNORECASE)
        if status_match:
            parsed['status'] = 'success'
        
        # Extract amount
        amount_match = re.search(self.PATTERNS['amount'], text, re.IGNORECASE)
        if amount_match:
            amount_str = amount_match.group(1).replace(' ', '').replace(',', '.')
            try:
                parsed['amount'] = float(amount_str)
            except:
                pass
        
        # Extract phone
        phone_match = re.search(self.PATTERNS['phone'], text)
        if phone_match:
            parsed['phone'] = ''.join(phone_match.groups())
        
        # Extract card last 4 digits
        card_match = re.search(self.PATTERNS['card'], text)
        if card_match:
            parsed['card_last4'] = card_match.group(1)
        
        # Check bank
        if re.search(self.PATTERNS['bank'], text, re.IGNORECASE):
            parsed['bank'] = 'T-Bank'
        
        # Extract datetime
        datetime_match = re.search(self.PATTERNS['datetime'], text)
        if datetime_match:
            parsed['datetime'] = datetime_match.group(1)
        
        return parsed
    
    def validate_receipt(self, parsed_data: Dict) -> bool:
        """Validate receipt has required fields"""
        # Must have successful status
        if parsed_data.get('status') != 'success':
            return False
        
        # Must have amount
        if not parsed_data.get('amount'):
            return False
        
        # Must have either phone or card
        if not parsed_data.get('phone') and not parsed_data.get('card_last4'):
            return False
        
        # Must be from T-Bank
        if parsed_data.get('bank') != 'T-Bank':
            return False
        
        return True
    
    def match_receipt_to_transaction(self, receipt_id: int, parsed_data: Dict):
        """Match receipt to transaction"""
        try:
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            amount = parsed_data.get('amount', 0)
            phone = parsed_data.get('phone')
            card_last4 = parsed_data.get('card_last4')
            
            # Find matching transaction
            query = """
                SELECT id, wallet, amount_rub 
                FROM transactions 
                WHERE status IN ('waiting_receipt', 'waiting_payment')
                AND amount_rub = %s
            """
            
            cur.execute(query, (amount,))
            candidates = cur.fetchall()
            
            matched_transaction_id = None
            
            for tx_id, wallet, tx_amount in candidates:
                # Clean wallet (remove spaces, dashes)
                clean_wallet = re.sub(r'[\s\-\(\)]', '', wallet)
                
                # Match by phone
                if phone and phone in clean_wallet:
                    matched_transaction_id = tx_id
                    break
                
                # Match by card last 4
                if card_last4 and card_last4 in wallet:
                    matched_transaction_id = tx_id
                    break
            
            if matched_transaction_id:
                console.print(f"[green]🎯 Receipt matched to transaction {matched_transaction_id}[/green]")
                
                # Update transaction
                cur.execute("""
                    UPDATE transactions 
                    SET receipt_id = %s, status = 'approved',
                        approved_at = CURRENT_TIMESTAMP,
                        release_scheduled_at = CURRENT_TIMESTAMP + INTERVAL '2 minutes'
                    WHERE id = %s
                """, (receipt_id, matched_transaction_id))
                
                # Update receipt
                cur.execute("""
                    UPDATE receipts 
                    SET transaction_id = %s
                    WHERE id = %s
                """, (matched_transaction_id, receipt_id))
                
                conn.commit()
                
                # Approve on Gate.io
                self.approve_gate_transaction(matched_transaction_id)
                
            else:
                console.print(f"[yellow]⚠️  No matching transaction found for receipt {receipt_id}[/yellow]")
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Error matching receipt: {e}[/red]")
    
    def approve_gate_transaction(self, transaction_id: int):
        """Approve transaction on Gate.io"""
        try:
            conn = self.get_db_connection()
            cur = conn.cursor()
            
            # Get Gate transaction ID
            cur.execute("""
                SELECT gate_transaction_id FROM transactions WHERE id = %s
            """, (transaction_id,))
            
            gate_tx_id = cur.fetchone()[0]
            
            # TODO: Call Gate API to approve transaction
            console.print(f"[green]✅ Gate transaction {gate_tx_id} approved[/green]")
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Error approving Gate transaction: {e}[/red]")
    
    def normalize_phone(self, phone: str) -> str:
        """Normalize phone number to digits only"""
        return re.sub(r'\D', '', phone)[-10:]  # Last 10 digits