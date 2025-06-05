#!/bin/bash

# Account Manager for iTrader Backend
# This script provides an interactive menu for managing Gate.io and Bybit accounts

# Initialize accounts file if it doesn't exist
init_accounts_file() {
    if [ ! -f "$ACCOUNTS_FILE" ]; then
        mkdir -p "$(dirname "$ACCOUNTS_FILE")"
        cat > "$ACCOUNTS_FILE" << 'EOF'
{
  "gate_accounts": [],
  "bybit_accounts": [],
  "last_updated": ""
}
EOF
        echo -e "${GREEN}✅ Created new accounts file${NC}"
    fi
}

# Load accounts from JSON file
load_accounts() {
    init_accounts_file
    GATE_ACCOUNTS=$(jq -r '.gate_accounts' "$ACCOUNTS_FILE" 2>/dev/null || echo "[]")
    BYBIT_ACCOUNTS=$(jq -r '.bybit_accounts' "$ACCOUNTS_FILE" 2>/dev/null || echo "[]")
}

# Save accounts to JSON file
save_accounts() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    jq --arg ts "$timestamp" '.last_updated = $ts' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
    echo -e "${GREEN}✅ Accounts saved successfully${NC}"
}

# Generate UUID
generate_uuid() {
    if command -v uuidgen >/dev/null 2>&1; then
        uuidgen | tr '[:upper:]' '[:lower:]'
    else
        # Fallback UUID generation
        echo "$(date +%s)-$(shuf -i 1000-9999 -n 1)-$(shuf -i 1000-9999 -n 1)-$(shuf -i 1000-9999 -n 1)"
    fi
}

# List Gate accounts
list_gate_accounts() {
    echo -e "\n${CYAN}=== Gate.io Accounts ===${NC}"
    local count=$(echo "$GATE_ACCOUNTS" | jq 'length')
    
    if [ "$count" -eq 0 ] || [ "$count" == "null" ]; then
        echo -e "${YELLOW}No Gate.io accounts configured${NC}"
    else
        echo "$GATE_ACCOUNTS" | jq -r '.[] | "ID: \(.id) | \(.email) | Balance: \(.balance) RUB | Status: \(.status)"' | nl
    fi
}

# List Bybit accounts
list_bybit_accounts() {
    echo -e "\n${CYAN}=== Bybit Accounts ===${NC}"
    local count=$(echo "$BYBIT_ACCOUNTS" | jq 'length')
    
    if [ "$count" -eq 0 ] || [ "$count" == "null" ]; then
        echo -e "${YELLOW}No Bybit accounts configured${NC}"
    else
        echo "$BYBIT_ACCOUNTS" | jq -r '.[] | "ID: \(.id) | \(.account_name) | API Key: \(.api_key[0:10])... | Active Ads: \(.active_ads) | Status: \(.status)"' | nl
    fi
}

# Add Gate account
add_gate_account() {
    echo -e "\n${CYAN}=== Add Gate.io Account ===${NC}"
    
    read -p "Email: " email
    read -s -p "Password: " password
    echo
    read -p "Initial balance (RUB) [10000000]: " balance
    balance=${balance:-10000000}
    
    local id=$(generate_uuid)
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local account_id=$(($(echo "$GATE_ACCOUNTS" | jq 'length') + 1))
    
    local new_account=$(jq -n \
        --arg id "$id" \
        --arg email "$email" \
        --arg password "$password" \
        --arg balance "$balance" \
        --arg timestamp "$timestamp" \
        --arg account_id "$account_id" \
        '{
            id: ($account_id | tonumber),
            email: $email,
            password: $password,
            balance: ($balance | tonumber),
            status: "active",
            created_at: $timestamp,
            updated_at: $timestamp
        }')
    
    # Update accounts.json
    jq --argjson account "$new_account" '.gate_accounts += [$account]' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
    
    echo -e "${GREEN}✅ Gate.io account added successfully${NC}"
}

# Add Bybit account
add_bybit_account() {
    echo -e "\n${CYAN}=== Add Bybit Account ===${NC}"
    
    read -p "Account name: " account_name
    read -p "API Key: " api_key
    read -s -p "API Secret: " api_secret
    echo
    
    local id=$(generate_uuid)
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local account_id=$(($(echo "$BYBIT_ACCOUNTS" | jq 'length') + 1))
    
    local new_account=$(jq -n \
        --arg id "$id" \
        --arg account_name "$account_name" \
        --arg api_key "$api_key" \
        --arg api_secret "$api_secret" \
        --arg timestamp "$timestamp" \
        --arg account_id "$account_id" \
        '{
            id: ($account_id | tonumber),
            account_name: $account_name,
            api_key: $api_key,
            api_secret: $api_secret,
            active_ads: 0,
            status: "available",
            created_at: $timestamp,
            updated_at: $timestamp
        }')
    
    # Update accounts.json
    jq --argjson account "$new_account" '.bybit_accounts += [$account]' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
    
    echo -e "${GREEN}✅ Bybit account added successfully${NC}"
}

