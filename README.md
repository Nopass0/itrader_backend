# iTrader Backend - P2P Trading Automation System

Automated P2P trading system integrating Gate.io and Bybit exchanges with intelligent transaction processing.

## Quick Start

```bash
# Install dependencies
cargo build --release

# Set up environment
cp .env.example .env
# Edit .env with your database and Redis URLs

# Configure accounts (interactive menu)
./run.sh --settings

# Or import accounts from CSV
./scripts/batch_import_accounts.sh gate data/gate_accounts.csv
./scripts/batch_import_accounts.sh bybit data/bybit_accounts.csv

# Run the application
./run.sh          # Manual mode
./run.sh --auto   # Automatic mode

# Run tests
./test.sh
```

## System Architecture

### Core Components
- **Gate.io Integration**: Monitors and processes P2P transactions
- **Bybit Integration**: Creates corresponding sell advertisements using official Bybit P2P SDK
  - Uses Rust-to-Python bridge (PyO3) for seamless integration
  - Leverages official Bybit Python SDK for P2P operations
  - Maintains type safety and error handling across language boundaries
- **Transaction Pipeline**: Gate → Accept → Calculate Rate → Create Bybit Ad
- **Account Management**: Multi-account support with JSON storage
- **WebSocket API**: Real-time monitoring and control

### Key Features
- **Multi-Account Support**: Manage multiple Gate.io and Bybit accounts
- **Auto-accepts pending transactions** (status 4)
- **Interactive Account Manager**: Add/edit/delete accounts via menu
- **Batch Import**: Import accounts from CSV files
- Mock rate calculation (103.50 RUB/USDT)
- OCR receipt validation
- No encryption - credentials stored in plain JSON
- Rate limiting (240 requests/minute for Gate API)

## AI Development Guidelines

### 1. Test-Driven Development Approach

When implementing new features, ALWAYS follow this pattern:

```bash
# Step 1: Write a failing test
cargo test feature_name -- --nocapture

# Step 2: Implement minimal code to pass
# Step 3: Run test again
cargo test feature_name -- --nocapture

# Step 4: Refactor if needed
# Step 5: Run ALL tests to ensure nothing broke
./test.sh
```

### 2. Feature Implementation Checklist

For EVERY new feature:

- [ ] **Understand existing code** - Use grep/search to find similar implementations
- [ ] **Write unit test first** - Test the specific functionality in isolation
- [ ] **Write integration test** - Test how it works with other components
- [ ] **Implement feature** - Start with minimal working code
- [ ] **Add error handling** - Handle all possible failure cases
- [ ] **Add logging** - Use appropriate log levels (debug, info, warn, error)
- [ ] **Update documentation** - Add comments and update relevant docs
- [ ] **Test edge cases** - Empty data, invalid input, network failures
- [ ] **Performance test** - Ensure no degradation for high-volume operations

### 3. Testing Strategy

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_specific_function() {
        // Arrange
        let input = prepare_test_data();
        
        // Act
        let result = function_to_test(input).await;
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }
}
```

#### Integration Tests
```bash
# Test Gate.io integration
cargo test --test gate_tests -- --nocapture

# Test Bybit integration  
cargo test --test bybit_tests -- --nocapture

# Test transaction processing
cargo test --test transaction_service_tests -- --nocapture
```

#### Manual Testing
```bash
# Start in development mode with verbose logging
RUST_LOG=debug ./run.sh

# Monitor specific module
RUST_LOG=itrader_backend::gate=debug ./run.sh

# Test specific transaction
./scripts/test_transaction.sh TRANSACTION_ID
```

### 4. Common Development Tasks

#### Adding New API Endpoint
1. Define route in `src/api/routes.rs`
2. Implement handler in `src/api/handlers/`
3. Add request/response models
4. Write unit test for handler
5. Write integration test for full flow
6. Test with curl/Postman
7. Update API documentation

#### Adding New Exchange Integration
1. Create new module in `src/exchanges/`
2. Define models in `models.rs`
3. Implement client with rate limiting
4. Add authentication logic
5. Write mock tests first
6. Test with real API (use test account)
7. Add to orchestrator pipeline
8. Update configuration

#### Modifying Transaction Processing
1. Understand current flow in `src/core/orchestrator.rs`
2. Write test for new behavior
3. Modify processing logic
4. Test with different transaction states
5. Test error scenarios
6. Verify idempotency
7. Check database consistency

### 5. Debugging Techniques

```bash
# Enable debug logging for specific module
RUST_LOG=itrader_backend::module_name=debug ./run.sh

# Test database queries
cargo test db_tests -- --nocapture

# Check API responses
curl -X GET http://localhost:3000/api/v1/transactions

# Monitor WebSocket events
wscat -c ws://localhost:3000/ws

