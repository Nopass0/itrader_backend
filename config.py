"""
Configuration Management
Loads and manages configuration from TOML files
"""

import os
import toml
from dataclasses import dataclass
from typing import List, Optional
from pathlib import Path


@dataclass
class Config:
    """Application configuration"""
    
    # Gate.io settings
    gate_poll_interval: int = 15  # seconds
    
    # Bybit settings
    bybit_api_key: str = ""
    bybit_api_secret: str = ""
    bybit_testnet: bool = False
    
    # Payment settings
    payment_bank: str = "Тинькофф"
    payment_phone: str = "+7 XXX XXX-XX-XX"
    payment_method_ids: List[str] = None  # ["75", "382"]  # Тинькофф, СБП
    receipt_email: str = "your-receipt@email.com"
    
    # Email settings
    email_monitoring_enabled: bool = True
    email_imap_server: str = "imap.gmail.com"
    email_imap_port: int = 993
    email_username: str = ""
    email_password: str = ""
    email_check_interval: int = 30  # seconds
    required_receipt_sender: str = "noreply@tinkoff.ru"
    
    # Trading settings
    profit_margin_percent: float = 2.0  # %
    min_order_amount: float = 1000.0  # RUB
    max_order_amount: float = 50000.0  # RUB
    ad_remarks: str = "Быстрая сделка. Отправьте чек на email после оплаты."
    
    # Database settings
    database_path: str = "trader.db"
    
    # Logging settings
    log_level: str = "INFO"
    log_file: str = "trader.log"
    
    def __post_init__(self):
        """Post-initialization setup"""
        if self.payment_method_ids is None:
            self.payment_method_ids = ["75", "382"]  # Default: Тинькофф, СБП
    
    @classmethod
    def load(cls, config_path: str = "config.toml") -> 'Config':
        """Load configuration from TOML file"""
        config_file = Path(config_path)
        
        # Try multiple locations
        if not config_file.exists():
            # Try in config directory
            config_file = Path("config") / config_path
            
        if not config_file.exists():
            # Try default config
            config_file = Path("config/default.toml")
        
        if not config_file.exists():
            raise FileNotFoundError(f"Configuration file not found: {config_path}")
        
        # Load TOML
        with open(config_file, 'r') as f:
            data = toml.load(f)
        
        # Load environment variables (override TOML)
        config_data = {}
        
        # Bybit settings
        config_data['bybit_api_key'] = os.getenv('BYBIT_API_KEY', data.get('bybit', {}).get('api_key', ''))
        config_data['bybit_api_secret'] = os.getenv('BYBIT_API_SECRET', data.get('bybit', {}).get('api_secret', ''))
        config_data['bybit_testnet'] = os.getenv('BYBIT_TESTNET', str(data.get('bybit', {}).get('testnet', False))).lower() == 'true'
        
        # Payment settings
        payment = data.get('payment', {})
        config_data['payment_bank'] = os.getenv('PAYMENT_BANK', payment.get('bank', 'Тинькофф'))
        config_data['payment_phone'] = os.getenv('PAYMENT_PHONE', payment.get('phone', '+7 XXX XXX-XX-XX'))
        config_data['payment_method_ids'] = payment.get('method_ids', ["75", "382"])
        config_data['receipt_email'] = os.getenv('RECEIPT_EMAIL', payment.get('receipt_email', 'your-receipt@email.com'))
        
        # Email settings
        email = data.get('email', {})
        config_data['email_monitoring_enabled'] = email.get('monitoring_enabled', True)
        config_data['email_imap_server'] = os.getenv('EMAIL_IMAP_SERVER', email.get('imap_server', 'imap.gmail.com'))
        config_data['email_imap_port'] = int(os.getenv('EMAIL_IMAP_PORT', email.get('imap_port', 993)))
        config_data['email_username'] = os.getenv('EMAIL_USERNAME', email.get('username', ''))
        config_data['email_password'] = os.getenv('EMAIL_PASSWORD', email.get('password', ''))
        config_data['email_check_interval'] = email.get('check_interval', 30)
        config_data['required_receipt_sender'] = email.get('required_receipt_sender', 'noreply@tinkoff.ru')
        
        # Trading settings
        trading = data.get('trading', {})
        config_data['profit_margin_percent'] = float(trading.get('profit_margin_percent', 2.0))
        config_data['min_order_amount'] = float(trading.get('min_order_amount', 1000.0))
        config_data['max_order_amount'] = float(trading.get('max_order_amount', 50000.0))
        config_data['ad_remarks'] = trading.get('ad_remarks', 'Быстрая сделка. Отправьте чек на email после оплаты.')
        
        # Gate settings
        gate = data.get('gate', {})
        config_data['gate_poll_interval'] = gate.get('poll_interval', 15)
        
        # Database settings
        config_data['database_path'] = data.get('database', {}).get('path', 'trader.db')
        
        # Logging settings
        logging = data.get('logging', {})
        config_data['log_level'] = logging.get('level', 'INFO')
        config_data['log_file'] = logging.get('file', 'trader.log')
        
        return cls(**config_data)
    
    def save_example(self, path: str = ".env.example"):
        """Save example environment file"""
        example = """# Bybit API Configuration
BYBIT_API_KEY=your_api_key_here
BYBIT_API_SECRET=your_api_secret_here
BYBIT_TESTNET=false

# Payment Configuration
PAYMENT_BANK=Тинькофф
PAYMENT_PHONE=+7 XXX XXX-XX-XX
RECEIPT_EMAIL=your-receipt@email.com

# Email Configuration (for receipt monitoring)
EMAIL_USERNAME=your-email@gmail.com
EMAIL_PASSWORD=your-app-password
EMAIL_IMAP_SERVER=imap.gmail.com
EMAIL_IMAP_PORT=993
"""
        with open(path, 'w') as f:
            f.write(example)