# Edit Gate account
edit_gate_account() {
    list_gate_accounts
    echo
    read -p "Enter account number to edit (or 0 to cancel): " num
    
    if [ "$num" -eq 0 ]; then
        return
    fi
    
    local index=$((num - 1))
    local account=$(echo "$GATE_ACCOUNTS" | jq ".[$index]")
    
    if [ "$account" == "null" ]; then
        echo -e "${RED}Invalid account number${NC}"
        return
    fi
    
    echo -e "\n${CYAN}Editing Gate.io Account${NC}"
    echo "Current email: $(echo "$account" | jq -r '.email')"
    read -p "New email (press Enter to keep current): " new_email
    
    echo "Current balance: $(echo "$account" | jq -r '.balance')"
    read -p "New balance (press Enter to keep current): " new_balance
    
    read -s -p "New password (press Enter to keep current): " new_password
    echo
    
    # Update fields if provided
    if [ -n "$new_email" ]; then
        account=$(echo "$account" | jq --arg email "$new_email" '.email = $email')
    fi
    
    if [ -n "$new_balance" ]; then
        account=$(echo "$account" | jq --arg balance "$new_balance" '.balance = ($balance | tonumber)')
    fi
    
    if [ -n "$new_password" ]; then
        account=$(echo "$account" | jq --arg password "$new_password" '.password = $password')
    fi
    
    # Update timestamp
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    account=$(echo "$account" | jq --arg ts "$timestamp" '.updated_at = $ts')
    
    # Save back to file
    jq --argjson account "$account" --arg index "$index" '.gate_accounts[($index | tonumber)] = $account' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
    
    echo -e "${GREEN}✅ Account updated successfully${NC}"
}

# Edit Bybit account
edit_bybit_account() {
    list_bybit_accounts
    echo
    read -p "Enter account number to edit (or 0 to cancel): " num
    
    if [ "$num" -eq 0 ]; then
        return
    fi
    
    local index=$((num - 1))
    local account=$(echo "$BYBIT_ACCOUNTS" | jq ".[$index]")
    
    if [ "$account" == "null" ]; then
        echo -e "${RED}Invalid account number${NC}"
        return
    fi
    
    echo -e "\n${CYAN}Editing Bybit Account${NC}"
    echo "Current account name: $(echo "$account" | jq -r '.account_name')"
    read -p "New account name (press Enter to keep current): " new_name
    
    echo "Current API key: $(echo "$account" | jq -r '.api_key' | cut -c1-20)..."
    read -p "New API key (press Enter to keep current): " new_api_key
    
    read -s -p "New API secret (press Enter to keep current): " new_api_secret
    echo
    
    # Update fields if provided
    if [ -n "$new_name" ]; then
        account=$(echo "$account" | jq --arg name "$new_name" '.account_name = $name')
    fi
    
    if [ -n "$new_api_key" ]; then
        account=$(echo "$account" | jq --arg key "$new_api_key" '.api_key = $key')
    fi
    
    if [ -n "$new_api_secret" ]; then
        account=$(echo "$account" | jq --arg secret "$new_api_secret" '.api_secret = $secret')
    fi
    
    # Update timestamp
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    account=$(echo "$account" | jq --arg ts "$timestamp" '.updated_at = $ts')
    
    # Save back to file
    jq --argjson account "$account" --arg index "$index" '.bybit_accounts[($index | tonumber)] = $account' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
    
    echo -e "${GREEN}✅ Account updated successfully${NC}"
}

# Delete Gate account
delete_gate_account() {
    list_gate_accounts
    echo
    read -p "Enter account number to delete (or 0 to cancel): " num
    
    if [ "$num" -eq 0 ]; then
        return
    fi
    
    local index=$((num - 1))
    local account=$(echo "$GATE_ACCOUNTS" | jq ".[$index]")
    
    if [ "$account" == "null" ]; then
        echo -e "${RED}Invalid account number${NC}"
        return
    fi
    
    echo -e "\n${YELLOW}⚠️  Are you sure you want to delete this account?${NC}"
    echo "Email: $(echo "$account" | jq -r '.email')"
    read -p "Type 'yes' to confirm: " confirm
    
    if [ "$confirm" == "yes" ]; then
        jq --arg index "$index" 'del(.gate_accounts[($index | tonumber)])' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
        echo -e "${GREEN}✅ Account deleted successfully${NC}"
    else
        echo -e "${YELLOW}Deletion cancelled${NC}"
    fi
}

# Delete Bybit account
delete_bybit_account() {
    list_bybit_accounts
    echo
    read -p "Enter account number to delete (or 0 to cancel): " num
    
    if [ "$num" -eq 0 ]; then
        return
    fi
    
    local index=$((num - 1))
    local account=$(echo "$BYBIT_ACCOUNTS" | jq ".[$index]")
    
    if [ "$account" == "null" ]; then
        echo -e "${RED}Invalid account number${NC}"
        return
    fi
    
    echo -e "\n${YELLOW}⚠️  Are you sure you want to delete this account?${NC}"
    echo "Account name: $(echo "$account" | jq -r '.account_name')"
    read -p "Type 'yes' to confirm: " confirm
    
    if [ "$confirm" == "yes" ]; then
        jq --arg index "$index" 'del(.bybit_accounts[($index | tonumber)])' "$ACCOUNTS_FILE" > "${ACCOUNTS_FILE}.tmp" && mv "${ACCOUNTS_FILE}.tmp" "$ACCOUNTS_FILE"
        echo -e "${GREEN}✅ Account deleted successfully${NC}"
    else
        echo -e "${YELLOW}Deletion cancelled${NC}"
    fi
}

