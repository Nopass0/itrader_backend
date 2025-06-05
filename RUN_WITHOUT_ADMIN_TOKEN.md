# Running Binaries Without admin_token Error

The `admin_token` error occurs because many binaries try to load the full application config, which requires all fields including `admin_token`. Here are several solutions:

## Solution 1: Set Environment Variable (Recommended)

Set the `APP__admin_token` environment variable before running any binary:

```bash
# One-time command
APP__admin_token=test cargo run --bin gate_list_pending

# Or export for the session
export APP__admin_token=test
cargo run --bin gate_list_pending
```

## Solution 2: Use Helper Scripts

Use the provided helper scripts that set the environment:

```bash
# List pending transactions
./list_pending.sh

# Search for a transaction
./search_transaction.sh 123456

# Approve a transaction
./approve_transaction.sh 123456 receipt.pdf
```

## Solution 3: Direct Binary Execution

The binaries have been updated to use minimal configuration:

```bash
# These binaries now work without full config
cargo run --bin gate_list_pending
cargo run --bin gate_search_transaction 123456
```

## Solution 4: Create .env File

Create a `.env` file in the project root:

```bash
APP__admin_token=your-token-here
DATABASE_URL=postgres://postgres:password@localhost/itrader
REDIS_URL=redis://localhost:6379
```

Then use with dotenv:
```bash
dotenv cargo run --bin gate_list_pending
```

## Environment Variables for Gate.io Binaries

You can customize Gate.io binaries with these environment variables:

- `GATE_API_URL` - Gate.io API base URL (default: https://panel.gate.cx/api/v1)
- `COOKIE_FILE` - Cookie file path (default: .gate_cookies.json)
- `APP__admin_token` - Admin token (required for full config)

## Examples

### List Pending Transactions
```bash
APP__admin_token=test cargo run --bin gate_list_pending
```

### Search Transaction
```bash
APP__admin_token=test cargo run --bin gate_search_transaction 2450530
```

### Custom Cookie File
```bash
COOKIE_FILE=.gate_cookies_account1.json APP__admin_token=test cargo run --bin gate_list_pending
```

### Different API URL
```bash
GATE_API_URL=https://api.gate.io/api/v1 APP__admin_token=test cargo run --bin gate_list_pending
```

## Quick Aliases

Add these to your `.bashrc` or `.zshrc`:

```bash
alias gate-pending='APP__admin_token=test cargo run --bin gate_list_pending'
alias gate-search='APP__admin_token=test cargo run --bin gate_search_transaction'
alias gate-approve='APP__admin_token=test cargo run --bin gate_approve_transaction'
```

## Summary

The simplest solution is to always set `APP__admin_token` when running binaries:

```bash
APP__admin_token=test <your-command>
```

This prevents the "missing field admin_token" error while allowing the binaries to run normally.