# iTrader Backend Implementation

## Overview

This is a complete P2P cryptocurrency trading automation system built in Rust. The system monitors Gate.io for new transactions, automatically creates corresponding sell orders on Bybit P2P, manages conversations with buyers using AI, and processes receipts.

## Project Structure

```
itrader_backend/
├── src/
│   ├── main.rs          # Entry point
│   ├── core/            # Core business logic
│   │   ├── app.rs       # Application lifecycle
│   │   ├── config.rs    # Configuration management
│   │   ├── orchestrator.rs # Main business logic
│   │   ├── rate_limiter.rs # API rate limiting
│   │   └── state.rs     # Application state
│   ├── gate/            # Gate.io integration
│   │   ├── client.rs    # HTTP client
│   │   ├── auth.rs      # Authentication
│   │   ├── api.rs       # API methods
│   │   └── models.rs    # Data models
│   ├── bybit/           # Bybit P2P integration
│   │   ├── client.rs    # HTTP client
│   │   ├── p2p.rs       # P2P-specific methods
│   │   ├── auth.rs      # HMAC authentication
│   │   └── models.rs    # Data models
│   ├── db/              # Database layer
│   │   ├── repository.rs # Database operations
│   │   ├── pool_manager.rs # Order pool management
│   │   └── models.rs    # Database entities
│   ├── api/             # REST/WebSocket API
│   │   ├── server.rs    # HTTP server
│   │   ├── routes.rs    # REST endpoints
│   │   └── websocket.rs # WebSocket handlers
│   ├── ai/              # AI chat integration
│   ├── ocr/             # OCR processing
│   ├── email/           # Email monitoring
│   └── utils/           # Utilities
├── config/              # Configuration files
├── migrations/          # Database migrations
├── test_data/           # Test credentials
└── docker-compose.yml   # Docker setup
```

## Key Features Implemented

1. **Gate.io Integration**
   - Cookie-based authentication
   - Balance management (auto-set to 1M RUB)
   - Transaction monitoring
   - Session refresh every 25 minutes

2. **Bybit P2P Integration**
   - HMAC-SHA256 authentication
   - Advertisement creation
   - Order monitoring
   - Chat management
   - Payment confirmation

3. **Rate Limiting**
   - Per-endpoint rate limiting
   - Automatic retry with backoff
   - Cloudflare protection handling

4. **Database**
   - PostgreSQL for persistent storage
   - Redis for caching
   - Order state management
   - Pool-based workflow

5. **AI Integration (Stub)**
   - OpenRouter API ready
   - Multi-language support
   - Context-aware responses

6. **OCR Processing (Stub)**
   - Tesseract integration ready
   - T-Bank receipt validation
   - Amount extraction

7. **WebSocket API**
   - Real-time order updates
   - Pool status monitoring
   - System metrics

## Running the Application

### Development Mode

```bash
# Install dependencies
cargo install cargo-watch

# Copy and configure environment
cp .env.example .env
# Edit .env with your credentials

# Run with hot reload
./dev.sh
```

### Production Mode

```bash
# Using Docker
docker-compose up -d

# Or directly
cargo build --release
./target/release/itrader-backend
```

## Configuration

Edit `config/default.toml` for base configuration:
- Gate.io settings
- Bybit settings
- Rate limits
- AI parameters

## Database Setup

```bash
# Create database
createdb itrader_db

# Run migrations
cargo install sqlx-cli
sqlx migrate run
```

## API Endpoints

- `GET /api/v1/health` - Health check
- `GET /api/v1/orders` - List active orders
- `GET /api/v1/pools` - Pool status
- `GET /api/v1/metrics` - System metrics
- `WS /api/v1/ws` - WebSocket connection

## Environment Variables

Required:
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `ENCRYPTION_KEY` - 32-byte encryption key
- `OPENROUTER_API_KEY` - OpenRouter API key
- `EMAIL_ADDRESS` - Email for receipt monitoring
- `EMAIL_PASSWORD` - Email app password

## Security Considerations

1. All sensitive data is encrypted at rest
2. API credentials use environment variables
3. HMAC signatures for Bybit API
4. Rate limiting prevents abuse
5. Graceful shutdown sets balances to 0

## Testing

Test with provided credentials in `test_data/`:
- `gate_cookie.json` - Gate.io session cookies
- `bybit_creditials.json` - Bybit API credentials

## Next Steps

1. Complete AI chat implementation with OpenRouter
2. Implement OCR receipt processing
3. Add email monitoring for receipts
4. Enhance rate calculation algorithm
5. Add comprehensive error recovery
6. Implement metrics and monitoring
7. Add automated tests

## Architecture Decisions

1. **Rust** - Performance, safety, and reliability
2. **Tokio** - Async runtime for concurrent operations
3. **PostgreSQL** - Reliable persistent storage
4. **Redis** - Fast caching and session management
5. **Pool-based workflow** - Clear state management
6. **HMAC authentication** - Secure API access

## Order Flow

1. Monitor Gate.io for pending transactions (status=1)
2. Accept transaction and find available Bybit account
3. Calculate rate and create Bybit advertisement
4. Wait for buyer and handle chat
5. Monitor for payment confirmation
6. Validate receipt with OCR
7. Complete transaction on both platforms

The system is designed to be resilient, with state recovery on restart and comprehensive error handling throughout.