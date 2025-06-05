#!/bin/bash

# Migration script for JSON to PostgreSQL using psql directly

DB_URL="${DATABASE_URL:-postgresql://postgres:root@localhost/itrader}"

echo "Starting JSON to PostgreSQL migration..."

# Function to execute SQL
exec_sql() {
    psql "$DB_URL" -c "$1" 2>&1
}

# Migrate settings.json
migrate_settings() {
    echo "Migrating settings..."
    
    if [ -f "db/settings.json" ]; then
        # Extract values from JSON
        admin_token=$(jq -r '.admin_token // empty' db/settings.json)
        balance_update_interval=$(jq -r '.balance_update_interval // 14400' db/settings.json)
        gate_relogin_interval=$(jq -r '.gate_relogin_interval // 1800' db/settings.json)
        rate_limit_per_minute=$(jq -r '.rate_limit_per_minute // 240' db/settings.json)
        payment_methods=$(jq -r '.payment_methods | @json // "[]"' db/settings.json)
        alternate_payments=$(jq -r '.alternate_payments // true' db/settings.json)
        ocr_validation=$(jq -r '.ocr_validation // true' db/settings.json)
        cleanup_days=$(jq -r '.cleanup_days // 30' db/settings.json)
        receipt_email=$(jq -r '.receipt_email // "noreply@tinkoff.ru"' db/settings.json)
        
        # Delete existing settings
        exec_sql "DELETE FROM settings"
        
        # Insert new settings
        exec_sql "INSERT INTO settings (id, admin_token, balance_update_interval, gate_relogin_interval, rate_limit_per_minute, payment_methods, alternate_payments, ocr_validation, cleanup_days, receipt_email) VALUES (1, '$admin_token', $balance_update_interval, $gate_relogin_interval, $rate_limit_per_minute, ARRAY['SBP', 'Tinkoff'], $alternate_payments, $ocr_validation, $cleanup_days, '$receipt_email')"
        
        echo "✓ Settings migrated successfully"
    else
        echo "settings.json not found"
    fi
}

# Migrate accounts.json
migrate_accounts() {
    echo "Migrating accounts..."
    
    if [ -f "data/accounts.json" ]; then
        # Migrate Gate accounts
        jq -c '.gate_accounts[]?' data/accounts.json | while read -r account; do
            email=$(echo "$account" | jq -r '.email')
            password=$(echo "$account" | jq -r '.password')
            balance=$(echo "$account" | jq -r '.balance // 10000000')
            status=$(echo "$account" | jq -r '.status // "active"')
            created_at=$(echo "$account" | jq -r '.created_at // "now()"')
            updated_at=$(echo "$account" | jq -r '.updated_at // "now()"')
            
            # Check if account exists
            exists=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM gate_accounts WHERE email='$email'" | xargs)
            if [ "$exists" -eq 0 ]; then
                exec_sql "INSERT INTO gate_accounts (email, password, balance, status, created_at, updated_at) VALUES ('$email', '$password', $balance, '$status', '$created_at', '$updated_at')"
                echo "✓ Migrated Gate account: $email"
            else
                echo "- Gate account $email already exists, skipping"
            fi
        done
        
        # Migrate Bybit accounts
        jq -c '.bybit_accounts[]?' data/accounts.json | while read -r account; do
            account_name=$(echo "$account" | jq -r '.account_name')
            api_key=$(echo "$account" | jq -r '.api_key')
            api_secret=$(echo "$account" | jq -r '.api_secret')
            active_ads=$(echo "$account" | jq -r '.active_ads // 0')
            status=$(echo "$account" | jq -r '.status // "available"')
            created_at=$(echo "$account" | jq -r '.created_at // "now()"')
            updated_at=$(echo "$account" | jq -r '.updated_at // "now()"')
            
            # Check if account exists
            exists=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM bybit_accounts WHERE account_name='$account_name'" | xargs)
            if [ "$exists" -eq 0 ]; then
                exec_sql "INSERT INTO bybit_accounts (account_name, api_key, api_secret, active_ads, status, created_at, updated_at) VALUES ('$account_name', '$api_key', '$api_secret', $active_ads, '$status', '$created_at', '$updated_at')"
                echo "✓ Migrated Bybit account: $account_name"
            else
                echo "- Bybit account $account_name already exists, skipping"
            fi
        done
    else
        echo "accounts.json not found"
    fi
}

