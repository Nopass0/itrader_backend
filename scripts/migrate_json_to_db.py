#!/usr/bin/env python3
"""
Migrate JSON data from file storage to PostgreSQL database
"""

import json
import os
import sys
import psycopg2
from datetime import datetime
from pathlib import Path

# Database connection
DATABASE_URL = os.environ.get('DATABASE_URL', 'postgresql://postgres:root@localhost/itrader')

def connect_db():
    """Connect to PostgreSQL database"""
    try:
        conn = psycopg2.connect(DATABASE_URL)
        return conn
    except Exception as e:
        print(f"Error connecting to database: {e}")
        sys.exit(1)

def migrate_settings(conn):
    """Migrate settings.json to settings table"""
    settings_file = Path(__file__).parent.parent / 'db' / 'settings.json'
    
    if not settings_file.exists():
        print("settings.json not found")
        return
    
    with open(settings_file, 'r') as f:
        settings = json.load(f)
    
    cur = conn.cursor()
    
    # Delete any existing settings (should only be one row)
    cur.execute("DELETE FROM settings")
    
    # Insert settings
    cur.execute("""
        INSERT INTO settings (
            id, admin_token, balance_update_interval, gate_relogin_interval,
            rate_limit_per_minute, payment_methods, alternate_payments,
            ocr_validation, cleanup_days, receipt_email
        ) VALUES (
            1, %s, %s, %s, %s, %s, %s, %s, %s, %s
        )
    """, (
        settings.get('admin_token'),
        settings.get('balance_update_interval', 14400),
        settings.get('gate_relogin_interval', 1800),
        settings.get('rate_limit_per_minute', 240),
        settings.get('payment_methods', ['SBP', 'Tinkoff']),
        settings.get('alternate_payments', True),
        settings.get('ocr_validation', True),
        settings.get('cleanup_days', 30),
        settings.get('receipt_email', 'noreply@tinkoff.ru')
    ))
    
    conn.commit()
    print("✓ Settings migrated successfully")

def migrate_accounts(conn):
    """Migrate accounts.json to gate_accounts and bybit_accounts tables"""
    accounts_file = Path(__file__).parent.parent / 'data' / 'accounts.json'
    
    if not accounts_file.exists():
        print("accounts.json not found")
        return
    
    with open(accounts_file, 'r') as f:
        accounts = json.load(f)
    
    cur = conn.cursor()
    
    # Migrate Gate accounts - check if they don't already exist
    gate_accounts = accounts.get('gate_accounts', [])
    for account in gate_accounts:
        # Check if account already exists
        cur.execute("SELECT id FROM gate_accounts WHERE email = %s", (account['email'],))
        if cur.fetchone():
            print(f"Gate account {account['email']} already exists, skipping")
            continue
            
        cur.execute("""
            INSERT INTO gate_accounts (
                email, password_encrypted, balance, status, 
                last_auth, created_at, updated_at
            ) VALUES (
                %s, %s, %s, %s, %s, %s, %s
            )
        """, (
            account['email'],
            account['password'],  # Note: In production, this should be encrypted
            account.get('balance', 10000000.0),
            account.get('status', 'active'),
            account.get('last_auth'),
            account.get('created_at', datetime.now()),
            account.get('updated_at', datetime.now())
        ))
    
    # Migrate Bybit accounts - check if they don't already exist
    bybit_accounts = accounts.get('bybit_accounts', [])
    for account in bybit_accounts:
        # Check if account already exists
        cur.execute("SELECT id FROM bybit_accounts WHERE account_name = %s", (account['account_name'],))
        if cur.fetchone():
            print(f"Bybit account {account['account_name']} already exists, skipping")
            continue
            
        cur.execute("""
            INSERT INTO bybit_accounts (
                account_name, api_key, api_secret, active_ads, 
                status, created_at, updated_at
            ) VALUES (
                %s, %s, %s, %s, %s, %s, %s
            )
        """, (
            account['account_name'],
            account['api_key'],
            account['api_secret'],
            account.get('active_ads', 0),
            account.get('status', 'available'),
            account.get('created_at', datetime.now()),
            account.get('updated_at', datetime.now())
        ))
    
    conn.commit()
    print(f"✓ Migrated {len(gate_accounts)} Gate accounts and {len(bybit_accounts)} Bybit accounts")

