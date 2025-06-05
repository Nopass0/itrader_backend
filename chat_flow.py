"""
Chat Flow Manager for P2P Trading
Handles the updated conversation flow with buyers
"""

import logging
import asyncio
import re
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
from enum import Enum

logger = logging.getLogger(__name__)


class ChatState(Enum):
    """Chat flow states"""
    INITIAL = "initial"
    WAITING_BANK_CONFIRMATION = "waiting_bank_confirmation"
    WAITING_PDF_CONFIRMATION = "waiting_pdf_confirmation"
    WAITING_SBP_CONFIRMATION = "waiting_sbp_confirmation"
    PAYMENT_DETAILS_SENT = "payment_details_sent"
    REJECTED = "rejected"
    COMPLETED = "completed"


class ChatSession:
    """Single chat session with a buyer"""
    
    def __init__(self, transaction_id: str, buyer_id: str, platform: str = "gate"):
        self.transaction_id = transaction_id
        self.buyer_id = buyer_id
        self.platform = platform
        self.state = ChatState.INITIAL
        self.created_at = datetime.now()
        self.updated_at = datetime.now()
        self.messages: List[Dict[str, Any]] = []
        self.rejection_reason: Optional[str] = None
        self.payment_details_sent = False
        self.payment_method: Optional[str] = None  # SBP or Tinkoff
        
    def add_message(self, sender: str, content: str):
        """Add message to history"""
        self.messages.append({
            "sender": sender,
            "content": content,
            "timestamp": datetime.now().isoformat()
        })
        self.updated_at = datetime.now()
        
    def to_dict(self) -> Dict[str, Any]:
        """Convert session to dictionary"""
        return {
            "transaction_id": self.transaction_id,
            "buyer_id": self.buyer_id,
            "platform": self.platform,
            "state": self.state.value,
            "created_at": self.created_at.isoformat(),
            "updated_at": self.updated_at.isoformat(),
            "messages": self.messages,
            "rejection_reason": self.rejection_reason,
            "payment_details_sent": self.payment_details_sent,
            "payment_method": self.payment_method
        }


