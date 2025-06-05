# WebSocket API Documentation

## Connection

Connect to the WebSocket endpoint at: `ws://localhost:3000/ws`

## Message Format

All messages are JSON-encoded with the following base structure:

```json
{
    "type": "message_type",
    "data": { ... }
}
```

## Client -> Server Messages

### Subscribe to Updates

```json
{
    "type": "subscribe",
    "data": {
        "channels": ["orders", "transactions", "metrics"]
    }
}
```

### Unsubscribe from Updates

```json
{
    "type": "unsubscribe",
    "data": {
        "channels": ["orders"]
    }
}
```

### Request Data

```json
{
    "type": "request",
    "data": {
        "resource": "orders", // "orders", "transactions", "metrics", "status"
        "filters": {
            "status": "active",
            "limit": 10
        }
    }
}
```

### Admin Commands (Requires Authentication)

```json
{
    "type": "admin",
    "data": {
        "token": "admin_token_here",
        "command": "approve_transaction",
        "params": {
            "transaction_id": "123456"
        }
    }
}
```

## Server -> Client Messages

### Connection Confirmation

```json
{
    "type": "connected",
    "data": {
        "message": "Connected to iTrader WebSocket",
        "version": "1.0.0"
    }
}
```

### Order Updates

```json
{
    "type": "order_update",
    "data": {
        "order": {
            "id": "order_id",
            "status": "pending",
            "amount": "10000.00",
            "currency": "RUB",
            "created_at": "2025-03-06T10:00:00Z"
        }
    }
}
```

### Transaction Updates

```json
{
    "type": "transaction_update",
    "data": {
        "transaction": {
            "id": "trans_id",
            "order_id": "order_id",
            "status": "waiting_receipt",
            "amount": "10000.00",
            "updated_at": "2025-03-06T10:01:00Z"
        }
    }
}
```

### Metrics Updates

```json
{
    "type": "metrics_update",
    "data": {
        "metrics": {
            "active_orders": 5,
            "pending_transactions": 2,
            "total_volume_24h": "1000000.00",
            "success_rate": 0.98
        }
    }
}
```

### Error Messages

```json
{
    "type": "error",
    "data": {
        "code": "INVALID_REQUEST",
        "message": "Invalid message format"
    }
}
```

## Authentication

Admin commands require authentication. Include the admin token in the message:

```json
{
    "type": "admin",
    "data": {
        "token": "your_admin_token",
        "command": "command_name",
        "params": { ... }
    }
}
```

## Available Admin Commands

- `approve_transaction`: Manually approve a pending transaction
- `reject_transaction`: Reject a transaction
- `set_balance`: Update Gate.io balance
- `toggle_auto_mode`: Enable/disable automatic transaction approval
- `get_system_status`: Get detailed system status

## Example Usage

### JavaScript Client

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
    // Subscribe to updates
    ws.send(JSON.stringify({
        type: 'subscribe',
        data: { channels: ['orders', 'transactions'] }
    }));
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log('Received:', message);
    
    switch(message.type) {
        case 'order_update':
            handleOrderUpdate(message.data.order);
            break;
        case 'transaction_update':
            handleTransactionUpdate(message.data.transaction);
            break;
    }
};
```