"""
Data Models
Defines data structures used throughout the application
"""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from decimal import Decimal
from typing import Optional, Dict, Any, List
from colorama import Fore

from utils import colored_print


@dataclass
class Transaction:
    """Gate.io transaction"""
    id: str
    order_id: str
    amount: Decimal
    currency: str
    fiat_amount: Decimal
    fiat_currency: str
    rate: Decimal
    status: int  # 1=Pending, 2=In Progress, 3=Completed
    buyer_name: str
    payment_method: str
    created_at: datetime
    updated_at: Optional[datetime] = None
    meta: Dict[str, Any] = field(default_factory=dict)
    
    def display(self):
        """Display transaction details"""
        status_map = {1: "Pending", 2: "In Progress", 3: "Completed"}
        status_color = {1: Fore.YELLOW, 2: Fore.CYAN, 3: Fore.GREEN}
        
        colored_print(f"Transaction ID: {self.id}", Fore.WHITE)
        colored_print(f"Amount: {self.amount} {self.currency}", Fore.WHITE)
        colored_print(f"Fiat: {self.fiat_amount} {self.fiat_currency}", Fore.WHITE)
        colored_print(f"Rate: {self.rate}", Fore.WHITE)
        colored_print(f"Status: {status_map.get(self.status, 'Unknown')}", status_color.get(self.status, Fore.WHITE))
        colored_print(f"Buyer: {self.buyer_name}", Fore.WHITE)
        colored_print(f"Payment: {self.payment_method}", Fore.WHITE)
        colored_print(f"Created: {self.created_at.strftime('%Y-%m-%d %H:%M:%S')}", Fore.WHITE)


@dataclass
class Advertisement:
    """Bybit P2P advertisement"""
    id: str
    transaction_id: str
    asset: str
    fiat: str
    price: Decimal
    amount: Decimal
    min_amount: Optional[Decimal] = None
    max_amount: Optional[Decimal] = None
    status: str = "active"  # active, paused, completed
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    payment_methods: List[Dict[str, str]] = field(default_factory=list)
    remarks: str = ""


@dataclass
class Order:
    """Bybit P2P order"""
    id: str
    ad_id: str
    amount: Decimal
    price: Decimal
    currency: str
    fiat_currency: str
    status: str  # 10=Created, 20=Paid, 30=Released
    buyer_id: str
    seller_id: str
    created_at: datetime
    payment_shown: bool = False
    payment_shown_at: Optional[datetime] = None
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Order':
        """Create Order from API response"""
        return cls(
            id=data.get('orderId', data.get('id', '')),
            ad_id=data.get('adId', ''),
            amount=Decimal(str(data.get('amount', '0'))),
            price=Decimal(str(data.get('price', '0'))),
            currency=data.get('tokenId', 'USDT'),
            fiat_currency=data.get('currencyId', 'RUB'),
            status=str(data.get('orderStatus', data.get('status', '10'))),
            buyer_id=data.get('buyerUserId', ''),
            seller_id=data.get('sellerUserId', ''),
            created_at=datetime.fromtimestamp(
                int(data.get('createdAt', 0)) / 1000,
                tz=timezone.utc
            ) if data.get('createdAt') else datetime.now(timezone.utc)
        )


@dataclass
class ReceiptValidation:
    """Receipt validation result"""
    is_valid: bool
    error: Optional[str] = None
    amount: Optional[Decimal] = None
    currency: str = "RUB"
    bank: Optional[str] = None
    reference: Optional[str] = None
    timestamp: Optional[datetime] = None


@dataclass
class ChatMessage:
    """Chat message"""
    id: str
    order_id: str
    user_id: str
    content: str
    timestamp: datetime
    is_buyer: bool