class ChatFlowManager:
    """Manages chat flow for all transactions"""
    
    # Chat messages
    MESSAGES = {
        "greeting": """Здравствуйте!
Оплата будет с Т банка? 
( просто напишите да/нет)""",
        
        "pdf_check": """Чек в формате пдф с официальной почты Т банка сможете отправить ? 
( просто напишите да/нет)""",
        
        "sbp_warning": """При СБП, если оплата будет на неверный банк, деньги потеряны.
( просто напишите подтверждаю/ не подтверждаю)""",
        
        "rejection": "Извините, мы не можем продолжить сделку.",
        
        "payment_template": """Спасибо за подтверждение!

Детали оплаты:
💳 Способ оплаты: {payment_method}
🏦 Банк получателя: {bank}
📱 Номер телефона: {phone}
💰 Сумма: {amount} RUB

📋 ВАЖНО:
1. Переводите ТОЛЬКО с Т-Банка
2. После оплаты отправьте PDF чек на email: {email}
3. Чек должен быть отправлен с официальной почты Т-Банка

⚠️ При отправке на неверный банк деньги будут потеряны!"""
    }
    
    def __init__(self, db_manager, transaction_manager):
        self.db = db_manager
        self.transaction_manager = transaction_manager
        self.sessions: Dict[str, ChatSession] = {}
        self.payment_alternator = 0  # For alternating payment methods
        
    def create_session(self, transaction_id: str, buyer_id: str, platform: str = "gate") -> ChatSession:
        """Create new chat session"""
        session = ChatSession(transaction_id, buyer_id, platform)
        self.sessions[transaction_id] = session
        
        # Save to database
        self._save_session(session)
        
        logger.info(f"Created chat session for transaction: {transaction_id}")
        return session
        
    def get_session(self, transaction_id: str) -> Optional[ChatSession]:
        """Get existing session"""
        if transaction_id in self.sessions:
            return self.sessions[transaction_id]
            
        # Try to load from database
        session_data = self.db.load_transaction(f"chat_{transaction_id}")
        if session_data:
            session = self._load_session(session_data)
            self.sessions[transaction_id] = session
            return session
            
        return None
        
    def _save_session(self, session: ChatSession):
        """Save session to database"""
        self.db.save_transaction(f"chat_{session.transaction_id}", session.to_dict())
        
    def _load_session(self, data: Dict[str, Any]) -> ChatSession:
        """Load session from dictionary"""
        session = ChatSession(
            data["transaction_id"],
            data["buyer_id"],
            data.get("platform", "gate")
        )
        session.state = ChatState(data["state"])
        session.created_at = datetime.fromisoformat(data["created_at"])
        session.updated_at = datetime.fromisoformat(data["updated_at"])
        session.messages = data.get("messages", [])
        session.rejection_reason = data.get("rejection_reason")
        session.payment_details_sent = data.get("payment_details_sent", False)
        session.payment_method = data.get("payment_method")
        
        return session
    
    async def process_new_buyer(self, transaction_id: str, buyer_id: str, 
                               platform: str = "gate") -> Tuple[str, ChatSession]:
        """Process new buyer - send initial message"""
        session = self.create_session(transaction_id, buyer_id, platform)
        
        # Send greeting
        message = self.MESSAGES["greeting"]
        session.add_message("bot", message)
        session.state = ChatState.WAITING_BANK_CONFIRMATION
        
        self._save_session(session)
        
        return message, session
    
    async def process_buyer_response(self, transaction_id: str, 
                                   message: str) -> Tuple[Optional[str], ChatSession]:
        """Process buyer's response and return next message"""
        session = self.get_session(transaction_id)
        if not session:
            logger.error(f"Session not found for transaction: {transaction_id}")
            return None, None
            
        # Add buyer message to history
        session.add_message("buyer", message)
        
        # Process based on current state
        response = None
        
        if session.state == ChatState.WAITING_BANK_CONFIRMATION:
            response = await self._process_bank_confirmation(session, message)
            
        elif session.state == ChatState.WAITING_PDF_CONFIRMATION:
            response = await self._process_pdf_confirmation(session, message)
            
        elif session.state == ChatState.WAITING_SBP_CONFIRMATION:
            response = await self._process_sbp_confirmation(session, message)
            
        elif session.state == ChatState.PAYMENT_DETAILS_SENT:
            # Already sent payment details, just acknowledge
            logger.info(f"Buyer responded after payment details: {transaction_id}")
            
        # Save updated session
        self._save_session(session)
        
        return response, session
    
    async def _process_bank_confirmation(self, session: ChatSession, message: str) -> Optional[str]:
        """Process bank confirmation response"""
        normalized = self._normalize_response(message)
        
        if normalized in ["да", "yes", "д"]:
            # Move to PDF confirmation
            response = self.MESSAGES["pdf_check"]
            session.add_message("bot", response)
            session.state = ChatState.WAITING_PDF_CONFIRMATION
            return response
            
        elif normalized in ["нет", "no", "н"]:
            # Reject
            session.state = ChatState.REJECTED
            session.rejection_reason = "Not using T-Bank"
            
            # Mark as "Дурак" in transaction
            await self._mark_transaction_fool(session.transaction_id)
            
            response = self.MESSAGES["rejection"]
            session.add_message("bot", response)
            return response
            
        else:
            # Invalid response, ask again
            response = "Пожалуйста, ответьте 'да' или 'нет'"
            session.add_message("bot", response)
            return response
    
    async def _process_pdf_confirmation(self, session: ChatSession, message: str) -> Optional[str]:
        """Process PDF confirmation response"""
        normalized = self._normalize_response(message)
        
        if normalized in ["да", "yes", "д"]:
            # Move to SBP warning
            response = self.MESSAGES["sbp_warning"]
            session.add_message("bot", response)
            session.state = ChatState.WAITING_SBP_CONFIRMATION
            return response
            
        elif normalized in ["нет", "no", "н"]:
            # Reject
            session.state = ChatState.REJECTED
            session.rejection_reason = "Cannot send PDF receipt"
            
            # Mark as "Дурак" in transaction
            await self._mark_transaction_fool(session.transaction_id)
            
            response = self.MESSAGES["rejection"]
            session.add_message("bot", response)
            return response
            
        else:
            # Invalid response, ask again
            response = "Пожалуйста, ответьте 'да' или 'нет'"
            session.add_message("bot", response)
            return response
    
    async def _process_sbp_confirmation(self, session: ChatSession, message: str) -> Optional[str]:
        """Process SBP warning confirmation"""
        normalized = self._normalize_response(message)
        
        if normalized in ["подтверждаю", "confirm", "п"]:
            # Send payment details
            payment_details = await self._get_payment_details(session)
            
            session.add_message("bot", payment_details)
            session.state = ChatState.PAYMENT_DETAILS_SENT
            session.payment_details_sent = True
            
            # Update transaction with payment details
            await self._update_transaction_payment_sent(session.transaction_id, session.payment_method)
            
            return payment_details
            
        elif normalized in ["не подтверждаю", "не", "нет", "cancel"]:
            # Reject
            session.state = ChatState.REJECTED
            session.rejection_reason = "Did not confirm SBP warning"
            
            # Mark as "Дурак" in transaction
            await self._mark_transaction_fool(session.transaction_id)
            
            response = self.MESSAGES["rejection"]
            session.add_message("bot", response)
            return response
            
        else:
            # Invalid response, ask again
            response = "Пожалуйста, ответьте 'подтверждаю' или 'не подтверждаю'"
            session.add_message("bot", response)
            return response
    
    def _normalize_response(self, message: str) -> str:
        """Normalize user response"""
        # Remove extra spaces and convert to lowercase
        normalized = message.strip().lower()
        
        # Remove punctuation
        normalized = re.sub(r'[^\w\s]', '', normalized)
        
        return normalized
    
    async def _get_payment_details(self, session: ChatSession) -> str:
        """Get payment details for transaction"""
        # Get transaction details
        transaction = await self.transaction_manager.get_transaction(session.transaction_id)
        
        if not transaction:
            logger.error(f"Transaction not found: {session.transaction_id}")
            return "Ошибка получения деталей платежа"
        
        # Alternate payment methods
        if self.payment_alternator % 2 == 0:
            session.payment_method = "SBP"
            bank = "Альфа-Банк"  # Example, should be from config
        else:
            session.payment_method = "Tinkoff"
            bank = "Тинькофф"
            
        self.payment_alternator += 1
        
        # Format payment details
        payment_details = self.MESSAGES["payment_template"].format(
            payment_method=session.payment_method,
            bank=bank,
            phone=transaction.get("phone", "+7 XXX XXX-XX-XX"),
            amount=transaction.get("amount", "0"),
            email=transaction.get("email", "receipts@example.com")
        )
        
        return payment_details
    
    async def _mark_transaction_fool(self, transaction_id: str):
        """Mark transaction buyer as fool"""
        transaction = await self.transaction_manager.get_transaction(transaction_id)
        if transaction:
            transaction["buyer_status"] = "fool"
            transaction["rejection_time"] = datetime.now().isoformat()
            await self.transaction_manager.update_transaction(transaction_id, transaction)
            
        logger.info(f"Marked transaction {transaction_id} buyer as fool")
    
    async def _update_transaction_payment_sent(self, transaction_id: str, payment_method: str):
        """Update transaction with payment details sent"""
        transaction = await self.transaction_manager.get_transaction(transaction_id)
        if transaction:
            transaction["payment_details_sent"] = True
            transaction["payment_method"] = payment_method
            transaction["payment_sent_time"] = datetime.now().isoformat()
            await self.transaction_manager.update_transaction(transaction_id, transaction)
            
        logger.info(f"Updated transaction {transaction_id} with payment details sent")
    
    def get_active_sessions(self) -> List[ChatSession]:
        """Get all active chat sessions"""
        active_sessions = []
        
        for session in self.sessions.values():
            if session.state not in [ChatState.REJECTED, ChatState.COMPLETED]:
                active_sessions.append(session)
                
        return active_sessions
    
    def get_session_stats(self) -> Dict[str, Any]:
        """Get chat session statistics"""
        stats = {
            "total_sessions": len(self.sessions),
            "active_sessions": 0,
            "rejected_sessions": 0,
            "completed_sessions": 0,
            "payment_sent": 0,
            "by_state": {}
        }
        
        for session in self.sessions.values():
            if session.state == ChatState.REJECTED:
                stats["rejected_sessions"] += 1
            elif session.state == ChatState.COMPLETED:
                stats["completed_sessions"] += 1
            elif session.state not in [ChatState.REJECTED, ChatState.COMPLETED]:
                stats["active_sessions"] += 1
                
            if session.payment_details_sent:
                stats["payment_sent"] += 1
                
            # Count by state
            state_name = session.state.value
            stats["by_state"][state_name] = stats["by_state"].get(state_name, 0) + 1
            
        return stats
    
    def cleanup_old_sessions(self, days: int = 7):
        """Clean up old completed/rejected sessions"""
        cutoff_time = datetime.now().timestamp() - (days * 24 * 60 * 60)
        sessions_to_remove = []
        
        for transaction_id, session in self.sessions.items():
            if session.state in [ChatState.REJECTED, ChatState.COMPLETED]:
                if session.updated_at.timestamp() < cutoff_time:
                    sessions_to_remove.append(transaction_id)
                    
        for transaction_id in sessions_to_remove:
            del self.sessions[transaction_id]
            logger.info(f"Cleaned up old session: {transaction_id}")
            
        return len(sessions_to_remove)