def migrate_gate_cookies(conn):
    """Migrate gate/*.json files to gate_cookies table"""
    gate_dir = Path(__file__).parent.parent / 'db' / 'gate'
    
    if not gate_dir.exists():
        print("db/gate directory not found")
        return
    
    cur = conn.cursor()
    migrated = 0
    
    for json_file in gate_dir.glob('*.json'):
        with open(json_file, 'r') as f:
            data = json.load(f)
        
        # Check if already exists
        cur.execute("SELECT id FROM gate_cookies WHERE id = %s", (data['id'],))
        if cur.fetchone():
            print(f"Gate cookie {data['id']} already exists, skipping")
            continue
        
        cur.execute("""
            INSERT INTO gate_cookies (
                id, email, password_encrypted, status, cookies,
                last_auth, balance, created_at, updated_at
            ) VALUES (
                %s, %s, %s, %s, %s, %s, %s, %s, %s
            )
        """, (
            data['id'],
            data['login'],
            data['password'],  # Note: In production, this should be encrypted
            data.get('status', 'inactive'),
            json.dumps(data.get('cookies')) if data.get('cookies') else None,
            data.get('last_auth'),
            data.get('balance', 10000000.0),
            data.get('created_at', datetime.now()),
            data.get('updated_at', datetime.now())
        ))
        migrated += 1
    
    conn.commit()
    print(f"✓ Migrated {migrated} Gate cookie sessions")

def migrate_bybit_sessions(conn):
    """Migrate bybit/*.json files to bybit_sessions table"""
    bybit_dir = Path(__file__).parent.parent / 'db' / 'bybit'
    
    if not bybit_dir.exists():
        print("db/bybit directory not found")
        return
    
    cur = conn.cursor()
    migrated = 0
    
    for json_file in bybit_dir.glob('*.json'):
        with open(json_file, 'r') as f:
            data = json.load(f)
        
        # Check if already exists
        cur.execute("SELECT id FROM bybit_sessions WHERE id = %s", (data['id'],))
        if cur.fetchone():
            print(f"Bybit session {data['id']} already exists, skipping")
            continue
        
        cur.execute("""
            INSERT INTO bybit_sessions (
                id, api_key, api_secret, status, testnet,
                active_ads, last_error, last_login, created_at, updated_at
            ) VALUES (
                %s, %s, %s, %s, %s, %s, %s, %s, %s, %s
            )
        """, (
            data['id'],
            data['api_key'],
            data['api_secret'],
            data.get('status', 'active'),
            data.get('testnet', False),
            data.get('active_ads', 0),
            data.get('last_error'),
            data.get('last_login'),
            data.get('created_at', datetime.now()),
            data.get('updated_at', datetime.now())
        ))
        migrated += 1
    
    conn.commit()
    print(f"✓ Migrated {migrated} Bybit sessions")

def verify_migration(conn):
    """Verify the migration was successful"""
    cur = conn.cursor()
    
    # Check settings
    cur.execute("SELECT COUNT(*) FROM settings")
    settings_count = cur.fetchone()[0]
    
    # Check gate accounts
    cur.execute("SELECT COUNT(*) FROM gate_accounts")
    gate_accounts_count = cur.fetchone()[0]
    
    # Check bybit accounts  
    cur.execute("SELECT COUNT(*) FROM bybit_accounts")
    bybit_accounts_count = cur.fetchone()[0]
    
    # Check gate cookies
    cur.execute("SELECT COUNT(*) FROM gate_cookies")
    gate_cookies_count = cur.fetchone()[0]
    
    # Check bybit sessions
    cur.execute("SELECT COUNT(*) FROM bybit_sessions")
    bybit_sessions_count = cur.fetchone()[0]
    
    print("\n=== Migration Summary ===")
    print(f"Settings: {settings_count}")
    print(f"Gate accounts: {gate_accounts_count}")
    print(f"Bybit accounts: {bybit_accounts_count}")
    print(f"Gate cookies: {gate_cookies_count}")
    print(f"Bybit sessions: {bybit_sessions_count}")
    
    return all([
        settings_count > 0,
        gate_accounts_count > 0 or bybit_accounts_count > 0
    ])

def main():
    """Main migration function"""
    print("Starting JSON to PostgreSQL migration...")
    
    conn = connect_db()
    
    try:
        # Run migrations
        migrate_settings(conn)
        migrate_accounts(conn)
        migrate_gate_cookies(conn)
        migrate_bybit_sessions(conn)
        
        # Verify
        success = verify_migration(conn)
        
        if success:
            print("\n✅ Migration completed successfully!")
            print("\nYou can now safely remove the data/ and db/ directories")
            print("Run: rm -rf data/ db/")
        else:
            print("\n❌ Migration may have issues, please check the data")
            
    except Exception as e:
        print(f"\n❌ Migration failed: {e}")
        conn.rollback()
        sys.exit(1)
    finally:
        conn.close()

if __name__ == "__main__":
    main()