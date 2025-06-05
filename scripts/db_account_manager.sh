#!/bin/bash

# Database Account Manager for iTrader Backend
# This script provides database management for accounts via CLI

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Database connection - use environment variable or default to postgres/root
DB_URL="${DATABASE_URL:-postgresql://postgres:root@localhost/itrader}"

# Check if psql is available
check_psql() {
    if ! command -v psql >/dev/null 2>&1; then
        echo -e "${RED}Error: psql is required but not installed${NC}"
        echo "Install with: sudo apt-get install postgresql-client"
        exit 1
    fi
}

# List Gate accounts
list_gate_accounts() {
    echo -e "\n${CYAN}=== Gate.io Accounts ===${NC}"
    
    # Check if there are any accounts
    count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM gate_accounts" | xargs)
    
    if [ "$count" -eq 0 ]; then
        echo -e "${YELLOW}  No Gate.io accounts found${NC}"
        return
    fi
    
    # Print header
    echo -e "${BLUE}   ID | Email                          | Balance (RUB)  | Status   | Last Auth${NC}"
    echo "  ----+--------------------------------+----------------+----------+------------------"
    
    psql "$DB_URL" -t -A -F'|' -c "
        SELECT 
            id, 
            email, 
            balance, 
            status,
            CASE 
                WHEN last_auth IS NULL THEN 'Never'
                ELSE to_char(last_auth, 'YYYY-MM-DD HH24:MI')
            END as last_auth
        FROM gate_accounts 
        ORDER BY id
    " | while IFS='|' read -r id email balance status last_auth; do
        # Handle empty fields
        id=${id:-0}
        email=${email:-"N/A"}
        balance=${balance:-0}
        status=${status:-"unknown"}
        last_auth=${last_auth:-"Never"}
        
        # Convert balance to proper format (handle decimal separator)
        balance_formatted=$(echo "$balance" | tr ',' '.')
        printf "  %3d | %-30s | %14.2f | %-8s | %s\n" \
            "$id" \
            "$email" \
            "$balance_formatted" \
            "$status" \
            "$last_auth"
    done
}

# List Bybit accounts
list_bybit_accounts() {
    echo -e "\n${CYAN}=== Bybit Accounts ===${NC}"
    
    # Check if there are any accounts
    count=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM bybit_accounts" | xargs)
    
    if [ "$count" -eq 0 ]; then
        echo -e "${YELLOW}  No Bybit accounts found${NC}"
        return
    fi
    
    # Print header
    echo -e "${BLUE}   ID | Account Name         | API Key       | Ads | Status     | Last Used${NC}"
    echo "  ----+----------------------+---------------+-----+------------+------------------"
    
    psql "$DB_URL" -t -A -F'|' -c "
        SELECT 
            id, 
            account_name, 
            SUBSTR(api_key, 1, 10) || '...' as api_key_short,
            active_ads, 
            status,
            CASE 
                WHEN last_used IS NULL THEN 'Never'
                ELSE to_char(last_used, 'YYYY-MM-DD HH24:MI')
            END as last_used
        FROM bybit_accounts 
        ORDER BY id
    " | while IFS='|' read -r id account_name api_key active_ads status last_used; do
        # Handle empty fields
        id=${id:-0}
        account_name=${account_name:-"N/A"}
        api_key=${api_key:-"N/A"}
        active_ads=${active_ads:-0}
        status=${status:-"unknown"}
        last_used=${last_used:-"Never"}
        
        printf "  %3d | %-20s | %-13s | %3d | %-10s | %s\n" \
            "$id" \
            "$account_name" \
            "$api_key" \
            "$active_ads" \
            "$status" \
            "$last_used"
    done
}

