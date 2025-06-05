"""
Email Monitor for Receipt Processing
Monitors email for payment receipts from noreply@tinkoff.ru
"""

import asyncio
import email
import imaplib
import logging
import re
from datetime import datetime, timezone
from decimal import Decimal
from email.header import decode_header
from typing import List, Dict, Any, Optional
import PyPDF2
from io import BytesIO

from models import ReceiptValidation
from ocr_processor import OCRProcessor
from utils import colored_print
from colorama import Fore

logger = logging.getLogger(__name__)


class EmailMonitor:
    """Monitors email for payment receipts"""
    
    def __init__(self, config):
        self.config = config
        self.imap = None
        self.ocr_processor = OCRProcessor()
        self.processed_emails = set()
        
    async def connect(self):
        """Connect to email server"""
        try:
            # Connect to IMAP server
            self.imap = imaplib.IMAP4_SSL(
                self.config.email_imap_server, 
                self.config.email_imap_port
            )
            
            # Login
            self.imap.login(
                self.config.email_username,
                self.config.email_password
            )
            
            # Select inbox
            self.imap.select('INBOX')
            
            logger.info(f"Connected to email server as {self.config.email_username}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to connect to email server: {e}")
            return False
    
    async def check_receipts(self) -> List[Dict[str, Any]]:
        """Check for new receipts from required sender"""
        if not self.imap:
            await self.connect()
        
        try:
            # Search for emails from required sender
            search_criteria = f'(FROM "{self.config.required_receipt_sender}" UNSEEN)'
            
            # Run search in executor to avoid blocking
            loop = asyncio.get_event_loop()
            result, data = await loop.run_in_executor(None, self.imap.search, None, search_criteria)
            
            if result != 'OK':
                return []
            
            email_ids = data[0].split()
            receipts = []
            
            for email_id in email_ids:
                if email_id in self.processed_emails:
                    continue
                
                receipt = await self._process_email(email_id)
                if receipt:
                    receipts.append(receipt)
                    self.processed_emails.add(email_id)
            
            return receipts
            
        except Exception as e:
            logger.error(f"Error checking receipts: {e}")
            return []
    
    async def _process_email(self, email_id: bytes) -> Optional[Dict[str, Any]]:
        """Process a single email"""
        try:
            loop = asyncio.get_event_loop()
            
            # Fetch email
            result, data = await loop.run_in_executor(None, self.imap.fetch, email_id, '(RFC822)')
            
            if result != 'OK':
                return None
            
            # Parse email
            raw_email = data[0][1]
            msg = email.message_from_bytes(raw_email)
            
            # Extract email info
            subject = self._decode_header(msg['Subject'])
            from_addr = self._decode_header(msg['From'])
            date = msg['Date']
            
            colored_print(f"📧 Processing email: {subject} from {from_addr}", Fore.CYAN)
            
            # Extract attachments
            attachments = []
            for part in msg.walk():
                if part.get_content_disposition() == 'attachment':
                    filename = part.get_filename()
                    if filename:
                        filename = self._decode_header(filename)
                        content = part.get_payload(decode=True)
                        
                        attachments.append({
                            'filename': filename,
                            'content': content
                        })
            
            return {
                'id': email_id.decode(),
                'subject': subject,
                'from': from_addr,
                'date': date,
                'attachments': attachments
            }
            
        except Exception as e:
            logger.error(f"Error processing email: {e}")
            return None
    
    def _decode_header(self, header_value: str) -> str:
        """Decode email header"""
        if not header_value:
            return ""
        
        decoded = decode_header(header_value)
        result = []
        
        for part, encoding in decoded:
            if isinstance(part, bytes):
                if encoding:
                    result.append(part.decode(encoding))
                else:
                    result.append(part.decode('utf-8', errors='ignore'))
            else:
                result.append(part)
        
        return ' '.join(result)
    
    async def process_pdf_receipt(self, pdf_content: bytes, expected_amount: Optional[Decimal] = None) -> ReceiptValidation:
        """Process PDF receipt and extract data"""
        try:
            # Extract text from PDF
            pdf_file = BytesIO(pdf_content)
            pdf_reader = PyPDF2.PdfReader(pdf_file)
            
            text = ""
            for page in pdf_reader.pages:
                text += page.extract_text() + "\n"
            
            logger.debug(f"Extracted PDF text: {text}")
            
            # Parse receipt data
            validation = self._parse_receipt_text(text, expected_amount)
            
            if validation.is_valid:
                colored_print(
                    f"✅ Valid receipt: {validation.amount} {validation.currency} "
                    f"from {validation.bank} (Ref: {validation.reference})",
                    Fore.GREEN
                )
            
            return validation
            
        except Exception as e:
            logger.error(f"Error processing PDF receipt: {e}")
            return ReceiptValidation(
                is_valid=False,
                error=f"PDF processing error: {str(e)}"
            )
    
    def _parse_receipt_text(self, text: str, expected_amount: Optional[Decimal] = None) -> ReceiptValidation:
        """Parse receipt text and validate"""
        try:
            # Extract amount
            amount = self._extract_amount(text)
            if not amount:
                return ReceiptValidation(is_valid=False, error="Amount not found")
            
            # Extract currency
            currency = self._extract_currency(text)
            
            # Extract bank
            bank = self._extract_bank(text)
            
            # Extract reference
            reference = self._extract_reference(text)
            
            # Validate amount if expected
            if expected_amount and abs(amount - expected_amount) > Decimal('0.01'):
                return ReceiptValidation(
                    is_valid=False,
                    error=f"Amount mismatch: found {amount}, expected {expected_amount}"
                )
            
            return ReceiptValidation(
                is_valid=True,
                amount=amount,
                currency=currency,
                bank=bank,
                reference=reference,
                timestamp=datetime.now(timezone.utc)
            )
            
        except Exception as e:
            logger.error(f"Error parsing receipt text: {e}")
            return ReceiptValidation(
                is_valid=False,
                error=f"Parse error: {str(e)}"
            )
    
    def _extract_amount(self, text: str) -> Optional[Decimal]:
        """Extract amount from receipt text"""
        # Look for amount patterns
        patterns = [
            r'Сумма[:\s]+([0-9\s]+(?:[,.]\d{2})?)\s*(?:₽|руб|RUB)',
            r'Итого[:\s]+([0-9\s]+(?:[,.]\d{2})?)\s*(?:₽|руб|RUB)',
            r'([0-9\s]+(?:[,.]\d{2})?)\s*(?:₽|руб|RUB)',
            r'Amount[:\s]+([0-9\s]+(?:[,.]\d{2})?)',
        ]
        
        for pattern in patterns:
            match = re.search(pattern, text, re.IGNORECASE)
            if match:
                amount_str = match.group(1)
                # Remove spaces and replace comma with dot
                amount_str = amount_str.replace(' ', '').replace(',', '.')
                try:
                    return Decimal(amount_str)
                except:
                    continue
        
        return None
    
    def _extract_currency(self, text: str) -> str:
        """Extract currency from receipt text"""
        if any(x in text for x in ['₽', 'руб', 'RUB', 'рубл']):
            return 'RUB'
        elif any(x in text for x in ['$', 'USD', 'dollar']):
            return 'USD'
        elif any(x in text for x in ['€', 'EUR', 'euro']):
            return 'EUR'
        return 'RUB'  # Default
    
    def _extract_bank(self, text: str) -> str:
        """Extract bank name from receipt text"""
        text_lower = text.lower()
        
        banks = {
            'тинькофф': 'Tinkoff',
            'т-банк': 'T-Bank',
            'tinkoff': 'Tinkoff',
            't-bank': 'T-Bank',
            'сбербанк': 'Sberbank',
            'сбер': 'Sberbank',
            'альфа': 'Alfa-Bank',
            'втб': 'VTB',
        }
        
        for pattern, bank_name in banks.items():
            if pattern in text_lower:
                return bank_name
        
        return 'Unknown'
    
    def _extract_reference(self, text: str) -> str:
        """Extract transaction reference from receipt text"""
        patterns = [
            r'Номер операции[:\s]*([\d\-]+)',
            r'Операция[:\s]*([\d\-]+)',
            r'Transaction[:\s]*([\d\-A-Za-z]+)',
            r'№[:\s]*([\d\-]+)',
            r'Чек[:\s]*([\d\-]+)',
        ]
        
        for pattern in patterns:
            match = re.search(pattern, text, re.IGNORECASE)
            if match:
                return match.group(1).strip()
        
        # Try to find any long number sequence
        numbers = re.findall(r'\d{6,}', text)
        if numbers:
            return numbers[0]
        
        return f"AUTO-{datetime.now().timestamp():.0f}"
    
    async def disconnect(self):
        """Disconnect from email server"""
        if self.imap:
            try:
                self.imap.close()
                self.imap.logout()
            except:
                pass
            self.imap = None