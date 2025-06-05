"""
Gmail API Client with OAuth2 Authentication
Handles email operations for the trading system
"""

import os
import base64
import logging
import pickle
import mimetypes
from typing import List, Dict, Any, Optional, Tuple
from datetime import datetime, timedelta
from email.mime.base import MIMEBase
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from email import encoders

from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError

from db_manager import DatabaseManager

logger = logging.getLogger(__name__)

# Gmail API scopes
SCOPES = [
    'https://www.googleapis.com/auth/gmail.readonly',
    'https://www.googleapis.com/auth/gmail.send',
    'https://www.googleapis.com/auth/gmail.modify',
    'https://www.googleapis.com/auth/gmail.compose'
]


class GmailClient:
    """Gmail API client with OAuth2 authentication"""
    
    def __init__(self, db_manager: DatabaseManager, account_email: Optional[str] = None):
        self.db = db_manager
        self.account_email = account_email
        self.service = None
        self.creds = None
        
    def authenticate(self, force_reauth: bool = False) -> bool:
        """Authenticate with Gmail API using OAuth2"""
        try:
            # Load existing token
            if not force_reauth:
                token_data = self.db.load_gmail_token(self.account_email)
                if token_data:
                    self.creds = Credentials(
                        token=token_data.get('access_token'),
                        refresh_token=token_data.get('refresh_token'),
                        token_uri=token_data.get('token_uri', 'https://oauth2.googleapis.com/token'),
                        client_id=token_data.get('client_id'),
                        client_secret=token_data.get('client_secret'),
                        scopes=token_data.get('scopes', SCOPES)
                    )
            
            # Refresh token if expired
            if self.creds and self.creds.expired and self.creds.refresh_token:
                logger.info("Refreshing expired token...")
                self.creds.refresh(Request())
                self._save_credentials()
            
            # Get new token if needed
            if not self.creds or not self.creds.valid:
                # Load credentials
                credentials_data = self.db.load_gmail_credentials()
                if not credentials_data:
                    logger.error("No Gmail credentials found. Please set up credentials first.")
                    return False
                
                flow = InstalledAppFlow.from_client_config(
                    credentials_data, SCOPES
                )
                
                # Run local server for OAuth2 flow
                logger.info("Starting OAuth2 authentication flow...")
                self.creds = flow.run_local_server(port=0)
                
                # Save the credentials
                self._save_credentials()
            
            # Build service
            self.service = build('gmail', 'v1', credentials=self.creds)
            
            # Get account email if not specified
            if not self.account_email:
                profile = self.service.users().getProfile(userId='me').execute()
                self.account_email = profile.get('emailAddress')
                logger.info(f"Authenticated as: {self.account_email}")
            
            return True
            
        except Exception as e:
            logger.error(f"Gmail authentication failed: {e}")
            return False
    
    def _save_credentials(self):
        """Save credentials to database"""
        if self.creds:
            token_data = {
                'access_token': self.creds.token,
                'refresh_token': self.creds.refresh_token,
                'token_uri': self.creds.token_uri,
                'client_id': self.creds.client_id,
                'client_secret': self.creds.client_secret,
                'scopes': self.creds.scopes,
                'expiry': self.creds.expiry.isoformat() if self.creds.expiry else None
            }
            self.db.save_gmail_token(token_data, self.account_email)
    
    def list_messages(self, query: str = '', max_results: int = 100) -> List[Dict[str, Any]]:
        """List messages matching query"""
        try:
            messages = []
            results = self.service.users().messages().list(
                userId='me',
                q=query,
                maxResults=max_results
            ).execute()
            
            messages = results.get('messages', [])
            
            # Handle pagination
            while 'nextPageToken' in results and len(messages) < max_results:
                page_token = results['nextPageToken']
                results = self.service.users().messages().list(
                    userId='me',
                    q=query,
                    pageToken=page_token,
                    maxResults=max_results - len(messages)
                ).execute()
                messages.extend(results.get('messages', []))
            
            return messages[:max_results]
            
        except HttpError as error:
            logger.error(f'Error listing messages: {error}')
            return []
    
    def get_message(self, message_id: str) -> Optional[Dict[str, Any]]:
        """Get full message details"""
        try:
            message = self.service.users().messages().get(
                userId='me',
                id=message_id,
                format='full'
            ).execute()
            
            # Parse message
            parsed = self._parse_message(message)
            return parsed
            
        except HttpError as error:
            logger.error(f'Error getting message {message_id}: {error}')
            return None
    
    def _parse_message(self, message: Dict[str, Any]) -> Dict[str, Any]:
        """Parse Gmail message into readable format"""
        parsed = {
            'id': message['id'],
            'threadId': message.get('threadId'),
            'labelIds': message.get('labelIds', []),
            'snippet': message.get('snippet', ''),
            'historyId': message.get('historyId'),
            'internalDate': message.get('internalDate'),
            'headers': {},
            'body': '',
            'attachments': []
        }
        
        # Parse headers
        if 'payload' in message and 'headers' in message['payload']:
            for header in message['payload']['headers']:
                parsed['headers'][header['name']] = header['value']
        
        # Parse body and attachments
        if 'payload' in message:
            self._parse_parts(message['payload'], parsed)
        
        # Convert internal date to datetime
        if parsed['internalDate']:
            timestamp = int(parsed['internalDate']) / 1000
            parsed['datetime'] = datetime.fromtimestamp(timestamp)
        
        return parsed
    
    def _parse_parts(self, payload: Dict[str, Any], parsed: Dict[str, Any]):
        """Recursively parse message parts"""
        if 'parts' in payload:
            for part in payload['parts']:
                self._parse_parts(part, parsed)
        else:
            # Extract body
            if payload.get('mimeType') in ['text/plain', 'text/html']:
                data = payload.get('body', {}).get('data', '')
                if data:
                    decoded = base64.urlsafe_b64decode(data).decode('utf-8', errors='ignore')
                    if payload.get('mimeType') == 'text/plain' and not parsed['body']:
                        parsed['body'] = decoded
                    elif payload.get('mimeType') == 'text/html':
                        parsed['html_body'] = decoded
            
            # Extract attachments
            if payload.get('filename'):
                attachment = {
                    'filename': payload['filename'],
                    'mimeType': payload.get('mimeType'),
                    'attachmentId': payload.get('body', {}).get('attachmentId'),
                    'size': payload.get('body', {}).get('size', 0)
                }
                parsed['attachments'].append(attachment)
    
    def get_attachment(self, message_id: str, attachment_id: str) -> Optional[bytes]:
        """Download attachment data"""
        try:
            attachment = self.service.users().messages().attachments().get(
                userId='me',
                messageId=message_id,
                id=attachment_id
            ).execute()
            
            data = attachment['data']
            return base64.urlsafe_b64decode(data)
            
        except HttpError as error:
            logger.error(f'Error downloading attachment: {error}')
            return None
    
    def send_message(self, to: str, subject: str, body: str, 
                    attachments: Optional[List[Tuple[str, bytes]]] = None) -> Optional[str]:
        """Send email message"""
        try:
            message = MIMEMultipart()
            message['to'] = to
            message['from'] = self.account_email
            message['subject'] = subject
            
            # Add body
            msg = MIMEText(body)
            message.attach(msg)
            
            # Add attachments
            if attachments:
                for filename, data in attachments:
                    self._attach_file(message, filename, data)
            
            # Send message
            raw = base64.urlsafe_b64encode(message.as_bytes()).decode()
            body = {'raw': raw}
            
            sent_message = self.service.users().messages().send(
                userId='me',
                body=body
            ).execute()
            
            logger.info(f"Sent message to {to}: {sent_message['id']}")
            return sent_message['id']
            
        except HttpError as error:
            logger.error(f'Error sending message: {error}')
            return None
    
    def _attach_file(self, message: MIMEMultipart, filename: str, data: bytes):
        """Attach file to message"""
        # Guess the content type
        content_type, encoding = mimetypes.guess_type(filename)
        if content_type is None or encoding is not None:
            content_type = 'application/octet-stream'
        
        main_type, sub_type = content_type.split('/', 1)
        
        # Create attachment
        msg = MIMEBase(main_type, sub_type)
        msg.set_payload(data)
        encoders.encode_base64(msg)
        msg.add_header(
            'Content-Disposition',
            f'attachment; filename= {filename}'
        )
        
        message.attach(msg)
    
    def mark_as_read(self, message_id: str) -> bool:
        """Mark message as read"""
        try:
            self.service.users().messages().modify(
                userId='me',
                id=message_id,
                body={'removeLabelIds': ['UNREAD']}
            ).execute()
            return True
        except HttpError as error:
            logger.error(f'Error marking message as read: {error}')
            return False
    
    def trash_message(self, message_id: str) -> bool:
        """Move message to trash"""
        try:
            self.service.users().messages().trash(
                userId='me',
                id=message_id
            ).execute()
            return True
        except HttpError as error:
            logger.error(f'Error trashing message: {error}')
            return False
    
    def search_receipts(self, start_date: Optional[datetime] = None, 
                       end_date: Optional[datetime] = None) -> List[Dict[str, Any]]:
        """Search for receipt emails from T-Bank"""
        query_parts = [
            'from:(noreply@tbank.ru OR noreply@tinkoff.ru)',
            'has:attachment',
            '(filename:pdf OR filename:PDF)'
        ]
        
        if start_date:
            query_parts.append(f'after:{start_date.strftime("%Y/%m/%d")}')
        
        if end_date:
            query_parts.append(f'before:{end_date.strftime("%Y/%m/%d")}')
        
        query = ' '.join(query_parts)
        messages = self.list_messages(query)
        
        receipts = []
        for msg in messages:
            full_message = self.get_message(msg['id'])
            if full_message:
                # Check if it has PDF attachments
                pdf_attachments = [
                    att for att in full_message['attachments']
                    if att['filename'].lower().endswith('.pdf')
                ]
                
                if pdf_attachments:
                    receipt_info = {
                        'message_id': full_message['id'],
                        'date': full_message.get('datetime'),
                        'from': full_message['headers'].get('From', ''),
                        'subject': full_message['headers'].get('Subject', ''),
                        'attachments': pdf_attachments
                    }
                    receipts.append(receipt_info)
        
        return receipts
    
    def download_receipt(self, message_id: str, attachment_id: str, 
                        filename: str) -> Optional[bytes]:
        """Download receipt PDF"""
        pdf_data = self.get_attachment(message_id, attachment_id)
        if pdf_data:
            logger.info(f"Downloaded receipt: {filename}")
            return pdf_data
        return None
    
    def monitor_new_receipts(self, callback, check_interval: int = 60):
        """Monitor for new receipt emails (run in thread)"""
        last_check = datetime.now()
        
        while True:
            try:
                # Search for new receipts since last check
                receipts = self.search_receipts(start_date=last_check)
                
                for receipt in receipts:
                    # Process each new receipt
                    for attachment in receipt['attachments']:
                        pdf_data = self.download_receipt(
                            receipt['message_id'],
                            attachment['attachmentId'],
                            attachment['filename']
                        )
                        
                        if pdf_data:
                            # Call the callback with receipt data
                            callback({
                                'message_id': receipt['message_id'],
                                'filename': attachment['filename'],
                                'pdf_data': pdf_data,
                                'date': receipt['date'],
                                'from': receipt['from'],
                                'subject': receipt['subject']
                            })
                
                last_check = datetime.now()
                
            except Exception as e:
                logger.error(f"Error monitoring receipts: {e}")
            
            # Wait before next check
            import time
            time.sleep(check_interval)
    
    def is_authenticated(self) -> bool:
        """Check if client is authenticated"""
        return self.service is not None and self.creds is not None and self.creds.valid
    
    def get_account_info(self) -> Optional[Dict[str, str]]:
        """Get Gmail account info"""
        if not self.is_authenticated():
            return None
            
        try:
            profile = self.service.users().getProfile(userId='me').execute()
            return {
                'emailAddress': profile.get('emailAddress'),
                'messagesTotal': profile.get('messagesTotal'),
                'threadsTotal': profile.get('threadsTotal'),
                'historyId': profile.get('historyId')
            }
        except HttpError as error:
            logger.error(f'Error getting account info: {error}')
            return None