# Add Gate account
add_gate_account() {
    echo -e "\n${CYAN}=== Add Gate.io Account ===${NC}"
    
    read -p "Email: " email
    
    # Check if email already exists
    existing=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM gate_accounts WHERE email='$email'" | tr -d ' ')
    if [ "$existing" -gt 0 ]; then
        echo -e "${RED}❌ Account with email $email already exists${NC}"
        return
    fi
    
    read -s -p "Password: " password
    echo
    read -p "Initial balance (RUB) [10000000]: " balance
    balance=${balance:-10000000}
    
    result=$(psql "$DB_URL" -t -c "
        INSERT INTO gate_accounts (email, password_encrypted, balance, status)
        VALUES ('$email', '$password', $balance, 'active')
        RETURNING id;
    " 2>&1)
    
    # Extract just the ID from the result (remove whitespace)
    id=$(echo "$result" | grep -E '^\s*[0-9]+\s*$' | tr -d ' ')
    
    if [ -n "$id" ]; then
        echo -e "${GREEN}✅ Gate.io account added successfully (ID: $id)${NC}"
    else
        echo -e "${RED}❌ Failed to add account: $result${NC}"
    fi
}

# Add Bybit account
add_bybit_account() {
    echo -e "\n${CYAN}=== Add Bybit Account ===${NC}"
    
    read -p "Account name: " account_name
    
    # Check if account name already exists
    existing=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM bybit_accounts WHERE account_name='$account_name'" | tr -d ' ')
    if [ "$existing" -gt 0 ]; then
        echo -e "${RED}❌ Account with name $account_name already exists${NC}"
        return
    fi
    
    read -p "API Key: " api_key
    read -s -p "API Secret: " api_secret
    echo
    
    result=$(psql "$DB_URL" -t -c "
        INSERT INTO bybit_accounts (account_name, api_key, api_secret, active_ads, status)
        VALUES ('$account_name', '$api_key', '$api_secret', 0, 'available')
        RETURNING id;
    " 2>&1)
    
    # Extract just the ID from the result (remove whitespace)
    id=$(echo "$result" | grep -E '^\s*[0-9]+\s*$' | tr -d ' ')
    
    if [ -n "$id" ]; then
        echo -e "${GREEN}✅ Bybit account added successfully (ID: $id)${NC}"
    else
        echo -e "${RED}❌ Failed to add account: $result${NC}"
    fi
}

# Update Gate account
update_gate_account() {
    list_gate_accounts
    echo
    read -p "Enter account ID to update (or 0 to cancel): " id
    
    if [ "$id" -eq 0 ]; then
        return
    fi
    
    echo -e "\n${CYAN}Updating Gate.io Account${NC}"
    read -p "New email (press Enter to keep current): " new_email
    read -p "New balance (press Enter to keep current): " new_balance
    read -s -p "New password (press Enter to keep current): " new_password
    echo
    read -p "New status (active/inactive, press Enter to keep current): " new_status
    
    # Build update query
    updates=""
    if [ -n "$new_email" ]; then
        updates="${updates}email='$new_email', "
    fi
    if [ -n "$new_balance" ]; then
        updates="${updates}balance=$new_balance, "
    fi
    if [ -n "$new_password" ]; then
        updates="${updates}password_encrypted='$new_password', "
    fi
    if [ -n "$new_status" ]; then
        updates="${updates}status='$new_status', "
    fi
    
    if [ -n "$updates" ]; then
        updates="${updates}updated_at=CURRENT_TIMESTAMP"
        
        result=$(psql "$DB_URL" -c "
            UPDATE gate_accounts 
            SET $updates
            WHERE id = $id;
        " 2>&1)
        
        if [[ $result == *"UPDATE"* ]]; then
            echo -e "${GREEN}✅ Account updated successfully${NC}"
        else
            echo -e "${RED}❌ Failed to update account: $result${NC}"
        fi
    else
        echo -e "${YELLOW}No changes made${NC}"
    fi
}

# Delete Gate account
delete_gate_account() {
    list_gate_accounts
    echo
    read -p "Enter account ID to delete (or 0 to cancel): " id
    
    if [ "$id" -eq 0 ]; then
        return
    fi
    
    # Get account info
    email=$(psql "$DB_URL" -t -c "SELECT email FROM gate_accounts WHERE id = $id" | xargs)
    
    if [ -z "$email" ]; then
        echo -e "${RED}Account not found${NC}"
        return
    fi
    
    echo -e "\n${YELLOW}⚠️  Are you sure you want to delete this account?${NC}"
    echo "Email: $email"
    read -p "Type 'yes' to confirm: " confirm
    
    if [ "$confirm" == "yes" ]; then
        psql "$DB_URL" -c "DELETE FROM gate_accounts WHERE id = $id"
        echo -e "${GREEN}✅ Account deleted successfully${NC}"
    else
        echo -e "${YELLOW}Deletion cancelled${NC}"
    fi
}

# Show statistics
show_statistics() {
    echo -e "\n${CYAN}=== Account Statistics ===${NC}"
    
    stats=$(psql "$DB_URL" -t -c "
        SELECT 
            (SELECT COUNT(*) FROM gate_accounts) as gate_count,
            (SELECT COUNT(*) FROM gate_accounts WHERE status = 'active') as gate_active,
            (SELECT COUNT(*) FROM bybit_accounts) as bybit_count,
            (SELECT COUNT(*) FROM bybit_accounts WHERE status = 'available') as bybit_available,
            (SELECT COALESCE(SUM(balance), 0) FROM gate_accounts) as total_balance,
            (SELECT COALESCE(SUM(active_ads), 0) FROM bybit_accounts) as total_ads
    ")
    
    IFS='|' read -r gate_count gate_active bybit_count bybit_available total_balance total_ads <<< "$stats"
    
    echo -e "${BLUE}Gate.io Accounts:${NC} $gate_count (Active: $gate_active)"
    echo -e "${BLUE}Bybit Accounts:${NC} $bybit_count (Available: $bybit_available)"
    echo -e "${BLUE}Total Gate Balance:${NC} $(printf "%'.2f" $total_balance) RUB"
    echo -e "${BLUE}Total Active Ads:${NC} $total_ads"
}

# Main menu
main_menu() {
    while true; do
        echo -e "\n${PURPLE}==============================="
        echo "  Database Account Management"
        echo -e "===============================${NC}"
        echo
        echo -e "${CYAN}Gate.io Accounts:${NC}"
        echo "  1. List Gate.io accounts"
        echo "  2. Add Gate.io account"
        echo "  3. Update Gate.io account"
        echo "  4. Delete Gate.io account"
        echo
        echo -e "${CYAN}Bybit Accounts:${NC}"
        echo "  5. List Bybit accounts"
        echo "  6. Add Bybit account"
        echo "  7. Update Bybit account"
        echo "  8. Delete Bybit account"
        echo
        echo -e "${CYAN}Utilities:${NC}"
        echo "  9. Show statistics"
        echo "  10. Run SQL query"
        echo
        echo "  0. Exit"
        echo
        read -p "Select option: " choice
        
        case $choice in
            1) list_gate_accounts; read -p "Press Enter to continue..." ;;
            2) add_gate_account ;;
            3) update_gate_account ;;
            4) delete_gate_account ;;
            5) list_bybit_accounts; read -p "Press Enter to continue..." ;;
            6) add_bybit_account ;;
            7) echo "Update Bybit account - Not implemented yet" ;;
            8) echo "Delete Bybit account - Not implemented yet" ;;
            9) show_statistics; read -p "Press Enter to continue..." ;;
            10) 
                echo "Enter SQL query (or 'exit' to cancel):"
                read -p "> " query
                if [ "$query" != "exit" ]; then
                    psql "$DB_URL" -c "$query"
                    read -p "Press Enter to continue..."
                fi
                ;;
            0) echo -e "${GREEN}Goodbye!${NC}"; break ;;
            *) echo -e "${RED}Invalid option${NC}" ;;
        esac
    done
}

# Check dependencies and run
check_psql
main_menu