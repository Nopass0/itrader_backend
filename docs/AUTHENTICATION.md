# Authentication Implementation

## Overview

The iTrader backend now supports both cookie-based authentication for Gate.io and API key authentication for Bybit.

## Gate.io Authentication

### Login with Credentials

The system can login to Gate.io using email and password credentials to obtain fresh cookies:

```rust
// Login and get cookies
let response = client.login(email, password).await?;
let cookies = client.get_cookies().await;
```

### Cookie Storage

Cookies are stored in JSON format compatible with browser extensions:

```json
[
  {
    "domain": ".panel.gate.cx",
    "expirationDate": 1748435739.0,
    "hostOnly": false,
    "httpOnly": true,
    "name": "sid",
    "path": "/",
    "secure": true,
    "value": "encrypted_session_id"
  }
]
```

### Using Cookies

Once cookies are obtained, they can be used for authenticated requests:

```rust
// Set cookies on client
client.set_cookies(cookies).await?;

// Make authenticated requests
let balance = client.get_balance("RUB").await?;
let transactions = client.get_transactions(filter).await?;
client.set_balance("RUB", amount).await?;
```

## Testing

### Running Tests

1. **Login Test** - Get fresh cookies:
   ```bash
   ./test.sh gate-login
   ```

2. **Authentication Test** - Test with saved cookies:
   ```bash
   ./test.sh gate-auth
   ```

3. **Other Tests** - Use authenticated client:
   ```bash
   ./test.sh gate-balance
   ./test.sh gate-tx
   ```

### Test Data Files

- `test_data/gate_creditials.json` - Login credentials
- `test_data/gate_cookie.json` - Saved cookies (auto-generated)
- `test_data/bybit_creditials.json` - Bybit API credentials

## Implementation Details

### Cookie Parsing

The system includes a cookie parser that extracts cookies from HTTP headers:

```rust
fn parse_cookie_string(cookie_str: &str) -> Option<Cookie> {
    // Parses Set-Cookie headers
    // Extracts name, value, domain, path, etc.
    // Sets appropriate expiration dates
}
```

### Rate Limiting

All requests are rate-limited to avoid hitting API limits:

```rust
self.rate_limiter.check_and_wait("gate").await?;
```

### Error Handling

The system handles various authentication errors:
- `SessionExpired` - Cookies have expired
- `CloudflareBlock` - Cloudflare challenge detected
- `Authentication` - Invalid credentials

## Production Usage

In production, the orchestrator will:

1. Load all Gate.io accounts from the database
2. Login to each account to get fresh cookies
3. Save cookies to the database
4. Set balance to 1,000,000 RUB
5. Start monitoring for transactions

The cookies are refreshed automatically when they expire.