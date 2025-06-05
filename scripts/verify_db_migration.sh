#!/bin/bash

# Verify database migration completeness

DB_URL="${DATABASE_URL:-postgresql://postgres:root@localhost/itrader}"

echo "=== Verifying Database Migration ==="
echo

# Check file system data
echo "📁 File System Data:"
echo "-------------------"

if [ -d "data" ]; then
    echo "data/ directory exists"
    if [ -f "data/accounts.json" ]; then
        gate_file_count=$(jq '.gate_accounts | length' data/accounts.json 2>/dev/null || echo 0)
        bybit_file_count=$(jq '.bybit_accounts | length' data/accounts.json 2>/dev/null || echo 0)
        echo "  - Gate accounts in file: $gate_file_count"
        echo "  - Bybit accounts in file: $bybit_file_count"
    fi
fi

if [ -d "db" ]; then
    echo "db/ directory exists"
    if [ -f "db/settings.json" ]; then
        echo "  - settings.json exists"
    fi
    if [ -d "db/gate" ]; then
        gate_cookie_files=$(find db/gate -name "*.json" | wc -l)
        echo "  - Gate cookie files: $gate_cookie_files"
    fi
    if [ -d "db/bybit" ]; then
        bybit_session_files=$(find db/bybit -name "*.json" | wc -l)
        echo "  - Bybit session files: $bybit_session_files"
    fi
fi

echo
echo "💾 Database Data:"
echo "-----------------"

# Check database tables
settings_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM settings" | xargs)
gate_accounts_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM gate_accounts" | xargs)
bybit_accounts_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM bybit_accounts" | xargs)
gate_cookies_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM gate_cookies" | xargs)
bybit_sessions_count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM bybit_sessions" | xargs)

echo "  - Settings: $settings_count"
echo "  - Gate accounts: $gate_accounts_count"
echo "  - Bybit accounts: $bybit_accounts_count"
echo "  - Gate cookies: $gate_cookies_count"
echo "  - Bybit sessions: $bybit_sessions_count"

echo
echo "📊 Comparison:"
echo "--------------"

# Compare counts
if [ -f "data/accounts.json" ]; then
    if [ "$gate_file_count" -eq "$gate_accounts_count" ]; then
        echo "✅ Gate accounts match"
    else
        echo "❌ Gate accounts mismatch (file: $gate_file_count, db: $gate_accounts_count)"
    fi
    
    if [ "$bybit_file_count" -eq "$bybit_accounts_count" ]; then
        echo "✅ Bybit accounts match"
    else
        echo "❌ Bybit accounts mismatch (file: $bybit_file_count, db: $bybit_accounts_count)"
    fi
fi

if [ -d "db/gate" ] && [ "$gate_cookie_files" -eq "$gate_cookies_count" ]; then
    echo "✅ Gate cookies match"
else
    echo "❌ Gate cookies mismatch (files: $gate_cookie_files, db: $gate_cookies_count)"
fi

if [ -d "db/bybit" ] && [ "$bybit_session_files" -eq "$bybit_sessions_count" ]; then
    echo "✅ Bybit sessions match"
else
    echo "❌ Bybit sessions mismatch (files: $bybit_session_files, db: $bybit_sessions_count)"
fi

echo
echo "🔍 Detailed Database Content:"
echo "-----------------------------"

echo
echo "Settings:"
psql "$DB_URL" -c "SELECT admin_token, balance_update_interval, gate_relogin_interval FROM settings LIMIT 1"

echo
echo "Gate Accounts (first 5):"
psql "$DB_URL" -c "SELECT id, email, balance, status FROM gate_accounts ORDER BY id LIMIT 5"

echo
echo "Bybit Accounts (first 5):"
psql "$DB_URL" -c "SELECT id, account_name, api_key, status FROM bybit_accounts ORDER BY id LIMIT 5"

echo
echo "✅ Migration verification complete!"