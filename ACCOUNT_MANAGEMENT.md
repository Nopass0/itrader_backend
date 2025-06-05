# Account Management System

The iTrader Backend supports managing multiple Gate.io and Bybit accounts for automated P2P trading operations.

## Quick Start

### Interactive Account Manager

```bash
./run.sh --settings
```

This opens an interactive menu where you can:
- Add, edit, and delete Gate.io accounts
- Add, edit, and delete Bybit accounts
- Import/export account configurations
- View account statistics

### Account Configuration File

Accounts are stored in `data/accounts.json`:

```json
{
  "gate_accounts": [
    {
      "id": 1,
      "email": "trader@example.com",
      "password": "secure_password",
      "balance": 10000000.0,
      "status": "active",
      "created_at": "2025-01-04T12:00:00Z",
      "updated_at": "2025-01-04T12:00:00Z"
    }
  ],
  "bybit_accounts": [
    {
      "id": 1,
      "account_name": "main_account",
      "api_key": "your_api_key",
      "api_secret": "your_api_secret",
      "active_ads": 0,
      "status": "available",
      "created_at": "2025-01-04T12:00:00Z",
      "updated_at": "2025-01-04T12:00:00Z"
    }
  ],
  "last_updated": "2025-01-04T12:00:00Z"
}
```

## Multi-Account Strategy

### Gate.io Accounts
- Multiple Gate.io accounts can monitor different transaction pools
- Each account maintains its own balance and authentication state
- Accounts are rotated to distribute load and avoid rate limits

### Bybit Accounts
- Multiple Bybit accounts allow creating more advertisements
- Each account has a limit on active ads (typically 8 per account)
- System automatically selects available accounts for new ads
- Accounts are marked as "busy" when at capacity

## Account Management Menu

### Main Menu Options

1. **List Gate.io accounts** - Shows all configured Gate.io accounts with balances
2. **Add Gate.io account** - Add a new Gate.io account
3. **Edit Gate.io account** - Modify existing account credentials or balance
4. **Delete Gate.io account** - Remove a Gate.io account

5. **List Bybit accounts** - Shows all configured Bybit accounts
6. **Add Bybit account** - Add a new Bybit account with API credentials
7. **Edit Bybit account** - Modify API keys or account details
8. **Delete Bybit account** - Remove a Bybit account

9. **Import accounts** - Load accounts from an external JSON file
10. **Export accounts** - Save current accounts to a timestamped file
11. **Show statistics** - Display account counts and total balances

## Account Fields

### Gate.io Account Fields
- **id**: Unique identifier (auto-generated)
- **email**: Gate.io login email
- **password**: Account password (stored in plain text - use dedicated trading accounts)
- **balance**: Available balance in RUB
- **status**: Account status (active/inactive)
- **cookies**: Authentication cookies (managed automatically)
- **last_auth**: Last successful authentication timestamp

### Bybit Account Fields
- **id**: Unique identifier (auto-generated)
- **account_name**: Friendly name for the account
- **api_key**: Bybit API key
- **api_secret**: Bybit API secret
- **active_ads**: Number of currently active advertisements
- **status**: Account status (available/busy)

## Security Considerations

⚠️ **Important Security Notes:**

1. Credentials are stored in plain text in `data/accounts.json`
2. Use dedicated trading accounts with limited permissions
3. Set appropriate file permissions: `chmod 600 data/accounts.json`
4. Never commit account files to version control
5. Use environment-specific configurations for different environments

## Example: Setting Up Multiple Accounts

### 1. Import Pre-configured Accounts
```bash
./run.sh --settings
# Select option 9 (Import accounts)
# Enter path: data/accounts_example_multi.json
```

### 2. Add Accounts Manually
```bash
./run.sh --settings

# Add Gate.io account
# Select option 2
# Enter email: trader1@example.com
# Enter password: ********
# Enter balance: 15000000

# Add Bybit account
# Select option 6
# Enter account name: bybit_main
# Enter API key: XXXXXXXXXX
# Enter API secret: YYYYYYYYYY
```

### 3. Verify Configuration
```bash
./run.sh --settings
# Select option 11 (Show statistics)
```

## Operational Tips

### Account Rotation
The system automatically rotates through available accounts:
- Gate.io: Selects accounts based on balance and last usage
- Bybit: Selects accounts with lowest active ad count

### Monitoring Account Health
- Check account statistics regularly
- Monitor for authentication failures
- Track active ad counts to ensure availability
- Review balances to ensure sufficient funds

### Backup Strategy
```bash
# Regular backups
./run.sh --settings
# Select option 10 (Export accounts)

# Automated backup
cp data/accounts.json "backups/accounts_$(date +%Y%m%d_%H%M%S).json"
```

## Troubleshooting

### Common Issues

1. **Authentication Failed**
   - Verify credentials are correct
   - Check if account requires 2FA
   - Ensure IP is whitelisted

2. **API Key Invalid**
   - Regenerate API keys in Bybit
   - Ensure API has P2P permissions
   - Check API key hasn't expired

3. **Account at Capacity**
   - Add more Bybit accounts
   - Clean up old advertisements
   - Increase account limits if possible

### Debug Commands

```bash
# Check account file integrity
jq . data/accounts.json

# List all Gate.io emails
jq -r '.gate_accounts[].email' data/accounts.json

# Count Bybit accounts
jq '.bybit_accounts | length' data/accounts.json

# Find accounts with active ads
jq '.bybit_accounts[] | select(.active_ads > 0)' data/accounts.json
```

## Integration with Trading System

The account management system integrates with:
- **Orchestrator**: Selects accounts for transaction processing
- **Auto Trader**: Rotates accounts for ad creation
- **Rate Limiter**: Tracks usage per account
- **WebSocket API**: Reports account status in real-time