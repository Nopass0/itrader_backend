#!/bin/bash

# Batch import accounts from CSV files
# Format for Gate accounts: email,password,balance
# Format for Bybit accounts: account_name,api_key,api_secret

ACCOUNTS_FILE="data/accounts.json"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Initialize accounts file if needed
init_accounts_file() {
    if [ ! -f "$ACCOUNTS_FILE" ]; then
        mkdir -p "$(dirname "$ACCOUNTS_FILE")"
        echo '{"gate_accounts":[],"bybit_accounts":[],"last_updated":""}' > "$ACCOUNTS_FILE"
    fi
}

# Import Gate accounts from CSV
import_gate_csv() {
    local csv_file="$1"
    
    if [ ! -f "$csv_file" ]; then
        echo -e "${RED}File not found: $csv_file${NC}"
        return 1
    fi
    
    echo -e "${YELLOW}Importing Gate.io accounts from $csv_file...${NC}"
    
    local count=0
    local current_id=$(jq '.gate_accounts | length' "$ACCOUNTS_FILE")
    
    while IFS=',' read -r email password balance; do
        # Skip header or empty lines
        if [[ "$email" == "email" ]] || [[ -z "$email" ]]; then
            continue
        fi
        
        ((current_id++))
        ((count++))
        
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        
        # Add account to JSON
        jq --arg id "$current_id" \
           --arg email "$email" \
           --arg password "$password" \
           --arg balance "${balance:-10000000}" \
           --arg timestamp "$timestamp" \
           '.gate_accounts += [{
               id: ($id | tonumber),
               email: $email,
               password: $password,
               balance: ($balance | tonumber),
               status: "active",
               created_at: $timestamp,
               updated_at: $timestamp
           }]' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
        
        echo "  ✓ Added: $email"
    done < "$csv_file"
    
    echo -e "${GREEN}Imported $count Gate.io accounts${NC}"
}

# Import Bybit accounts from CSV
import_bybit_csv() {
    local csv_file="$1"
    
    if [ ! -f "$csv_file" ]; then
        echo -e "${RED}File not found: $csv_file${NC}"
        return 1
    fi
    
    echo -e "${YELLOW}Importing Bybit accounts from $csv_file...${NC}"
    
    local count=0
    local current_id=$(jq '.bybit_accounts | length' "$ACCOUNTS_FILE")
    
    while IFS=',' read -r account_name api_key api_secret; do
        # Skip header or empty lines
        if [[ "$account_name" == "account_name" ]] || [[ -z "$account_name" ]]; then
            continue
        fi
        
        ((current_id++))
        ((count++))
        
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        
        # Add account to JSON
        jq --arg id "$current_id" \
           --arg account_name "$account_name" \
           --arg api_key "$api_key" \
           --arg api_secret "$api_secret" \
           --arg timestamp "$timestamp" \
           '.bybit_accounts += [{
               id: ($id | tonumber),
               account_name: $account_name,
               api_key: $api_key,
               api_secret: $api_secret,
               active_ads: 0,
               status: "available",
               created_at: $timestamp,
               updated_at: $timestamp
           }]' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
        
        echo "  ✓ Added: $account_name"
    done < "$csv_file"
    
    echo -e "${GREEN}Imported $count Bybit accounts${NC}"
}

# Update last_updated timestamp
update_timestamp() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    jq --arg ts "$timestamp" '.last_updated = $ts' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
}

# Main
main() {
    echo -e "${GREEN}=== Batch Account Import ===${NC}"
    echo
    
    # Check dependencies
    if ! command -v jq >/dev/null 2>&1; then
        echo -e "${RED}Error: jq is required but not installed${NC}"
        exit 1
    fi
    
    init_accounts_file
    
    case "$1" in
        gate)
            if [ -z "$2" ]; then
                echo "Usage: $0 gate <csv_file>"
                echo "CSV format: email,password,balance"
                exit 1
            fi
            import_gate_csv "$2"
            update_timestamp
            ;;
            
        bybit)
            if [ -z "$2" ]; then
                echo "Usage: $0 bybit <csv_file>"
                echo "CSV format: account_name,api_key,api_secret"
                exit 1
            fi
            import_bybit_csv "$2"
            update_timestamp
            ;;
            
        *)
            echo "Usage: $0 {gate|bybit} <csv_file>"
            echo
            echo "Examples:"
            echo "  $0 gate gate_accounts.csv"
            echo "  $0 bybit bybit_accounts.csv"
            echo
            echo "CSV Formats:"
            echo "  Gate.io:  email,password,balance"
            echo "  Bybit:    account_name,api_key,api_secret"
            exit 1
            ;;
    esac
}

main "$@"