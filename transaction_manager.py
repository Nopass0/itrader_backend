"""
Transaction Manager for Virtual Trading System
Manages transaction lifecycle and OCR validation
"""

import logging
import asyncio
import uuid
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from enum import Enum
import re

logger = logging.getLogger(__name__)


class TransactionStatus(Enum):
    """Transaction status states"""
    PENDING = "pending"
    WAITING_PAYMENT = "waiting_payment"
    PAYMENT_RECEIVED = "payment_received"
    VALIDATING = "validating"
    APPROVED = "approved"
    REJECTED = "rejected"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


class Transaction:
    """Single transaction object"""
    
    def __init__(self, transaction_id: Optional[str] = None):
        self.id = transaction_id or str(uuid.uuid4())
        self.platform = "gate"  # or "bybit"
        self.account_id = None
        self.buyer_id = None
        self.seller_id = None
        self.amount = 0.0
        self.currency = "RUB"
        self.crypto = "USDT"
        self.price = 0.0
        self.status = TransactionStatus.PENDING
        self.created_at = datetime.now()
        self.updated_at = datetime.now()
        
        # Payment details
        self.payment_method = None  # SBP or Tinkoff
        self.payment_phone = None
        self.payment_card = None
        self.payment_bank = None
        self.payment_details_sent = False
        
        # Receipt/Check details
        self.receipt_received = False
        self.receipt_filename = None
        self.receipt_validated = False
        self.receipt_validation_result = {}
        
        # Buyer info
        self.buyer_status = None  # fool, verified, etc.
        self.buyer_messages = []
        
        # Additional metadata
        self.metadata = {}
        
    def to_dict(self) -> Dict[str, Any]:
        """Convert transaction to dictionary"""
        return {
            "id": self.id,
            "platform": self.platform,
            "account_id": self.account_id,
            "buyer_id": self.buyer_id,
            "seller_id": self.seller_id,
            "amount": self.amount,
            "currency": self.currency,
            "crypto": self.crypto,
            "price": self.price,
            "status": self.status.value,
            "created_at": self.created_at.isoformat(),
            "updated_at": self.updated_at.isoformat(),
            "payment_method": self.payment_method,
            "payment_phone": self.payment_phone,
            "payment_card": self.payment_card,
            "payment_bank": self.payment_bank,
            "payment_details_sent": self.payment_details_sent,
            "receipt_received": self.receipt_received,
            "receipt_filename": self.receipt_filename,
            "receipt_validated": self.receipt_validated,
            "receipt_validation_result": self.receipt_validation_result,
            "buyer_status": self.buyer_status,
            "buyer_messages": self.buyer_messages,
            "metadata": self.metadata
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Transaction':
        """Create transaction from dictionary"""
        tx = cls(data.get("id"))
        tx.platform = data.get("platform", "gate")
        tx.account_id = data.get("account_id")
        tx.buyer_id = data.get("buyer_id")
        tx.seller_id = data.get("seller_id")
        tx.amount = float(data.get("amount", 0))
        tx.currency = data.get("currency", "RUB")
        tx.crypto = data.get("crypto", "USDT")
        tx.price = float(data.get("price", 0))
        tx.status = TransactionStatus(data.get("status", "pending"))
        tx.created_at = datetime.fromisoformat(data.get("created_at", datetime.now().isoformat()))
        tx.updated_at = datetime.fromisoformat(data.get("updated_at", datetime.now().isoformat()))
        
        tx.payment_method = data.get("payment_method")
        tx.payment_phone = data.get("payment_phone")
        tx.payment_card = data.get("payment_card")
        tx.payment_bank = data.get("payment_bank")
        tx.payment_details_sent = data.get("payment_details_sent", False)
        
        tx.receipt_received = data.get("receipt_received", False)
        tx.receipt_filename = data.get("receipt_filename")
        tx.receipt_validated = data.get("receipt_validated", False)
        tx.receipt_validation_result = data.get("receipt_validation_result", {})
        
        tx.buyer_status = data.get("buyer_status")
        tx.buyer_messages = data.get("buyer_messages", [])
        tx.metadata = data.get("metadata", {})
        
        return tx


class TransactionManager:
    """Manages all transactions"""
    
    def __init__(self, db_manager):
        self.db = db_manager
        self.transactions: Dict[str, Transaction] = {}
        self._load_transactions()
        
    def _load_transactions(self):
        """Load transactions from database"""
        virtual_transactions = self.db.load_virtual_transactions()
        
        for tx_id, tx_data in virtual_transactions.items():
            try:
                transaction = Transaction.from_dict(tx_data)
                self.transactions[tx_id] = transaction
            except Exception as e:
                logger.error(f"Failed to load transaction {tx_id}: {e}")
    
    def create_transaction(self, platform: str, account_id: str, 
                         buyer_id: str, amount: float, price: float,
                         currency: str = "RUB", crypto: str = "USDT") -> Transaction:
        """Create new transaction"""
        transaction = Transaction()
        transaction.platform = platform
        transaction.account_id = account_id
        transaction.buyer_id = buyer_id
        transaction.amount = amount
        transaction.price = price
        transaction.currency = currency
        transaction.crypto = crypto
        
        # Store in memory and database
        self.transactions[transaction.id] = transaction
        self.db.save_transaction(transaction.id, transaction.to_dict())
        
        logger.info(f"Created transaction: {transaction.id}")
        return transaction
    
    async def get_transaction(self, transaction_id: str) -> Optional[Transaction]:
        """Get transaction by ID"""
        if transaction_id in self.transactions:
            return self.transactions[transaction_id]
            
        # Try to load from database
        tx_data = self.db.load_transaction(transaction_id)
        if tx_data:
            transaction = Transaction.from_dict(tx_data)
            self.transactions[transaction_id] = transaction
            return transaction
            
        return None
    
    async def update_transaction(self, transaction_id: str, updates: Dict[str, Any]) -> bool:
        """Update transaction"""
        transaction = await self.get_transaction(transaction_id)
        if not transaction:
            return False
            
        # Apply updates
        for key, value in updates.items():
            if hasattr(transaction, key):
                setattr(transaction, key, value)
                
        transaction.updated_at = datetime.now()
        
        # Save to database
        self.db.save_transaction(transaction_id, transaction.to_dict())
        
        return True
    
    def update_transaction_status(self, transaction_id: str, status: TransactionStatus) -> bool:
        """Update transaction status"""
        transaction = self.transactions.get(transaction_id)
        if not transaction:
            return False
            
        transaction.status = status
        transaction.updated_at = datetime.now()
        
        # Save to database
        self.db.save_transaction(transaction_id, transaction.to_dict())
        
        logger.info(f"Updated transaction {transaction_id} status to: {status.value}")
        return True
    
    def set_payment_details(self, transaction_id: str, payment_method: str,
                          phone: str, card: Optional[str] = None, bank: Optional[str] = None) -> bool:
        """Set payment details for transaction"""
        transaction = self.transactions.get(transaction_id)
        if not transaction:
            return False
            
        transaction.payment_method = payment_method
        transaction.payment_phone = phone
        transaction.payment_card = card
        transaction.payment_bank = bank
        transaction.payment_details_sent = True
        transaction.status = TransactionStatus.WAITING_PAYMENT
        transaction.updated_at = datetime.now()
        
        # Save to database
        self.db.save_transaction(transaction_id, transaction.to_dict())
        
        logger.info(f"Set payment details for transaction: {transaction_id}")
        return True
    
    async def process_receipt(self, transaction_id: str, receipt_filename: str,
                            pdf_data: bytes, metadata: Dict[str, Any]) -> Dict[str, Any]:
        """Process receipt for transaction"""
        transaction = await self.get_transaction(transaction_id)
        if not transaction:
            return {"success": False, "error": "Transaction not found"}
            
        # Save receipt
        try:
            receipt_path = self.db.save_check(
                f"{transaction_id}_{datetime.now().timestamp()}",
                pdf_data,
                {
                    "transaction_id": transaction_id,
                    "account_id": transaction.account_id,
                    "filename": receipt_filename,
                    **metadata
                }
            )
            
            transaction.receipt_received = True
            transaction.receipt_filename = receipt_filename
            transaction.status = TransactionStatus.VALIDATING
            transaction.updated_at = datetime.now()
            
            # Save transaction
            self.db.save_transaction(transaction_id, transaction.to_dict())
            
            logger.info(f"Received receipt for transaction: {transaction_id}")
            return {"success": True, "receipt_path": receipt_path}
            
        except Exception as e:
            logger.error(f"Failed to process receipt for transaction {transaction_id}: {e}")
            return {"success": False, "error": str(e)}
    
    def validate_receipt(self, transaction_id: str, ocr_result: Dict[str, Any]) -> Dict[str, Any]:
        """Validate receipt OCR results against transaction"""
        transaction = self.transactions.get(transaction_id)
        if not transaction:
            return {"valid": False, "error": "Transaction not found"}
            
        validation_result = {
            "valid": True,
            "checks": {},
            "errors": []
        }
        
        # Extract phone and card from OCR result
        ocr_phone = self._extract_phone(ocr_result.get("text", ""))
        ocr_card = self._extract_card(ocr_result.get("text", ""))
        ocr_amount = self._extract_amount(ocr_result.get("text", ""))
        
        # Check phone number
        if transaction.payment_phone:
            phone_match = self._compare_phones(transaction.payment_phone, ocr_phone)
            validation_result["checks"]["phone"] = {
                "expected": transaction.payment_phone,
                "found": ocr_phone,
                "match": phone_match
            }
            if not phone_match:
                validation_result["valid"] = False
                validation_result["errors"].append("Phone number mismatch")
        
        # Check card number (if applicable)
        if transaction.payment_card:
            card_match = self._compare_cards(transaction.payment_card, ocr_card)
            validation_result["checks"]["card"] = {
                "expected": transaction.payment_card,
                "found": ocr_card,
                "match": card_match
            }
            if not card_match:
                validation_result["valid"] = False
                validation_result["errors"].append("Card number mismatch")
        
        # Check amount
        if transaction.amount:
            amount_match = self._compare_amounts(transaction.amount, ocr_amount)
            validation_result["checks"]["amount"] = {
                "expected": transaction.amount,
                "found": ocr_amount,
                "match": amount_match
            }
            if not amount_match:
                validation_result["valid"] = False
                validation_result["errors"].append("Amount mismatch")
        
        # Update transaction
        transaction.receipt_validated = True
        transaction.receipt_validation_result = validation_result
        
        if validation_result["valid"]:
            transaction.status = TransactionStatus.APPROVED
        else:
            transaction.status = TransactionStatus.REJECTED
            
        transaction.updated_at = datetime.now()
        
        # Save to database
        self.db.save_transaction(transaction_id, transaction.to_dict())
        
        logger.info(f"Validated receipt for transaction {transaction_id}: {validation_result['valid']}")
        return validation_result
    
    def _extract_phone(self, text: str) -> Optional[str]:
        """Extract phone number from text"""
        # Look for Russian phone patterns
        patterns = [
            r'\+7\s*\d{3}\s*\d{3}[\s-]?\d{2}[\s-]?\d{2}',
            r'8\s*\d{3}\s*\d{3}[\s-]?\d{2}[\s-]?\d{2}',
            r'\d{3}\s*\d{3}[\s-]?\d{2}[\s-]?\d{2}'
        ]
        
        for pattern in patterns:
            match = re.search(pattern, text)
            if match:
                # Normalize phone number
                phone = re.sub(r'[\s-]', '', match.group())
                return phone
                
        return None
    
    def _extract_card(self, text: str) -> Optional[str]:
        """Extract card number from text"""
        # Look for card number patterns (last 4 digits usually)
        patterns = [
            r'\*{4}\s*\d{4}',
            r'\d{4}\s*\*{4}\s*\*{4}\s*\d{4}',
            r'карта.*?(\d{4})'
        ]
        
        for pattern in patterns:
            match = re.search(pattern, text, re.IGNORECASE)
            if match:
                # Extract last 4 digits
                digits = re.findall(r'\d{4}', match.group())
                if digits:
                    return digits[-1]
                    
        return None
    
    def _extract_amount(self, text: str) -> Optional[float]:
        """Extract amount from text"""
        # Look for amount patterns
        patterns = [
            r'сумма[:\s]*([0-9\s,\.]+)\s*(?:руб|rub|₽)',
            r'итого[:\s]*([0-9\s,\.]+)\s*(?:руб|rub|₽)',
            r'([0-9\s,\.]+)\s*(?:руб|rub|₽)'
        ]
        
        for pattern in patterns:
            matches = re.findall(pattern, text, re.IGNORECASE)
            if matches:
                # Take the largest amount found
                amounts = []
                for match in matches:
                    # Clean and convert to float
                    clean = match.replace(' ', '').replace(',', '.')
                    try:
                        amount = float(clean)
                        amounts.append(amount)
                    except:
                        pass
                        
                if amounts:
                    return max(amounts)
                    
        return None
    
    def _compare_phones(self, expected: str, found: Optional[str]) -> bool:
        """Compare phone numbers"""
        if not found:
            return False
            
        # Normalize both numbers
        expected_norm = re.sub(r'[\s\-\+]', '', expected)
        found_norm = re.sub(r'[\s\-\+]', '', found)
        
        # Remove country code if present
        if expected_norm.startswith('7'):
            expected_norm = expected_norm[1:]
        if expected_norm.startswith('8'):
            expected_norm = expected_norm[1:]
        if found_norm.startswith('7'):
            found_norm = found_norm[1:]
        if found_norm.startswith('8'):
            found_norm = found_norm[1:]
            
        return expected_norm == found_norm
    
    def _compare_cards(self, expected: str, found: Optional[str]) -> bool:
        """Compare card numbers (usually last 4 digits)"""
        if not found:
            return False
            
        # Get last 4 digits
        expected_last4 = re.sub(r'\D', '', expected)[-4:]
        found_last4 = re.sub(r'\D', '', found)[-4:]
        
        return expected_last4 == found_last4
    
    def _compare_amounts(self, expected: float, found: Optional[float]) -> bool:
        """Compare amounts with tolerance"""
        if not found:
            return False
            
        # Allow 1% tolerance for rounding
        tolerance = expected * 0.01
        return abs(expected - found) <= tolerance
    
    def get_pending_transactions(self, platform: Optional[str] = None) -> List[Transaction]:
        """Get all pending transactions"""
        pending = []
        
        for transaction in self.transactions.values():
            if transaction.status in [TransactionStatus.PENDING, TransactionStatus.WAITING_PAYMENT]:
                if platform is None or transaction.platform == platform:
                    pending.append(transaction)
                    
        return sorted(pending, key=lambda x: x.created_at)
    
    def get_transactions_by_status(self, status: TransactionStatus) -> List[Transaction]:
        """Get transactions by status"""
        return [
            tx for tx in self.transactions.values()
            if tx.status == status
        ]
    
    def get_account_transactions(self, account_id: str) -> List[Transaction]:
        """Get all transactions for an account"""
        return [
            tx for tx in self.transactions.values()
            if tx.account_id == account_id
        ]
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get transaction statistics"""
        stats = {
            "total": len(self.transactions),
            "by_status": {},
            "by_platform": {},
            "by_payment_method": {},
            "total_amount": 0,
            "approved_amount": 0,
            "rejected_count": 0,
            "fool_count": 0
        }
        
        for transaction in self.transactions.values():
            # Count by status
            status = transaction.status.value
            stats["by_status"][status] = stats["by_status"].get(status, 0) + 1
            
            # Count by platform
            platform = transaction.platform
            stats["by_platform"][platform] = stats["by_platform"].get(platform, 0) + 1
            
            # Count by payment method
            if transaction.payment_method:
                pm = transaction.payment_method
                stats["by_payment_method"][pm] = stats["by_payment_method"].get(pm, 0) + 1
            
            # Sum amounts
            stats["total_amount"] += transaction.amount
            
            if transaction.status == TransactionStatus.APPROVED:
                stats["approved_amount"] += transaction.amount
            elif transaction.status == TransactionStatus.REJECTED:
                stats["rejected_count"] += 1
                
            if transaction.buyer_status == "fool":
                stats["fool_count"] += 1
                
        return stats
    
    def cleanup_old_transactions(self, days: int = 30):
        """Clean up old completed transactions"""
        cutoff_time = datetime.now() - timedelta(days=days)
        transactions_to_remove = []
        
        for tx_id, transaction in self.transactions.items():
            if transaction.status in [TransactionStatus.COMPLETED, TransactionStatus.CANCELLED]:
                if transaction.updated_at < cutoff_time:
                    transactions_to_remove.append(tx_id)
                    
        for tx_id in transactions_to_remove:
            del self.transactions[tx_id]
            logger.info(f"Cleaned up old transaction: {tx_id}")
            
        return len(transactions_to_remove)