# Import accounts from file
import_accounts() {
    echo -e "\n${CYAN}=== Import Accounts ===${NC}"
    read -p "Enter path to import file: " import_path
    
    if [ ! -f "$import_path" ]; then
        echo -e "${RED}File not found: $import_path${NC}"
        return
    fi
    
    # Validate JSON
    if ! jq empty "$import_path" 2>/dev/null; then
        echo -e "${RED}Invalid JSON file${NC}"
        return
    fi
    
    # Check structure
    if ! jq -e '.gate_accounts' "$import_path" >/dev/null 2>&1 || ! jq -e '.bybit_accounts' "$import_path" >/dev/null 2>&1; then
        echo -e "${RED}Invalid file structure. Expected 'gate_accounts' and 'bybit_accounts' arrays${NC}"
        return
    fi
    
    echo -e "${YELLOW}This will replace all existing accounts. Continue?${NC}"
    read -p "Type 'yes' to confirm: " confirm
    
    if [ "$confirm" == "yes" ]; then
        cp "$import_path" "$ACCOUNTS_FILE"
        echo -e "${GREEN}✅ Accounts imported successfully${NC}"
        load_accounts
    else
        echo -e "${YELLOW}Import cancelled${NC}"
    fi
}

# Export accounts
export_accounts() {
    echo -e "\n${CYAN}=== Export Accounts ===${NC}"
    local export_path="accounts_export_$(date +%Y%m%d_%H%M%S).json"
    
    cp "$ACCOUNTS_FILE" "$export_path"
    echo -e "${GREEN}✅ Accounts exported to: $export_path${NC}"
}

# Show statistics
show_statistics() {
    load_accounts
    
    echo -e "\n${CYAN}=== Account Statistics ===${NC}"
    
    local gate_count=$(echo "$GATE_ACCOUNTS" | jq 'length')
    local bybit_count=$(echo "$BYBIT_ACCOUNTS" | jq 'length')
    local total_balance=$(echo "$GATE_ACCOUNTS" | jq '[.[] | .balance] | add // 0')
    local active_ads=$(echo "$BYBIT_ACCOUNTS" | jq '[.[] | .active_ads] | add // 0')
    
    echo -e "${BLUE}Gate.io Accounts:${NC} $gate_count"
    echo -e "${BLUE}Bybit Accounts:${NC} $bybit_count"
    echo -e "${BLUE}Total Gate Balance:${NC} $(echo "$total_balance" | awk '{printf "%\047.2f", $1}') RUB"
    echo -e "${BLUE}Total Active Ads:${NC} $active_ads"
    
    local last_updated=$(jq -r '.last_updated // "Never"' "$ACCOUNTS_FILE")
    echo -e "${BLUE}Last Updated:${NC} $last_updated"
}

# Main menu
manage_accounts_menu() {
    while true; do
        load_accounts
        
        echo -e "\n${PURPLE}==============================="
        echo "    Account Management Menu"
        echo -e "===============================${NC}"
        echo
        echo -e "${CYAN}Gate.io Accounts:${NC}"
        echo "  1. List Gate.io accounts"
        echo "  2. Add Gate.io account"
        echo "  3. Edit Gate.io account"
        echo "  4. Delete Gate.io account"
        echo
        echo -e "${CYAN}Bybit Accounts:${NC}"
        echo "  5. List Bybit accounts"
        echo "  6. Add Bybit account"
        echo "  7. Edit Bybit account"
        echo "  8. Delete Bybit account"
        echo
        echo -e "${CYAN}Utilities:${NC}"
        echo "  9. Import accounts from file"
        echo "  10. Export accounts to file"
        echo "  11. Show statistics"
        echo
        echo "  0. Exit"
        echo
        read -p "Select option: " choice
        
        case $choice in
            1) list_gate_accounts; read -p "Press Enter to continue..." ;;
            2) add_gate_account ;;
            3) edit_gate_account ;;
            4) delete_gate_account ;;
            5) list_bybit_accounts; read -p "Press Enter to continue..." ;;
            6) add_bybit_account ;;
            7) edit_bybit_account ;;
            8) delete_bybit_account ;;
            9) import_accounts ;;
            10) export_accounts ;;
            11) show_statistics; read -p "Press Enter to continue..." ;;
            0) echo -e "${GREEN}Goodbye!${NC}"; break ;;
            *) echo -e "${RED}Invalid option${NC}" ;;
        esac
    done
}

# Check dependencies
check_dependencies() {
    if ! command -v jq >/dev/null 2>&1; then
        echo -e "${RED}Error: jq is required but not installed${NC}"
        echo "Install with: sudo apt-get install jq"
        exit 1
    fi
}

# Initialize
check_dependencies