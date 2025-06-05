"""
Database Manager for Multi-Account Trading System
Handles file-based storage for accounts, transactions, and checks
"""

import json
import os
import logging
from datetime import datetime
from typing import Dict, List, Optional, Any
from pathlib import Path
import shutil

logger = logging.getLogger(__name__)


class DatabaseManager:
    """Manages file-based database operations"""
    
    def __init__(self, base_path: str = "db"):
        """Initialize database manager with base path"""
        self.base_path = Path(base_path)
        self._init_directories()
        
    def _init_directories(self):
        """Create necessary directories if they don't exist"""
        directories = [
            self.base_path,
            self.base_path / "gate",
            self.base_path / "bybit",
            self.base_path / "gmail",
            self.base_path / "transactions",
            self.base_path / "checks"
        ]
        
        for directory in directories:
            directory.mkdir(parents=True, exist_ok=True)
            
    def _read_json_file(self, file_path: Path) -> Optional[Dict[str, Any]]:
        """Read JSON file safely"""
        try:
            if file_path.exists():
                with open(file_path, 'r', encoding='utf-8') as f:
                    return json.load(f)
        except Exception as e:
            logger.error(f"Error reading {file_path}: {e}")
        return None
        
    def _write_json_file(self, file_path: Path, data: Dict[str, Any]):
        """Write JSON file safely"""
        try:
            # Write to temp file first
            temp_path = file_path.with_suffix('.tmp')
            with open(temp_path, 'w', encoding='utf-8') as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            
            # Move temp file to actual file
            shutil.move(str(temp_path), str(file_path))
        except Exception as e:
            logger.error(f"Error writing {file_path}: {e}")
            raise
    
    # Gate.io Account Management
    def save_gate_account(self, account_id: str, login: str, password: str, 
                         additional_data: Optional[Dict] = None) -> bool:
        """Save Gate.io account credentials"""
        try:
            account_data = {
                "id": account_id,
                "login": login,
                "password": password,
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat(),
                "status": "active"
            }
            
            if additional_data:
                account_data.update(additional_data)
                
            file_path = self.base_path / "gate" / f"{account_id}.json"
            self._write_json_file(file_path, account_data)
            
            logger.info(f"Saved Gate.io account: {account_id}")
            return True
        except Exception as e:
            logger.error(f"Failed to save Gate.io account {account_id}: {e}")
            return False
    
    def load_gate_account(self, account_id: str) -> Optional[Dict[str, Any]]:
        """Load Gate.io account data"""
        file_path = self.base_path / "gate" / f"{account_id}.json"
        return self._read_json_file(file_path)
    
    def list_gate_accounts(self) -> List[str]:
        """List all Gate.io account IDs"""
        gate_dir = self.base_path / "gate"
        accounts = []
        
        for file_path in gate_dir.glob("*.json"):
            if not file_path.name.endswith("_cookies.json"):
                account_id = file_path.stem
                accounts.append(account_id)
                
        return accounts
    
    def save_gate_cookies(self, account_id: str, cookies: Dict[str, Any]) -> bool:
        """Save Gate.io cookies/session"""
        try:
            cookie_data = {
                "account_id": account_id,
                "cookies": cookies,
                "saved_at": datetime.now().isoformat()
            }
            
            file_path = self.base_path / "gate" / f"{account_id}_cookies.json"
            self._write_json_file(file_path, cookie_data)
            
            logger.info(f"Saved Gate.io cookies for: {account_id}")
            return True
        except Exception as e:
            logger.error(f"Failed to save Gate.io cookies for {account_id}: {e}")
            return False
    
    def load_gate_cookies(self, account_id: str) -> Optional[Dict[str, Any]]:
        """Load Gate.io cookies/session"""
        file_path = self.base_path / "gate" / f"{account_id}_cookies.json"
        cookie_data = self._read_json_file(file_path)
        
        if cookie_data:
            return cookie_data.get("cookies")
        return None
    
    def delete_gate_account(self, account_id: str) -> bool:
        """Delete Gate.io account and associated data"""
        try:
            # Delete account file
            account_file = self.base_path / "gate" / f"{account_id}.json"
            if account_file.exists():
                account_file.unlink()
                
            # Delete cookies file
            cookies_file = self.base_path / "gate" / f"{account_id}_cookies.json"
            if cookies_file.exists():
                cookies_file.unlink()
                
            logger.info(f"Deleted Gate.io account: {account_id}")
            return True
        except Exception as e:
            logger.error(f"Failed to delete Gate.io account {account_id}: {e}")
            return False
    
    # Bybit Account Management
    def save_bybit_account(self, account_id: str, api_key: str, api_secret: str,
                          additional_data: Optional[Dict] = None) -> bool:
        """Save Bybit account API credentials"""
        try:
            account_data = {
                "id": account_id,
                "api_key": api_key,
                "api_secret": api_secret,
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat(),
                "status": "active"
            }
            
            if additional_data:
                account_data.update(additional_data)
                
            file_path = self.base_path / "bybit" / f"{account_id}.json"
            self._write_json_file(file_path, account_data)
            
            logger.info(f"Saved Bybit account: {account_id}")
            return True
        except Exception as e:
            logger.error(f"Failed to save Bybit account {account_id}: {e}")
            return False
    
    def load_bybit_account(self, account_id: str) -> Optional[Dict[str, Any]]:
        """Load Bybit account data"""
        file_path = self.base_path / "bybit" / f"{account_id}.json"
        return self._read_json_file(file_path)
    
    def list_bybit_accounts(self) -> List[str]:
        """List all Bybit account IDs"""
        bybit_dir = self.base_path / "bybit"
        return [f.stem for f in bybit_dir.glob("*.json")]
    
    def delete_bybit_account(self, account_id: str) -> bool:
        """Delete Bybit account"""
        try:
            account_file = self.base_path / "bybit" / f"{account_id}.json"
            if account_file.exists():
                account_file.unlink()
                
            logger.info(f"Deleted Bybit account: {account_id}")
            return True
        except Exception as e:
            logger.error(f"Failed to delete Bybit account {account_id}: {e}")
            return False
    
    # Gmail Account Management
    def save_gmail_credentials(self, credentials: Dict[str, Any]) -> bool:
        """Save Gmail OAuth2 credentials"""
        try:
            file_path = self.base_path / "gmail" / "credentials.json"
            self._write_json_file(file_path, credentials)
            
            logger.info("Saved Gmail credentials")
            return True
        except Exception as e:
            logger.error(f"Failed to save Gmail credentials: {e}")
            return False
    
    def load_gmail_credentials(self) -> Optional[Dict[str, Any]]:
        """Load Gmail OAuth2 credentials"""
        file_path = self.base_path / "gmail" / "credentials.json"
        return self._read_json_file(file_path)
    
    def save_gmail_token(self, token: Dict[str, Any], account_email: Optional[str] = None) -> bool:
        """Save Gmail OAuth2 token"""
        try:
            token_data = {
                "token": token,
                "email": account_email,
                "saved_at": datetime.now().isoformat()
            }
            
            filename = f"token_{account_email}.json" if account_email else "token.json"
            file_path = self.base_path / "gmail" / filename
            self._write_json_file(file_path, token_data)
            
            logger.info(f"Saved Gmail token for: {account_email or 'default'}")
            return True
        except Exception as e:
            logger.error(f"Failed to save Gmail token: {e}")
            return False
    
    def load_gmail_token(self, account_email: Optional[str] = None) -> Optional[Dict[str, Any]]:
        """Load Gmail OAuth2 token"""
        filename = f"token_{account_email}.json" if account_email else "token.json"
        file_path = self.base_path / "gmail" / filename
        token_data = self._read_json_file(file_path)
        
        if token_data:
            return token_data.get("token")
        return None
    
    def list_gmail_accounts(self) -> List[str]:
        """List all Gmail accounts with tokens"""
        gmail_dir = self.base_path / "gmail"
        accounts = []
        
        for file_path in gmail_dir.glob("token_*.json"):
            # Extract email from filename
            email = file_path.stem.replace("token_", "")
            accounts.append(email)
            
        return accounts
    
    # Transaction Management
    def save_transaction(self, transaction_id: str, transaction_data: Dict[str, Any]) -> bool:
        """Save transaction data"""
        try:
            transaction_data["id"] = transaction_id
            transaction_data["updated_at"] = datetime.now().isoformat()
            
            file_path = self.base_path / "transactions" / f"{transaction_id}.json"
            self._write_json_file(file_path, transaction_data)
            
            # Also update virtual transactions file
            self._update_virtual_transactions(transaction_id, transaction_data)
            
            logger.info(f"Saved transaction: {transaction_id}")
            return True
        except Exception as e:
            logger.error(f"Failed to save transaction {transaction_id}: {e}")
            return False
    
    def load_transaction(self, transaction_id: str) -> Optional[Dict[str, Any]]:
        """Load transaction data"""
        file_path = self.base_path / "transactions" / f"{transaction_id}.json"
        return self._read_json_file(file_path)
    
    def list_transactions(self, status: Optional[str] = None) -> List[Dict[str, Any]]:
        """List all transactions, optionally filtered by status"""
        transactions = []
        trans_dir = self.base_path / "transactions"
        
        for file_path in trans_dir.glob("*.json"):
            if file_path.name != "virtual_transactions.json":
                trans_data = self._read_json_file(file_path)
                if trans_data:
                    if status is None or trans_data.get("status") == status:
                        transactions.append(trans_data)
                        
        return sorted(transactions, key=lambda x: x.get("created_at", ""), reverse=True)
    
    def _update_virtual_transactions(self, transaction_id: str, transaction_data: Dict[str, Any]):
        """Update virtual transactions file"""
        vt_file = self.base_path / "transactions" / "virtual_transactions.json"
        vt_data = self._read_json_file(vt_file) or {"transactions": {}}
        
        vt_data["transactions"][transaction_id] = transaction_data
        vt_data["updated_at"] = datetime.now().isoformat()
        
        self._write_json_file(vt_file, vt_data)
    
    def load_virtual_transactions(self) -> Dict[str, Dict[str, Any]]:
        """Load all virtual transactions"""
        vt_file = self.base_path / "transactions" / "virtual_transactions.json"
        vt_data = self._read_json_file(vt_file) or {"transactions": {}}
        return vt_data.get("transactions", {})
    
    # Check/Receipt Management
    def save_check(self, check_id: str, pdf_data: bytes, metadata: Dict[str, Any]) -> str:
        """Save PDF check/receipt"""
        try:
            # Generate filename
            date_str = datetime.now().strftime("%Y_%m_%d")
            account_id = metadata.get("account_id", "unknown")
            tx_id = metadata.get("transaction_id", check_id)
            
            filename = f"CHECK_{date_str}_{account_id}_{tx_id}.pdf"
            file_path = self.base_path / "checks" / filename
            
            # Save PDF
            with open(file_path, 'wb') as f:
                f.write(pdf_data)
                
            # Save metadata
            meta_path = file_path.with_suffix('.json')
            metadata["filename"] = filename
            metadata["saved_at"] = datetime.now().isoformat()
            self._write_json_file(meta_path, metadata)
            
            logger.info(f"Saved check: {filename}")
            return str(file_path)
        except Exception as e:
            logger.error(f"Failed to save check {check_id}: {e}")
            raise
    
    def load_check_metadata(self, filename: str) -> Optional[Dict[str, Any]]:
        """Load check metadata"""
        meta_path = self.base_path / "checks" / filename.replace('.pdf', '.json')
        return self._read_json_file(meta_path)
    
    def list_checks(self, transaction_id: Optional[str] = None) -> List[Dict[str, Any]]:
        """List all checks, optionally filtered by transaction ID"""
        checks = []
        checks_dir = self.base_path / "checks"
        
        for meta_file in checks_dir.glob("*.json"):
            metadata = self._read_json_file(meta_file)
            if metadata:
                if transaction_id is None or metadata.get("transaction_id") == transaction_id:
                    checks.append(metadata)
                    
        return sorted(checks, key=lambda x: x.get("saved_at", ""), reverse=True)
    
    # Settings Management
    def save_settings(self, settings: Dict[str, Any]) -> bool:
        """Save application settings"""
        try:
            settings["updated_at"] = datetime.now().isoformat()
            file_path = self.base_path / "settings.json"
            self._write_json_file(file_path, settings)
            
            logger.info("Saved application settings")
            return True
        except Exception as e:
            logger.error(f"Failed to save settings: {e}")
            return False
    
    def load_settings(self) -> Dict[str, Any]:
        """Load application settings"""
        file_path = self.base_path / "settings.json"
        return self._read_json_file(file_path) or {
            "balance_update_interval": 14400,  # 4 hours in seconds
            "gate_relogin_interval": 1800,     # 30 minutes in seconds
            "rate_limit_per_minute": 240,      # Gate.io rate limit
            "admin_token": None,               # WebSocket admin token
            "payment_methods": ["SBP", "Tinkoff"],
            "alternate_payments": True,
            "ocr_validation": True
        }
    
    def cleanup_old_data(self, days: int = 30):
        """Clean up old transaction data and checks"""
        try:
            cutoff_date = datetime.now().timestamp() - (days * 24 * 60 * 60)
            
            # Clean old transactions
            trans_dir = self.base_path / "transactions"
            for file_path in trans_dir.glob("*.json"):
                if file_path.name != "virtual_transactions.json":
                    trans_data = self._read_json_file(file_path)
                    if trans_data:
                        created_at = trans_data.get("created_at", "")
                        if created_at:
                            created_timestamp = datetime.fromisoformat(created_at).timestamp()
                            if created_timestamp < cutoff_date and trans_data.get("status") in ["completed", "cancelled"]:
                                file_path.unlink()
                                logger.info(f"Cleaned up old transaction: {file_path.name}")
            
            # Clean old checks
            checks_dir = self.base_path / "checks"
            for meta_file in checks_dir.glob("*.json"):
                metadata = self._read_json_file(meta_file)
                if metadata:
                    saved_at = metadata.get("saved_at", "")
                    if saved_at:
                        saved_timestamp = datetime.fromisoformat(saved_at).timestamp()
                        if saved_timestamp < cutoff_date:
                            # Delete PDF and metadata
                            pdf_file = meta_file.with_suffix('.pdf')
                            if pdf_file.exists():
                                pdf_file.unlink()
                            meta_file.unlink()
                            logger.info(f"Cleaned up old check: {meta_file.name}")
                            
        except Exception as e:
            logger.error(f"Error during cleanup: {e}")