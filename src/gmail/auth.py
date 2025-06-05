#!/usr/bin/env python3
"""
Gmail Authentication Manager
Handles OAuth2 authentication and credential storage in database
"""

import os
import json
import pickle
import psycopg2
from datetime import datetime
from google.auth.transport.requests import Request
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build
from google.oauth2.credentials import Credentials

# Gmail API scope
SCOPES = ['https://www.googleapis.com/auth/gmail.readonly']

class GmailAuthManager:
    def __init__(self, db_url: str):
        self.db_url = db_url
        
    def get_db_connection(self):
        """Get database connection"""
        return psycopg2.connect(self.db_url)
    
    def setup_gmail_account(self, credentials_file: str = None):
        """Setup Gmail account with OAuth2"""
        creds = None
        
        # Try to load existing credentials from database
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                SELECT credentials, token, refresh_token, token_expiry
                FROM gmail_accounts
                WHERE is_active = true
                LIMIT 1
            """)
            
            result = cur.fetchone()
            if result and result[1]:  # Has token
                creds_data = {
                    'token': result[1],
                    'refresh_token': result[2],
                    'token_uri': 'https://oauth2.googleapis.com/token',
                    'client_id': '',
                    'client_secret': '',
                    'scopes': SCOPES
                }
                
                # Extract client info from credentials if available
                if result[0]:
                    try:
                        cred_json = json.loads(result[0])
                        if 'installed' in cred_json:
                            creds_data['client_id'] = cred_json['installed']['client_id']
                            creds_data['client_secret'] = cred_json['installed']['client_secret']
                    except:
                        pass
                
                creds = Credentials.from_authorized_user_info(creds_data, SCOPES)
        except:
            pass
        finally:
            conn.close()
        
        # If credentials are invalid or don't exist
        if not creds or not creds.valid:
            if creds and creds.expired and creds.refresh_token:
                creds.refresh(Request())
            else:
                if not credentials_file or not os.path.exists(credentials_file):
                    raise Exception("Please provide path to credentials.json file")
                
                flow = InstalledAppFlow.from_client_secrets_file(
                    credentials_file, SCOPES)
                creds = flow.run_local_server(port=0)
            
            # Save credentials to database
            self._save_credentials(creds, credentials_file)
        
        return creds
    
    def _save_credentials(self, creds, credentials_file=None):
        """Save credentials to database"""
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            
            # Read credentials file if provided
            creds_json = ""
            if credentials_file and os.path.exists(credentials_file):
                with open(credentials_file, 'r') as f:
                    creds_json = f.read()
            
            # Get user email
            service = build('gmail', 'v1', credentials=creds)
            profile = service.users().getProfile(userId='me').execute()
            email = profile.get('emailAddress', 'unknown')
            
            # Upsert credentials
            cur.execute("""
                INSERT INTO gmail_accounts 
                (email, credentials, token, refresh_token, token_expiry, is_active)
                VALUES (%s, %s, %s, %s, %s, true)
                ON CONFLICT (email) DO UPDATE SET
                    credentials = EXCLUDED.credentials,
                    token = EXCLUDED.token,
                    refresh_token = EXCLUDED.refresh_token,
                    token_expiry = EXCLUDED.token_expiry,
                    is_active = true,
                    updated_at = CURRENT_TIMESTAMP
            """, (
                email,
                creds_json,
                creds.token,
                creds.refresh_token,
                creds.expiry
            ))
            
            conn.commit()
            print(f"✅ Gmail credentials saved for {email}")
            
        except Exception as e:
            conn.rollback()
            print(f"Error saving credentials: {e}")
        finally:
            conn.close()
    
    def get_gmail_service(self):
        """Get authenticated Gmail service"""
        creds = self.setup_gmail_account()
        return build('gmail', 'v1', credentials=creds)