# Migrate Gate cookies
migrate_gate_cookies() {
    echo "Migrating Gate cookies..."
    
    if [ -d "db/gate" ]; then
        for json_file in db/gate/*.json; do
            if [ -f "$json_file" ]; then
                id=$(jq -r '.id' "$json_file")
                email=$(jq -r '.login' "$json_file")
                password=$(jq -r '.password' "$json_file")
                status=$(jq -r '.status // "inactive"' "$json_file")
                cookies=$(jq -r '.cookies // null' "$json_file")
                last_auth=$(jq -r '.last_auth // null' "$json_file")
                balance=$(jq -r '.balance // 10000000' "$json_file")
                created_at=$(jq -r '.created_at // "now()"' "$json_file")
                updated_at=$(jq -r '.updated_at // "now()"' "$json_file")
                
                # Check if exists
                exists=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM gate_cookies WHERE id='$id'" | xargs)
                if [ "$exists" -eq 0 ]; then
                    if [ "$cookies" = "null" ]; then
                        cookies_sql="NULL"
                    else
                        cookies_sql="'$(echo "$cookies" | jq -c .)'"
                    fi
                    
                    if [ "$last_auth" = "null" ]; then
                        last_auth_sql="NULL"
                    else
                        last_auth_sql="'$last_auth'"
                    fi
                    
                    exec_sql "INSERT INTO gate_cookies (id, email, password, status, cookies, last_auth, balance, created_at, updated_at) VALUES ('$id', '$email', '$password', '$status', $cookies_sql, $last_auth_sql, $balance, '$created_at', '$updated_at')"
                    echo "✓ Migrated Gate cookie: $id"
                else
                    echo "- Gate cookie $id already exists, skipping"
                fi
            fi
        done
    else
        echo "db/gate directory not found"
    fi
}

# Migrate Bybit sessions
migrate_bybit_sessions() {
    echo "Migrating Bybit sessions..."
    
    if [ -d "db/bybit" ]; then
        for json_file in db/bybit/*.json; do
            if [ -f "$json_file" ]; then
                id=$(jq -r '.id' "$json_file")
                api_key=$(jq -r '.api_key' "$json_file")
                api_secret=$(jq -r '.api_secret' "$json_file")
                status=$(jq -r '.status // "active"' "$json_file")
                testnet=$(jq -r '.testnet // false' "$json_file")
                active_ads=$(jq -r '.active_ads // 0' "$json_file")
                last_error=$(jq -r '.last_error // null' "$json_file")
                last_login=$(jq -r '.last_login // null' "$json_file")
                created_at=$(jq -r '.created_at // "now()"' "$json_file")
                updated_at=$(jq -r '.updated_at // "now()"' "$json_file")
                
                # Check if exists
                exists=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM bybit_sessions WHERE id='$id'" | xargs)
                if [ "$exists" -eq 0 ]; then
                    if [ "$last_error" = "null" ]; then
                        last_error_sql="NULL"
                    else
                        last_error_sql="'$last_error'"
                    fi
                    
                    if [ "$last_login" = "null" ]; then
                        last_login_sql="NULL"
                    else
                        last_login_sql="'$last_login'"
                    fi
                    
                    exec_sql "INSERT INTO bybit_sessions (id, api_key, api_secret, status, testnet, active_ads, last_error, last_login, created_at, updated_at) VALUES ('$id', '$api_key', '$api_secret', '$status', $testnet, $active_ads, $last_error_sql, $last_login_sql, '$created_at', '$updated_at')"
                    echo "✓ Migrated Bybit session: $id"
                else
                    echo "- Bybit session $id already exists, skipping"
                fi
            fi
        done
    else
        echo "db/bybit directory not found"
    fi
}

# Verify migration
verify_migration() {
    echo -e "\n=== Migration Summary ==="
    
    settings_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM settings" | xargs)
    gate_accounts_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM gate_accounts" | xargs)
    bybit_accounts_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM bybit_accounts" | xargs)
    gate_cookies_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM gate_cookies" | xargs)
    bybit_sessions_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM bybit_sessions" | xargs)
    
    echo "Settings: $settings_count"
    echo "Gate accounts: $gate_accounts_count"
    echo "Bybit accounts: $bybit_accounts_count"
    echo "Gate cookies: $gate_cookies_count"
    echo "Bybit sessions: $bybit_sessions_count"
    
    if [ "$settings_count" -gt 0 ] && ([ "$gate_accounts_count" -gt 0 ] || [ "$bybit_accounts_count" -gt 0 ]); then
        echo -e "\n✅ Migration completed successfully!"
        echo -e "\nYou can now safely remove the data/ and db/ directories"
        echo "Run: rm -rf data/ db/"
        return 0
    else
        echo -e "\n❌ Migration may have issues, please check the data"
        return 1
    fi
}

# Check if jq is installed
if ! command -v jq >/dev/null 2>&1; then
    echo "Error: jq is required but not installed"
    echo "Install with: sudo apt-get install jq"
    exit 1
fi

# Run migrations
migrate_settings
migrate_accounts
migrate_gate_cookies
migrate_bybit_sessions

# Verify
verify_migration