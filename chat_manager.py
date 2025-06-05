"""
Chat Manager for Bybit P2P
Handles automated chat flows with buyers
"""

import asyncio
import logging
from datetime import datetime, timezone
from typing import Dict, Any, Optional, Set
from colorama import Fore

from models import Advertisement, Order
from utils import colored_print

logger = logging.getLogger(__name__)


class ChatManager:
    """Manages automated chat flows for P2P orders"""
    
    # Chat messages in Russian
    GREETING_MESSAGE = "Добрый день! Вы прочитали условия объявления и правила P2P?"
    AGREEMENT_KEYWORDS = ['да', 'yes', 'согласен', 'согласна', 'прочитал', 'прочитала', 'ознакомлен', 'ознакомлена', 'понял', 'поняла', 'ок', 'ok']
    
    PAYMENT_TEMPLATE = """✅ Спасибо за подтверждение!

💳 Реквизиты для оплаты:
Банк: {bank}
Телефон: {phone}
Сумма: {amount} {currency}

📧 ВАЖНО: После оплаты обязательно отправьте чек на email: {email}

⚠️ Чек должен прийти с адреса noreply@tinkoff.ru
⏰ Время на оплату: 15 минут

После получения и проверки чека я подтвержу получение платежа."""

    COMPLETION_MESSAGE = """✅ Платеж получен и подтвержден!
Спасибо за сделку! 
Криптовалюта отправлена на ваш кошелек."""

    REMINDER_MESSAGE = """⏰ Напоминаю:
- Отправьте чек на email после оплаты
- Чек должен прийти с адреса noreply@tinkoff.ru
- Осталось времени: {minutes} минут"""
    
    def __init__(self, bybit_client):
        self.bybit_client = bybit_client
        self.active_chats: Dict[str, Dict[str, Any]] = {}
        self.processed_messages: Set[str] = set()
        self.orders_with_shown_payment: Set[str] = set()
        
    async def start_chat_flow(self, advertisement: Advertisement):
        """Start monitoring chat for new orders on this advertisement"""
        logger.info(f"Starting chat flow for advertisement {advertisement.id}")
        
        # Monitor for new orders
        asyncio.create_task(self._monitor_ad_orders(advertisement))
    
    async def _monitor_ad_orders(self, advertisement: Advertisement):
        """Monitor advertisement for new orders"""
        processed_orders = set()
        
        while True:
            try:
                # Get orders for this ad
                orders = await self.bybit_client.get_ad_orders(advertisement.id)
                
                for order_data in orders:
                    order = Order.from_dict(order_data)
                    
                    if order.id not in processed_orders:
                        processed_orders.add(order.id)
                        colored_print(f"🆕 New order {order.id} for ad {advertisement.id}", Fore.YELLOW)
                        
                        # Send greeting message
                        await self.send_greeting(order)
                        
                        # Start monitoring this order's chat
                        asyncio.create_task(self._monitor_order_chat(order))
                
                await asyncio.sleep(10)  # Check every 10 seconds
                
            except Exception as e:
                logger.error(f"Error monitoring ad orders: {e}")
                await asyncio.sleep(30)
    
    async def send_greeting(self, order: Order) -> bool:
        """Send initial greeting message"""
        try:
            success = await self.bybit_client.send_chat_message(order.id, self.GREETING_MESSAGE)
            if success:
                colored_print(f"📤 Sent greeting to order {order.id}", Fore.CYAN)
                self.active_chats[order.id] = {
                    'order': order,
                    'greeted': True,
                    'agreed': False,
                    'payment_shown': False,
                    'started_at': datetime.now(timezone.utc)
                }
            return success
        except Exception as e:
            logger.error(f"Error sending greeting: {e}")
            return False
    
    async def _monitor_order_chat(self, order: Order):
        """Monitor chat messages for a specific order"""
        while order.id in self.active_chats:
            try:
                # Get chat messages
                messages = await self.bybit_client.get_chat_messages(order.id)
                
                for msg in messages:
                    msg_id = msg.get('id')
                    
                    # Skip if already processed
                    if msg_id in self.processed_messages:
                        continue
                    
                    self.processed_messages.add(msg_id)
                    
                    # Skip our own messages
                    if msg.get('userId') == self.bybit_client.account_info.get('id'):
                        continue
                    
                    # Process buyer's message
                    await self._process_buyer_message(order, msg)
                
                # Send reminders if needed
                await self._check_send_reminder(order)
                
                await asyncio.sleep(5)  # Check every 5 seconds
                
            except Exception as e:
                logger.error(f"Error monitoring order chat: {e}")
                await asyncio.sleep(10)
    
    async def _process_buyer_message(self, order: Order, message: Dict[str, Any]):
        """Process a message from the buyer"""
        content = message.get('content', '').lower().strip()
        chat_state = self.active_chats.get(order.id, {})
        
        colored_print(f"💬 Buyer message in order {order.id}: {message.get('content')}", Fore.BLUE)
        
        # Check if buyer agreed to terms
        if not chat_state.get('agreed'):
            if any(keyword in content for keyword in self.AGREEMENT_KEYWORDS):
                chat_state['agreed'] = True
                colored_print(f"✅ Buyer agreed to terms in order {order.id}", Fore.GREEN)
                
                # Send payment details
                await self.send_payment_details(order)
    
    async def check_buyer_agreement(self, order: Order) -> bool:
        """Check if buyer has agreed to terms"""
        chat_state = self.active_chats.get(order.id, {})
        return chat_state.get('agreed', False)
    
    async def send_payment_details(self, order: Order) -> bool:
        """Send payment details to buyer"""
        if order.id in self.orders_with_shown_payment:
            return True  # Already sent
        
        try:
            # Get order details for amount
            order_details = await self.bybit_client.get_order_details(order.id)
            if not order_details:
                logger.error(f"Failed to get order details for {order.id}")
                return False
            
            amount = order_details.get('amount', '0')
            currency = order_details.get('currencyId', 'RUB')
            
            # Format payment message
            message = self.PAYMENT_TEMPLATE.format(
                bank=self.bybit_client.config.payment_bank,
                phone=self.bybit_client.config.payment_phone,
                amount=amount,
                currency=currency,
                email=self.bybit_client.config.receipt_email
            )
            
            success = await self.bybit_client.send_chat_message(order.id, message)
            if success:
                colored_print(f"💳 Sent payment details for order {order.id}", Fore.GREEN)
                self.orders_with_shown_payment.add(order.id)
                
                chat_state = self.active_chats.get(order.id, {})
                chat_state['payment_shown'] = True
                chat_state['payment_shown_at'] = datetime.now(timezone.utc)
                
            return success
            
        except Exception as e:
            logger.error(f"Error sending payment details: {e}")
            return False
    
    async def _check_send_reminder(self, order: Order):
        """Check if we need to send a payment reminder"""
        chat_state = self.active_chats.get(order.id, {})
        
        if not chat_state.get('payment_shown'):
            return
        
        payment_shown_at = chat_state.get('payment_shown_at')
        if not payment_shown_at:
            return
        
        # Send reminder after 5 and 10 minutes
        elapsed = (datetime.now(timezone.utc) - payment_shown_at).total_seconds() / 60
        
        if elapsed >= 5 and not chat_state.get('reminder_5min'):
            remaining = 15 - int(elapsed)
            message = self.REMINDER_MESSAGE.format(minutes=remaining)
            
            if await self.bybit_client.send_chat_message(order.id, message):
                chat_state['reminder_5min'] = True
                colored_print(f"⏰ Sent 5-minute reminder for order {order.id}", Fore.YELLOW)
        
        elif elapsed >= 10 and not chat_state.get('reminder_10min'):
            remaining = 15 - int(elapsed)
            message = self.REMINDER_MESSAGE.format(minutes=remaining)
            
            if await self.bybit_client.send_chat_message(order.id, message):
                chat_state['reminder_10min'] = True
                colored_print(f"⏰ Sent 10-minute reminder for order {order.id}", Fore.YELLOW)
    
    async def send_completion_message(self, order: Order) -> bool:
        """Send transaction completion message"""
        try:
            success = await self.bybit_client.send_chat_message(order.id, self.COMPLETION_MESSAGE)
            if success:
                colored_print(f"🎉 Sent completion message for order {order.id}", Fore.GREEN)
                
                # Remove from active chats
                if order.id in self.active_chats:
                    del self.active_chats[order.id]
                    
            return success
        except Exception as e:
            logger.error(f"Error sending completion message: {e}")
            return False