# Admin API Documentation

## Authentication

All admin endpoints require authentication using a bearer token. The token should be included in the `Authorization` header:

```
Authorization: Bearer YOUR_ADMIN_TOKEN
```

Alternatively, you can pass the token as a query parameter:

```
/admin/status?token=YOUR_ADMIN_TOKEN
```

## Endpoints

### POST /admin/approve

Approve a pending transaction.

**Request Body:**
```json
{
    "transaction_id": "12345"
}
```

**Response:**
```json
{
    "success": true,
    "message": "Transaction 12345 approved",
    "data": null
}
```

### POST /admin/reject

Reject/cancel a pending transaction.

**Request Body:**
```json
{
    "transaction_id": "12345"
}
```

**Response:**
```json
{
    "success": true,
    "message": "Transaction 12345 rejected",
    "data": null
}
```

### POST /admin/balance

Update the Gate.io balance.

**Request Body:**
```json
{
    "amount": "10000000.00"
}
```

**Response:**
```json
{
    "success": true,
    "message": "Balance updated to 10000000.00",
    "data": null
}
```

### POST /admin/auto-mode

Toggle automatic transaction approval mode.

**Request Body:**
```json
{
    "enabled": true
}
```

**Response:**
```json
{
    "success": true,
    "message": "Auto mode set to true",
    "data": null
}
```

### GET /admin/status

Get comprehensive system status.

**Response:**
```json
{
    "success": true,
    "message": "System status retrieved",
    "data": {
        "status": "running",
        "uptime_seconds": 3600,
        "auto_mode": true,
        "active_orders": 5,
        "version": "1.0.0",
        "gate_authenticated": true,
        "last_check": "2025-03-06T10:00:00Z"
    }
}
```

### GET /admin/transactions

Get transaction history with optional filters.

**Query Parameters:**
- `status` (optional): Filter by transaction status
- `limit` (optional): Number of results to return (default: 50)

**Example:**
```
GET /admin/transactions?status=pending&limit=10
```

**Response:**
```json
{
    "success": true,
    "message": "Transactions retrieved",
    "data": {
        "transactions": [
            {
                "id": "12345",
                "order_id": "order_123",
                "amount": "10000.00",
                "status": "pending",
                "created_at": "2025-03-06T10:00:00Z"
            }
        ],
        "total": 1,
        "limit": 10,
        "filters": {
            "status": "pending"
        }
    }
}
```

### GET /admin/logs

Retrieve system logs.

**Query Parameters:**
- `level` (optional): Log level filter (debug, info, warn, error) - default: "info"
- `limit` (optional): Number of log entries to return (default: 100)

**Example:**
```
GET /admin/logs?level=error&limit=50
```

**Response:**
```json
{
    "success": true,
    "message": "Logs retrieved",
    "data": {
        "logs": [
            {
                "timestamp": "2025-03-06T10:00:00Z",
                "level": "error",
                "message": "Failed to process transaction",
                "context": {
                    "transaction_id": "12345"
                }
            }
        ],
        "total": 1,
        "limit": 50,
        "level": "error"
    }
}
```

## Error Responses

All endpoints return consistent error responses:

**401 Unauthorized:**
```json
{
    "error": "Invalid admin token"
}
```

**400 Bad Request:**
```json
{
    "success": false,
    "message": "Invalid request format",
    "data": null
}
```

**500 Internal Server Error:**
```json
{
    "success": false,
    "message": "Internal server error: [details]",
    "data": null
}
```

## Example Usage

### Using cURL

```bash
# Get system status
curl -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
     http://localhost:3000/admin/status

# Approve transaction
curl -X POST \
     -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"transaction_id":"12345"}' \
     http://localhost:3000/admin/approve

# Toggle auto mode
curl -X POST \
     -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"enabled":true}' \
     http://localhost:3000/admin/auto-mode
```

### Using JavaScript

```javascript
const adminToken = 'YOUR_ADMIN_TOKEN';
const baseUrl = 'http://localhost:3000';

// Get system status
async function getSystemStatus() {
    const response = await fetch(`${baseUrl}/admin/status`, {
        headers: {
            'Authorization': `Bearer ${adminToken}`
        }
    });
    return response.json();
}

// Approve transaction
async function approveTransaction(transactionId) {
    const response = await fetch(`${baseUrl}/admin/approve`, {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${adminToken}`,
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ transaction_id: transactionId })
    });
    return response.json();
}
```

## Security Notes

1. **Token Storage**: Never expose the admin token in client-side code or version control
2. **HTTPS**: Always use HTTPS in production to prevent token interception
3. **Token Rotation**: Regularly rotate admin tokens for security
4. **Access Logs**: Monitor admin API access logs for suspicious activity
5. **Rate Limiting**: Consider implementing rate limiting on admin endpoints