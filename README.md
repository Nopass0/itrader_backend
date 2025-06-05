# iTrader Backend

P2P cryptocurrency trading automation system for Gate.io and Bybit.

## Features

- Automated Gate.io transaction monitoring
- Bybit P2P advertisement management
- AI-powered customer communication
- OCR receipt processing
- Real-time WebSocket API
- Rate limiting and session management

## Quick Start

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. Install system dependencies:
   ```bash
   ./install-deps.sh
   ```
   
   This will install:
   - OpenSSL development libraries
   - PostgreSQL
   - Redis
   - Tesseract OCR with Russian language pack

3. Run development server (auto-setup):
   ```bash
   ./dev.sh
   ```
   
   This will automatically:
   - Create `.env` with default values
   - Create database if needed
   - Run migrations
   - Start with hot reload

4. Run the auto-trader:
   ```bash
   # Manual mode (default - safer, requires confirmation)
   ./start.sh
   
   # Automatic mode (auto-confirms all transactions)
   ./start.sh --auto
   ```

## Testing

```bash
# Run specific tests
./test.sh gate-auth      # Test Gate.io authentication
./test.sh gate-tx        # Test Gate.io transactions
./test.sh bybit-auth     # Test Bybit authentication
./test.sh bybit-ads      # Test Bybit advertisements

# Run all tests
./test.sh all
```

See [TESTING.md](TESTING.md) for detailed testing guide.

## API Documentation

WebSocket endpoint: `ws://localhost:8080/ws`
REST API: `http://localhost:8080/api/v1`

## Documentation

- [Implementation Details](IMPLEMENTATION.md)
- [Testing Guide](TESTING.md)
- [Project Specification](PROJECT.md)