# Inspect database state
psql -U postgres -d itrader -c "SELECT * FROM orders;"
```

### 6. Performance Optimization

Before optimizing:
1. **Measure first** - Use benchmarks to identify bottlenecks
2. **Profile the code** - Use tools like `perf` or `flamegraph`
3. **Test under load** - Simulate real-world conditions
4. **Monitor resources** - CPU, memory, database connections

```bash
# Run benchmarks
cargo bench

# Profile with flamegraph
cargo flamegraph --bin itrader-backend

# Load test
artillery quick --count 100 --num 10 http://localhost:3000/api/v1/health
```

### 7. Error Handling Best Practices

```rust
// Always use Result type
pub async fn process_transaction(id: &str) -> Result<Transaction> {
    // Log context
    info!("Processing transaction: {}", id);
    
    // Handle errors gracefully
    let tx = get_transaction(id).await
        .map_err(|e| {
            error!("Failed to get transaction {}: {}", id, e);
            e
        })?;
    
    // Validate before processing
    validate_transaction(&tx)?;
    
    // Return success
    Ok(tx)
}
```

### 8. Code Review Checklist

Before committing:
- [ ] All tests pass (`./test.sh`)
- [ ] No compiler warnings
- [ ] Code follows Rust idioms
- [ ] Error messages are helpful
- [ ] Logging is appropriate
- [ ] No hardcoded values
- [ ] Configuration is documented
- [ ] Breaking changes noted

## Bybit P2P Integration

### Python Bridge Architecture

The Bybit integration uses a Rust-to-Python bridge to leverage the official Bybit P2P SDK:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Rust Code     │────▶│   PyO3 Bridge   │────▶│  Bybit Python   │
│  (Type Safe)    │     │   (FFI Layer)   │     │     SDK         │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

#### Key Components:
- **python_bridge.rs**: Core PyO3 integration managing Python interpreter
- **p2p_python.rs**: Python-based P2P client implementation
- **python_rate_fetcher.rs**: Rate fetching using Python SDK
- **bybit_wrapper.py**: Python wrapper around official Bybit SDK

#### Benefits:
- Uses official Bybit SDK ensuring compatibility
- Automatic handling of authentication and signing
- Built-in rate limiting and retry logic
- Access to latest P2P features without reimplementation

#### Usage:
```rust
// The API remains the same whether using native or Python implementation
let client = BybitP2PClient::new(config);
let ads = client.get_advertisements(params).await?;
```

The implementation is selected at compile time using the `python-sdk` feature flag.

## Configuration

### Environment Variables
```bash
DATABASE_URL=postgres://user:pass@localhost/itrader
REDIS_URL=redis://localhost:6379
RUST_LOG=info
```

### Account Configuration

#### Interactive Account Manager
```bash
./run.sh --settings
```

Features:
- Add/Edit/Delete Gate.io accounts
- Add/Edit/Delete Bybit accounts  
- Import/Export configurations
- View account statistics

#### Manual Configuration
Edit `data/accounts.json`:
```json
{
  "gate_accounts": [{
    "id": 1,
    "email": "user@example.com",
    "password": "password",
    "balance": 10000000.0,
    "status": "active"
  }],
  "bybit_accounts": [{
    "id": 1,
    "account_name": "user",
    "api_key": "key",
    "api_secret": "secret",
    "active_ads": 0,
    "status": "available"
  }]
}
```

#### Batch Import
```bash
# Import Gate.io accounts from CSV
./scripts/batch_import_accounts.sh gate accounts.csv

# Import Bybit accounts from CSV  
./scripts/batch_import_accounts.sh bybit bybit.csv
```

See [ACCOUNT_MANAGEMENT.md](./ACCOUNT_MANAGEMENT.md) for detailed documentation.

## Troubleshooting

### Common Issues

1. **Database connection failed**
   - Check DATABASE_URL is correct
   - Ensure PostgreSQL is running
   - Run migrations: `sqlx migrate run`

2. **Rate limit exceeded**
   - Reduce concurrent requests
   - Check rate limiter configuration
   - Add delays between requests

3. **Transaction not processing**
   - Check account balance
   - Verify credentials are correct
   - Check logs for specific errors

### Debug Commands
```bash
# Check system health
curl http://localhost:3000/health

# List pending transactions
cargo run --bin gate_list_pending

# Test Gate.io connection
cargo run --bin gate_test_login

# Process specific transaction
cargo run --bin gate_approve_transaction TRANSACTION_ID
```

## Development Workflow

1. **Create feature branch**
   ```bash
   git checkout -b feature/new-feature
   ```

2. **Develop with TDD**
   - Write failing test
   - Implement feature
   - Make test pass
   - Refactor

3. **Test thoroughly**
   ```bash
   ./test.sh
   cargo clippy
   cargo fmt
   ```

4. **Commit with clear message**
   ```bash
   git add .
   git commit -m "feat: add new feature with tests"
   ```

5. **Push and create PR**
   ```bash
   git push origin feature/new-feature
   ```

## License

Proprietary - All rights reserved