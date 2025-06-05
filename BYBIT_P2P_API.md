# Bybit P2P API Implementation

## Overview

This document describes the Bybit P2P API implementation in the itrader_backend project. The implementation provides methods for managing P2P advertisements, orders, and chat messages.

## Requirements

### Account Requirements
To use the Bybit P2P API, your account must meet these requirements:

1. **Verified Advertiser Status**: Must be a verified P2P advertiser (VA level or above)
2. **P2P API Permissions**: Must enable P2P permissions in API Key Management
3. **Merchant Verification**: Must complete Bybit's merchant verification process

Without these requirements, all P2P API endpoints will return 404 errors.

### How to Enable P2P API Access
1. Become a P2P advertiser on Bybit
2. Complete merchant verification
3. Reach VA (Verified Advertiser) status
4. Enable P2P permissions when creating API keys

## Implemented Methods

### Advertisement Management
- `get_all_my_advertisements()` - Get all advertisements (active, inactive, hidden)
- `get_active_advertisements()` - Get only active advertisements
- `get_my_advertisements()` - Get advertisements with basic filtering
- `create_advertisement()` - Create a new P2P advertisement
- `delete_advertisement()` - Delete an existing advertisement
- `update_advertisement_status()` - Update advertisement status

### Order Management
- `get_advertisement_orders(ad_id)` - Get all orders for a specific advertisement
- `get_order_details(order_id)` - Get detailed information about an order
- `create_order()` - Create a new P2P order
- `confirm_payment_received()` - Confirm payment received for an order
- `release_order()` - Release cryptocurrency for a completed order
- `appeal_order()` - Appeal a disputed order
- `monitor_order_status()` - Monitor order status changes

### Chat Management
- `get_chat_messages(order_id)` - Get chat messages for a specific order
- `send_chat_message()` - Send a message in order chat
- `get_all_order_chats(ad_id)` - Get all chats for all orders of an advertisement

### Account Management
- `get_account_info()` - Get P2P account information
- `get_active_ads_count()` - Get count of active advertisements
- `is_account_available()` - Check if account can create more ads

## API Endpoints Used

All endpoints use the base URL: `https://api.bybit.com`

- Advertisement endpoints: `/p2p/item/*`
- Order endpoints: `/p2p/order/*`
- Chat endpoints: `/p2p/chat/*`
- Account endpoints: `/p2p/account-info`

## Authentication

The implementation uses Bybit's standard authentication:
- HMAC-SHA256 signature generation
- Required headers: X-BAPI-API-KEY, X-BAPI-TIMESTAMP, X-BAPI-SIGN, X-BAPI-RECV-WINDOW

## Error Handling

Common errors:
- **404 Not Found**: Account doesn't have P2P API permissions
- **401 Unauthorized**: Invalid API credentials
- **403 Forbidden**: Insufficient permissions
- **429 Too Many Requests**: Rate limit exceeded

## Testing

The implementation includes comprehensive tests in `tests/bybit_tests.rs`:
- `test_bybit_get_all_advertisements`
- `test_bybit_get_active_advertisements`
- `test_bybit_get_advertisement_chats`

Note: Tests will fail with 404 errors unless run with an account that has proper P2P API permissions.

## Configuration

Configuration is loaded from `config/default.toml`:
```toml
[bybit]
rest_url = "https://api.bybit.com"
ws_url = "wss://stream.bybit.com"
p2p_api_version = "v5"
max_ads_per_account = 2
```

## Usage Example

```rust
use itrader_backend::bybit::BybitP2PClient;

// Create client
let client = BybitP2PClient::new(
    "https://api.bybit.com".to_string(),
    api_key,
    api_secret,
    rate_limiter,
    max_ads_per_account,
)?;

// Get active advertisements
let ads = client.get_active_advertisements().await?;

// Get orders for an advertisement
let orders = client.get_advertisement_orders(&ad_id).await?;

// Get chat messages for all orders
let chats = client.get_all_order_chats(&ad_id).await?;
```

## References

- [Bybit P2P API Documentation](https://bybit-exchange.github.io/docs/p2p/guide)
- [Official Python SDK](https://github.com/bybit-exchange/bybit_p2p)
- [Bybit API Key Management](https://www.bybit.com/app/user/api-management)