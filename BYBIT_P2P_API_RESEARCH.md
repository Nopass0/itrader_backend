# Bybit P2P API Research: Alternative Methods for Getting P2P Rates

## Overview

This document outlines alternative methods for fetching P2P rates from Bybit using their official API and Python SDK, based on research of their official documentation and APIs.

## Current Implementation

The current implementation uses:
- Direct HTTP requests to `https://api2.bybit.com/fiat/otc/item/online`
- Custom authentication headers
- Manual HMAC signature generation

## Official API Alternatives

### 1. Official P2P API Endpoint (V5)

Bybit provides an official V5 API endpoint for fetching P2P advertisements:

**Endpoint:** `POST /v5/p2p/item/online`  
**Base URL:** `https://api.bybit.com` (production) or `https://api-testnet.bybit.com` (testnet)

**Request Parameters:**
```json
{
    "tokenId": "USDT",      // Required: Token ID (e.g., "USDT", "ETH", "BTC")
    "currencyId": "RUB",    // Required: Currency ID (e.g., "RUB", "USD", "EUR")
    "side": "0",            // Required: "0" = buy, "1" = sell
    "page": "1",            // Optional: Page number (default: 1)
    "size": "10"            // Optional: Page size (default: 10)
}
```

**Response includes:**
- `price`: Ad price
- `lastQuantity`: Available token quantity
- `minAmount`: Minimum transaction amount
- `maxAmount`: Maximum transaction amount
- `payments`: Payment method IDs
- `nickName`: Trader nickname
- `recentOrderNum`: Recent order count
- `recentExecuteRate`: Recent completion rate

### 2. Official Python SDK - bybit_p2p

Bybit provides an official Python SDK specifically for P2P operations:

**Installation:**
```bash
pip install bybit-p2p
```

**Usage Example:**
```python
from bybit_p2p import P2P

# Initialize client
api = P2P(
    testnet=False,  # Use True for testnet
    api_key="your_api_key",
    api_secret="your_api_secret"
)

# Get online ads (P2P rates)
ads = api.get_online_ads(
    tokenId="USDT",
    currencyId="RUB",
    side="0"  # 0 = buy (looking at sell orders)
)

# Process results
for ad in ads['result']['items']:
    print(f"Price: {ad['price']}, Amount: {ad['lastQuantity']}")
```

### 3. Using pybit (General Bybit SDK)

The main Bybit Python SDK (pybit) also supports P2P operations:

**Installation:**
```bash
pip install pybit
```

**Note:** The pybit SDK focuses more on trading operations. For P2P-specific operations, the bybit_p2p SDK is recommended.

## Key Differences from Current Implementation

### 1. Authentication
- **Current**: Uses `api2.bybit.com` with custom headers
- **Official V5**: Uses standard Bybit authentication with HMAC signatures
- **SDK**: Handles authentication automatically

### 2. Endpoint Structure
- **Current**: `/fiat/otc/item/online`
- **Official V5**: `/v5/p2p/item/online`

### 3. Request Method
- Both use POST requests with JSON body

### 4. Response Format
- Similar structure but field names may vary slightly
- Official API uses snake_case for some fields

## Implementation Recommendations

### 1. Update Python Wrapper

Update the existing `bybit_wrapper.py` to include a method for fetching P2P rates:

```python
def get_p2p_rates(self, token_id: str = "USDT", currency_id: str = "RUB", 
                  payment_methods: List[str] = None, page: int = 1, size: int = 10) -> Dict[str, Any]:
    """
    Get P2P rates from online advertisements
    
    Args:
        token_id: Cryptocurrency token (default: USDT)
        currency_id: Fiat currency (default: RUB)
        payment_methods: Optional list of payment method IDs to filter
        page: Page number for pagination
        size: Number of results per page
    
    Returns:
        Dictionary with P2P advertisement data including rates
    """
    import requests
    
    body = {
        "tokenId": token_id,
        "currencyId": currency_id,
        "side": "0",  # 0 = buy (we're looking at sell orders)
        "page": str(page),
        "size": str(size)
    }
    
    headers = self._get_auth_headers("/v5/p2p/item/online", {}, "POST", body)
    
    response = requests.post(
        f"{self.base_url}/v5/p2p/item/online",
        headers=headers,
        json=body
    )
    
    if response.status_code == 200:
        data = response.json()
        if data.get('ret_code') == 0:
            return data.get('result', {})
        else:
            raise Exception(f"API error: {data.get('ret_msg')}")
    else:
        raise Exception(f"HTTP error: {response.status_code}")
```

### 2. Rust Integration

Update the Rust rate fetcher to use the official V5 endpoint:

```rust
// In rate_fetcher.rs
pub async fn fetch_p2p_trades_v5(&self, page: u32) -> Result<P2PTradeResponse> {
    let url = format!("{}/v5/p2p/item/online", self.base_url);
    
    let payload = serde_json::json!({
        "tokenId": "USDT",
        "currencyId": "RUB",
        "side": "0",  // 0 = buy
        "page": page.to_string(),
        "size": "10"
    });
    
    // Use authenticated request with proper headers
    // Implementation would need to include HMAC signature generation
}
```

### 3. Using Official SDK

For the most reliable implementation, consider using the official bybit_p2p SDK:

```python
# Add to bybit_wrapper.py
def get_p2p_rates_sdk(self) -> List[Dict[str, Any]]:
    """Get P2P rates using official SDK"""
    try:
        from bybit_p2p import P2P
        
        # Initialize P2P client
        p2p_client = P2P(
            testnet=self.testnet,
            api_key=self.api_key,
            api_secret=self.api_secret
        )
        
        # Get online ads
        response = p2p_client.get_online_ads(
            tokenId="USDT",
            currencyId="RUB",
            side="0"
        )
        
        if response.get('ret_code') == 0:
            return response.get('result', {}).get('items', [])
        else:
            raise Exception(f"P2P API error: {response.get('ret_msg')}")
    except ImportError:
        raise Exception("bybit_p2p SDK not installed. Run: pip install bybit-p2p")
```

## Advantages of Official API

1. **Stability**: Official endpoints are less likely to change
2. **Authentication**: Standard Bybit authentication flow
3. **Support**: Official documentation and support available
4. **Rate Limits**: Clear rate limit policies
5. **SDK Support**: Official Python SDK handles complexities

## Migration Path

1. **Phase 1**: Add official API methods alongside existing implementation
2. **Phase 2**: Test and compare results between methods
3. **Phase 3**: Gradually migrate to official API
4. **Phase 4**: Remove dependency on unofficial endpoints

## Additional P2P Endpoints Available

The official P2P API also provides:
- `/v5/p2p/item/create` - Create advertisements
- `/v5/p2p/order/list` - Get order list
- `/v5/p2p/order/detail` - Get order details
- `/v5/p2p/order/confirm-payment` - Confirm payment
- `/v5/p2p/order/release` - Release funds
- `/v5/p2p/user/personal/info` - Get user info

## Rate Limiting

- Official API rate limits are typically 10-20 requests per second
- Use the recv_window parameter to handle time synchronization issues
- Implement exponential backoff for rate limit errors

## Conclusion

The official Bybit V5 P2P API provides a more stable and supported way to fetch P2P rates compared to using internal/undocumented endpoints. The official Python SDK (bybit_p2p) makes integration even easier by handling authentication and request formatting automatically.