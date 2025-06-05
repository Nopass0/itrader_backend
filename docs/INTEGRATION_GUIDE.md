# iTrader Backend Integration Guide

## Overview

The iTrader backend system automates cryptocurrency trading between Gate.io and Bybit P2P platforms. This guide covers how to integrate all components for a complete working system.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Gate.io API   │────▶│  iTrader Core   │────▶│  Bybit P2P API  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │  Email Monitoring   │
                    │   (Receipt OCR)     │
                    └─────────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   WebSocket API     │
                    │  Admin Dashboard    │
                    └─────────────────────┘
```

## Component Integration

### 1. Core Trading Loop

The main trading loop coordinates all components:

```rust
// src/core/orchestrator.rs
pub async fn start_trading_loop(state: Arc<AppState>) {
    // Balance updater (every 4 hours)
    let balance_task = update_balance_periodically(state.clone());
    
    // Transaction monitor (every 5 minutes)
    let monitor_task = monitor_transactions(state.clone());
    
    // Email receipt monitor
    let email_task = monitor_email_receipts(state.clone());
    
    // WebSocket server
    let ws_task = start_websocket_server(state.clone());
    
    tokio::select! {
        _ = balance_task => {},
        _ = monitor_task => {},
        _ = email_task => {},
        _ = ws_task => {},
    }
}
```

### 2. Transaction Flow

1. **Gate.io Order Detection**
   - Monitor new orders every 5 minutes
   - Check order status and amount

2. **Bybit P2P Ad Creation**
   - Create matching P2P advertisement
   - Use template messaging system

3. **Receipt Validation**
   - Monitor email for payment receipts
   - Extract PDF attachments
   - OCR validation of amount and details

4. **Transaction Approval**
   - Auto-approve if validation passes
   - Manual approval via admin API if needed

### 3. Email Integration

Configure email monitoring in `config/default.toml`:

```toml
[email]
imap_server = "imap.gmail.com"
imap_port = 993
username = "your-email@gmail.com"
password = "app-specific-password"
```

The email monitor will:
- Check for new emails every 30 seconds
- Filter emails from known banks
- Extract PDF receipts
- Process with OCR

### 4. OCR Setup

Install Tesseract OCR:

```bash
# Ubuntu/Debian
sudo apt-get install tesseract-ocr tesseract-ocr-rus tesseract-ocr-eng

# macOS
brew install tesseract tesseract-lang

# Verify installation
tesseract --version
```

### 5. WebSocket Client Integration

Connect to WebSocket for real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

// Subscribe to updates
ws.send(JSON.stringify({
    type: 'subscribe',
    data: { channels: ['orders', 'transactions'] }
}));

// Handle updates
ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    switch(msg.type) {
        case 'order_update':
            handleOrderUpdate(msg.data);
            break;
        case 'transaction_update':
            handleTransactionUpdate(msg.data);
            break;
    }
};
```

### 6. Admin Integration

Use the admin API for manual control:

```bash
# Set admin token
export ADMIN_TOKEN="your-secure-token"

# Check system status
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
     http://localhost:3000/admin/status

# Approve transaction manually
curl -X POST \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"transaction_id":"12345"}' \
     http://localhost:3000/admin/approve
```

## Complete Setup Example

1. **Environment Setup**

```bash
# .env file
DATABASE_URL=postgres://user:pass@localhost/itrader
ENCRYPTION_KEY=your-32-byte-encryption-key
ADMIN_TOKEN=your-secure-admin-token
RUST_LOG=info,itrader=debug
```

2. **Configuration**

```toml
# config/production.toml
[server]
host = "0.0.0.0"
port = 3000

[gate]
api_url = "https://api.gate.io"
ws_url = "wss://api.gate.io"
api_key = "your-api-key"
api_secret = "your-api-secret"
user_id = "your-user-id"

[bybit]
api_key = "your-api-key"
api_secret = "your-api-secret"

[email]
imap_server = "imap.gmail.com"
imap_port = 993
username = "receipts@yourdomain.com"
password = "app-password"

[trading]
auto_approve = true
min_amount = "1000.00"
max_amount = "100000.00"
```

3. **Database Setup**

```bash
# Run migrations
psql $DATABASE_URL < migrations/001_create_accounts.sql
psql $DATABASE_URL < migrations/002_create_orders.sql
psql $DATABASE_URL < migrations/003_create_pools.sql
```

4. **Start the System**

```bash
# Build and run
cargo build --release
./target/release/itrader-backend
```

## Monitoring & Maintenance

### Health Checks

```bash
# Check system health
curl http://localhost:3000/health

# Monitor logs
tail -f logs/itrader.log | grep -E "ERROR|WARN"
```

### Performance Tuning

1. **Database Optimization**
   - Add indexes on frequently queried columns
   - Regular vacuum and analyze

2. **Rate Limiting**
   - Adjust Gate.io rate limits based on usage
   - Monitor API quota usage

3. **OCR Accuracy**
   - Train Tesseract with receipt samples
   - Adjust preprocessing for better accuracy

## Troubleshooting

### Common Issues

1. **Gate.io Authentication Failures**
   - Check cookie expiration
   - Verify API credentials
   - Check rate limits

2. **Email Connection Issues**
   - Verify IMAP settings
   - Check firewall rules
   - Enable "less secure apps" or use app passwords

3. **OCR Failures**
   - Check Tesseract installation
   - Verify language packs installed
   - Check PDF quality

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug ./target/release/itrader-backend
```

## Security Considerations

1. **API Keys**: Store securely, never commit to git
2. **Admin Token**: Use strong, unique tokens
3. **Database**: Use SSL connections in production
4. **Email**: Use app-specific passwords
5. **Encryption**: Rotate encryption keys regularly

## Deployment

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y \
    tesseract-ocr \
    tesseract-ocr-rus \
    tesseract-ocr-eng \
    ca-certificates
COPY --from=builder /app/target/release/itrader-backend /usr/local/bin/
CMD ["itrader-backend"]
```

### Kubernetes Deployment

See `k8s/` directory for Kubernetes manifests.

## Support

For issues or questions:
1. Check logs for error details
2. Review this documentation
3. Check component-specific docs in `docs/`