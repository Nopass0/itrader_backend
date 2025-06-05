"""
Utility Functions
Common utilities used throughout the application
"""

import asyncio
import logging
import time
from datetime import datetime
from typing import Optional, Any
from colorama import Fore, Style, init
import click

# Initialize colorama
init(autoreset=True)


def setup_logging(log_file: str = "trader.log", level: str = "INFO") -> logging.Logger:
    """Setup logging configuration"""
    
    # Create logger
    logger = logging.getLogger()
    logger.setLevel(getattr(logging, level.upper()))
    
    # Remove existing handlers
    logger.handlers = []
    
    # Console handler with color
    console_handler = logging.StreamHandler()
    console_handler.setLevel(getattr(logging, level.upper()))
    
    # File handler
    file_handler = logging.FileHandler(log_file)
    file_handler.setLevel(getattr(logging, level.upper()))
    
    # Formatter
    formatter = logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )
    
    console_handler.setFormatter(formatter)
    file_handler.setFormatter(formatter)
    
    logger.addHandler(console_handler)
    logger.addHandler(file_handler)
    
    return logger


def colored_print(message: str, color=Fore.WHITE, style=Style.NORMAL):
    """Print colored message to console"""
    print(f"{style}{color}{message}{Style.RESET_ALL}")


def confirm_action(prompt: str) -> bool:
    """Ask user for confirmation"""
    return click.confirm(prompt, default=True)


class RateLimiter:
    """Simple rate limiter for API calls"""
    
    def __init__(self, max_requests: int, window_seconds: int):
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self.requests = []
        self.lock = asyncio.Lock()
    
    async def acquire(self):
        """Wait if necessary to respect rate limit"""
        async with self.lock:
            now = time.time()
            
            # Remove old requests outside the window
            self.requests = [t for t in self.requests if now - t < self.window_seconds]
            
            # If at limit, wait
            if len(self.requests) >= self.max_requests:
                sleep_time = self.window_seconds - (now - self.requests[0]) + 0.1
                if sleep_time > 0:
                    await asyncio.sleep(sleep_time)
                    # Recurse to check again
                    return await self.acquire()
            
            # Add this request
            self.requests.append(now)


def format_amount(amount: Any, currency: str = "") -> str:
    """Format amount with currency"""
    try:
        if currency:
            return f"{float(amount):,.2f} {currency}"
        else:
            return f"{float(amount):,.2f}"
    except:
        return str(amount)


def format_datetime(dt: Optional[datetime]) -> str:
    """Format datetime for display"""
    if not dt:
        return "N/A"
    return dt.strftime("%Y-%m-%d %H:%M:%S UTC")


def truncate_string(s: str, max_length: int = 50) -> str:
    """Truncate string with ellipsis"""
    if len(s) <= max_length:
        return s
    return s[:max_length-3] + "..."