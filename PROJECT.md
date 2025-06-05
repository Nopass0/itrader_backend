# Complete P2P Trading Bot Development Guide

## 🚨 CRITICAL: Test Data Information
**IMPORTANT**: In the `test_data` folder, you will find:
- Cookie data for one Gate.io account
- API key and secret for one Bybit account

**ALWAYS** use these test credentials to verify API functionality before implementing any features. Make test requests to ensure they are working properly.

## 📋 Project Overview

You are building a P2P cryptocurrency trading automation system that:
1. Monitors Gate.io (panel.gate.cx) for new transactions
2. Automatically creates corresponding sell orders on Bybit P2P
3. Manages conversations with buyers using AI
4. Processes receipts and completes transactions

## 🏗️ Architecture Overview

### Core Components:
1. **Gate.io Client** - Session management, transaction monitoring
2. **Bybit P2P Module** - Advertisement creation, order management
3. **Rate Limiter** - Centralized request management with Cloudflare bypass
4. **AI Chat Manager** - OpenRouter integration for customer communication
5. **OCR Module** - Receipt processing and validation
6. **WebSocket API** - Real-time status updates

### Data Flow:
1. Gate account logs in → Saves cookies → Sets balance to 1,000,000 RUB
2. Monitors for new transactions (status = 1)
3. Accepts transaction → Finds available Bybit account
4. Creates advertisement with calculated rate
5. Waits for buyer → AI handles conversation
6. Receives receipt → OCR validation → Complete transaction

## 🛠️ Technology Stack

### Language: Rust (Edition 2021, Version 1.75+)

### Core Dependencies:
```toml
[package]
name = "p2p-trading-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async Runtime
tokio = { version = "1.35", features = ["full"] }

# Web Framework
axum = { version = "0.7", features = ["ws", "json"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "json", "chrono"] }

# Cache
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# HTTP Client
reqwest = { version = "0.11", features = ["json", "cookies", "gzip", "rustls-tls"] }
reqwest-middleware = "0.2"

# WebSocket
tokio-tungstenite = { version = "0.21", features = ["rustls-tls-webpki-roots"] }

# Authentication
jsonwebtoken = "9.2"
hmac = "0.12"
sha2 = "0.10"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Rate Limiting
governor = "0.6"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Email
imap = "3.0.0-alpha.13"
mail-parser = "0.9"

# OCR
tesseract = "0.15"

# AI
async-openai = "0.18"

# Utilities
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
config = "0.13"
dotenv = "0.15"

# Encryption
aes-gcm = "0.10"
argon2 = "0.5"

[dev-dependencies]
wiremock = "0.5"
proptest = "1.4"
```

## 📁 Project Structure

```
p2p-trading-bot/
├── Cargo.toml
├── .env.example
├── README.md
├── test_data/                    # TEST CREDENTIALS HERE
│   ├── gate_cookies.json         # Gate.io test account cookies
│   └── bybit_credentials.json    # Bybit API key and secret
├── config/
│   ├── default.toml             # Default configuration
│   └── production.toml          # Production overrides
├── migrations/
│   ├── 001_create_accounts.sql
│   ├── 002_create_orders.sql
│   └── 003_create_pools.sql
├── src/
│   ├── main.rs                  # Entry point with graceful shutdown
│   ├── lib.rs                   # Library exports
│   ├── api/
│   │   ├── mod.rs              # API module exports
│   │   ├── server.rs           # Axum server setup
│   │   ├── routes.rs           # REST endpoints
│   │   └── websocket.rs        # WebSocket handlers
│   ├── gate/
│   │   ├── mod.rs              # Gate module exports
│   │   ├── client.rs           # HTTP client with cookie management
│   │   ├── auth.rs             # Login and session refresh
│   │   ├── models.rs           # Request/Response types
│   │   └── api.rs              # API method implementations
│   ├── bybit/
│   │   ├── mod.rs              # Bybit module exports
│   │   ├── client.rs           # REST & WebSocket clients
│   │   ├── p2p.rs              # P2P-specific endpoints
│   │   ├── auth.rs             # HMAC signature generation
│   │   └── models.rs           # P2P order types
│   ├── core/
│   │   ├── mod.rs              # Core module exports
│   │   ├── orchestrator.rs     # Main business logic
│   │   ├── rate_limiter.rs     # Centralized rate limiting
│   │   ├── state.rs            # Application state
│   │   └── config.rs           # Configuration loading
│   ├── ai/
│   │   ├── mod.rs              # AI module exports
│   │   ├── chat.rs             # OpenRouter integration
│   │   ├── prompts.rs          # System prompts
│   │   └── conversation.rs     # Conversation management
│   ├── ocr/
│   │   ├── mod.rs              # OCR module exports
│   │   ├── processor.rs        # Tesseract integration
│   │   └── validators.rs       # Receipt validation
│   ├── email/
│   │   ├── mod.rs              # Email module exports
│   │   ├── imap_client.rs      # IMAP connection
│   │   ├── parser.rs           # Email parsing
│   │   └── monitor.rs          # Receipt monitoring
│   ├── db/
│   │   ├── mod.rs              # Database module exports
│   │   ├── models.rs           # Database entities
│   │   ├── repository.rs       # Database operations
│   │   └── pool_manager.rs     # Pool state management
│   └── utils/
│       ├── mod.rs              # Utilities exports
│       ├── error.rs            # Custom error types
│       ├── crypto.rs           # Encryption utilities
│       └── retry.rs            # Retry logic
└── tests/
    ├── integration/
    └── common/
```

## 📊 Database Schema

```sql
-- Gate.io accounts
CREATE TABLE gate_accounts (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_encrypted TEXT NOT NULL,
    cookies JSONB,
    last_auth TIMESTAMP WITH TIME ZONE,
    balance DECIMAL(20, 2) DEFAULT 0,
    status VARCHAR(50) DEFAULT 'inactive',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Bybit accounts
CREATE TABLE bybit_accounts (
    id SERIAL PRIMARY KEY,
    account_name VARCHAR(255) UNIQUE NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    api_secret_encrypted TEXT NOT NULL,
    active_ads INTEGER DEFAULT 0,
    status VARCHAR(50) DEFAULT 'available',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Orders tracking
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gate_transaction_id VARCHAR(255) UNIQUE NOT NULL,
    bybit_order_id VARCHAR(255),
    gate_account_id INTEGER REFERENCES gate_accounts(id),
    bybit_account_id INTEGER REFERENCES bybit_accounts(id),
    amount DECIMAL(20, 8) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    fiat_currency VARCHAR(10) NOT NULL,
    rate DECIMAL(10, 4) NOT NULL,
    total_fiat DECIMAL(20, 2) NOT NULL,
    status VARCHAR(50) NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Order pools for state recovery
CREATE TABLE order_pools (
    id SERIAL PRIMARY KEY,
    pool_type VARCHAR(50) NOT NULL, -- 'pending', 'active', 'completed'
    order_id UUID REFERENCES orders(id),
    data JSONB NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- AI conversation history
CREATE TABLE ai_conversations (
    id SERIAL PRIMARY KEY,
    order_id UUID REFERENCES orders(id),
    messages JSONB DEFAULT '[]',
    customer_language VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);
CREATE INDEX idx_pools_pool_type ON order_pools(pool_type);
CREATE INDEX idx_gate_accounts_status ON gate_accounts(status);
```

## 🔧 Configuration

### config/default.toml
```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgresql://postgres:password@localhost/p2p_trading"
max_connections = 10
min_connections = 2

[redis]
url = "redis://localhost:6379"
pool_size = 10

[gate]
base_url = "https://panel.gate.cx/api/v1"
session_refresh_interval = 1500  # 25 minutes
balance_check_interval = 60      # 1 minute
target_balance = 1000000         # 1M RUB
min_balance = 300000             # 300K RUB
request_timeout = 30             # seconds

[bybit]
rest_url = "https://api.bybit.com"
ws_url = "wss://stream.bybit.com"
p2p_api_version = "v5"
max_ads_per_account = 2

[ai]
openrouter_api_key = "${OPENROUTER_API_KEY}"
model = "anthropic/claude-3-sonnet"
max_tokens = 1000
temperature = 0.7
response_delay_min = 15
response_delay_max = 45

[rate_limits]
gate_requests_per_minute = 30
bybit_requests_per_minute = 50
default_burst_size = 10

[email]
imap_server = "imap.gmail.com"
imap_port = 993
email = "${EMAIL_ADDRESS}"
password = "${EMAIL_PASSWORD}"
check_interval = 30  # seconds

[ocr]
tesseract_lang = "eng+rus"
confidence_threshold = 80

[monitoring]
metrics_port = 9090
health_check_interval = 30
```

## 💻 Core Implementation Guidelines

### 1. ALWAYS Document Your Code Thoroughly

```rust
/// Manages Gate.io account sessions and authentication.
///
/// This struct handles:
/// - Initial authentication with email/password
/// - Cookie persistence and session refresh
/// - Balance management (setting and checking)
/// - Transaction monitoring
///
/// # Example
/// ```
/// let manager = GateAccountManager::new(config);
/// manager.authenticate_account("user@example.com", "password").await?;
/// ```
pub struct GateAccountManager {
    /// Shared application state
    state: Arc<AppState>,
    /// HTTP client with cookie jar
    client: Client,
    /// Rate limiter for API requests
    rate_limiter: Arc<RateLimiter>,
}

impl GateAccountManager {
    /// Creates a new Gate account manager.
    ///
    /// # Arguments
    /// * `state` - Shared application state containing config and database
    ///
    /// # Returns
    /// A new instance of GateAccountManager
    pub fn new(state: Arc<AppState>) -> Self {
        // Implementation...
    }

    /// Authenticates a Gate.io account and saves cookies.
    ///
    /// # Arguments
    /// * `email` - Account email address
    /// * `password` - Account password (will be encrypted before storage)
    ///
    /// # Returns
    /// * `Ok(())` - Authentication successful, cookies saved
    /// * `Err(AppError)` - Authentication failed or network error
    ///
    /// # Errors
    /// - `AppError::Authentication` - Invalid credentials
    /// - `AppError::Network` - Connection failed
    /// - `AppError::RateLimit` - Too many requests
    pub async fn authenticate_account(
        &self,
        email: &str,
        password: &str,
    ) -> Result<()> {
        // Implementation...
    }
}
```

### 2. Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limited, retry after {retry_after} seconds")]
    RateLimit { retry_after: u64 },

    #[error("Invalid amount: {amount}, must be between {min} and {max}")]
    InvalidAmount { amount: Decimal, min: Decimal, max: Decimal },

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

// Always use Result type alias
pub type Result<T> = std::result::Result<T, AppError>;
```

### 3. Rate Limiter Implementation

```rust
/// Centralized rate limiter for all external API calls.
///
/// Uses token bucket algorithm to ensure we don't exceed API limits.
/// Automatically handles 429 responses with exponential backoff.
pub struct RateLimiter {
    /// Map of endpoint to rate limiter
    limiters: HashMap<String, Arc<Governor<NotKeyed, InMemoryState, DefaultClock>>>,
    /// Default rate limiter for unknown endpoints
    default_limiter: Arc<Governor<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiter {
    /// Checks rate limit and waits if necessary.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint identifier (e.g., "gate_transactions")
    ///
    /// # Example
    /// ```
    /// rate_limiter.check_and_wait("gate_transactions").await?;
    /// // Now safe to make request
    /// ```
    pub async fn check_and_wait(&self, endpoint: &str) -> Result<()> {
        let limiter = self.limiters
            .get(endpoint)
            .unwrap_or(&self.default_limiter);

        match limiter.check() {
            Ok(_) => {
                debug!("Rate limit OK for endpoint: {}", endpoint);
                Ok(())
            }
            Err(_) => {
                let wait_time = limiter.until_ready().await;
                warn!(
                    "Rate limited on endpoint '{}', waiting for {:?}",
                    endpoint, wait_time
                );
                Ok(())
            }
        }
    }
}
```

### 4. Gate.io API Client

```rust
/// Gate.io API client with cookie management and Cloudflare bypass.
///
/// IMPORTANT: Always test with credentials from test_data folder first!
pub struct GateClient {
    client: Client,
    base_url: String,
    cookies: Arc<Mutex<CookieJar>>,
    rate_limiter: Arc<RateLimiter>,
}

impl GateClient {
    /// Logs into Gate.io and saves session cookies.
    ///
    /// # Test First!
    /// Use test credentials from test_data/gate_cookies.json
    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResponse> {
        // Rate limit check
        self.rate_limiter.check_and_wait("gate_login").await?;

        let request_body = json!({
            "login": email,
            "password": password
        });

        let response = self.client
            .post(format!("{}/auth/basic/login", self.base_url))
            .headers(self.build_headers())
            .json(&request_body)
            .send()
            .await
            .context("Failed to send login request")?;

        // Handle Cloudflare challenges
        if response.status() == StatusCode::FORBIDDEN {
            warn!("Cloudflare challenge detected, implement bypass logic");
            return Err(AppError::CloudflareBlock);
        }

        // Extract and save cookies
        if let Some(cookies) = response.headers().get_all("set-cookie") {
            let mut jar = self.cookies.lock().await;
            for cookie in cookies {
                // Parse and store cookie
            }
        }

        let body: GateResponse<LoginResponse> = response
            .json()
            .await
            .context("Failed to parse login response")?;

        if !body.success {
            return Err(AppError::Authentication(
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(body.response.unwrap())
    }

    /// Builds standard headers for Gate.io requests.
    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".parse().unwrap());
        headers.insert("referer", "https://panel.gate.cx/".parse().unwrap());
        headers.insert("origin", "https://panel.gate.cx".parse().unwrap());
        headers.insert("accept", "application/json, text/plain, */*".parse().unwrap());
        headers.insert("accept-language", "en-US,en;q=0.9".parse().unwrap());
        headers.insert("accept-encoding", "gzip, deflate, br".parse().unwrap());
        headers.insert("dnt", "1".parse().unwrap());
        headers
    }
}
```

### 5. Bybit P2P Implementation

```rust
/// Bybit P2P client for advertisement and order management.
///
/// IMPORTANT: Test with credentials from test_data/bybit_credentials.json
pub struct BybitP2PClient {
    api_key: String,
    api_secret: String,
    client: Client,
    rate_limiter: Arc<RateLimiter>,
}

impl BybitP2PClient {
    /// Creates a new P2P advertisement.
    ///
    /// # Arguments
    /// * `params` - Advertisement parameters including amount, price, payment methods
    ///
    /// # Example
    /// ```
    /// let ad = client.create_advertisement(AdParams {
    ///     asset: "USDT",
    ///     fiat: "RUB",
    ///     price: "91.50",
    ///     amount: "1000.00",
    ///     payment_methods: vec!["1"], // Bank card
    ///     remarks: "Fast release, T-Bank only",
    /// }).await?;
    /// ```
    pub async fn create_advertisement(&self, params: AdParams) -> Result<Advertisement> {
        self.rate_limiter.check_and_wait("bybit_p2p_create").await?;

        let timestamp = chrono::Utc::now().timestamp_millis().to_string();
        let sign = self.generate_signature(&params, &timestamp)?;

        let response = self.client
            .post("https://api.bybit.com/v5/p2p/item/create")
            .header("X-BAPI-API-KEY", &self.api_key)
            .header("X-BAPI-SIGN", sign)
            .header("X-BAPI-TIMESTAMP", timestamp)
            .json(&params)
            .send()
            .await?;

        // Handle response...
    }

    /// Generates HMAC-SHA256 signature for request.
    fn generate_signature(&self, params: &impl Serialize, timestamp: &str) -> Result<String> {
        // Implementation following Bybit docs
    }
}
```

### 6. AI Chat Manager

```rust
/// Manages AI-powered conversations with P2P buyers.
///
/// Uses OpenRouter API to generate contextual responses.
pub struct ChatManager {
    client: OpenRouterClient,
    config: ChatConfig,
}

impl ChatManager {
    /// Generates a response for the current conversation state.
    ///
    /// # Arguments
    /// * `order` - Current order details
    /// * `message` - Incoming message from buyer
    /// * `history` - Previous conversation messages
    ///
    /// # Returns
    /// Generated response with appropriate delay
    pub async fn generate_response(
        &self,
        order: &Order,
        message: &str,
        history: &[ChatMessage],
    ) -> Result<String> {
        // Build context
        let context = self.build_context(order, history)?;

        // Detect language
        let language = self.detect_language(message).await?;

        // Generate response
        let prompt = self.build_prompt(context, message, language)?;
        let response = self.client.complete(prompt).await?;

        // Add random delay for human-like behavior
        let delay = rand::thread_rng().gen_range(
            self.config.min_delay..self.config.max_delay
        );
        tokio::time::sleep(Duration::from_secs(delay)).await;

        Ok(response)
    }
}
```

### 7. OCR Receipt Processor

```rust
/// Processes payment receipts using Tesseract OCR.
///
/// Validates T-Bank receipts and extracts transaction details.
pub struct ReceiptProcessor {
    tesseract: Tesseract,
}

impl ReceiptProcessor {
    /// Processes receipt image and validates payment.
    ///
    /// # Arguments
    /// * `image_data` - Receipt image bytes
    /// * `expected_amount` - Expected payment amount
    ///
    /// # Returns
    /// Extracted receipt data if valid
    pub async fn process_receipt(
        &self,
        image_data: &[u8],
        expected_amount: Decimal,
    ) -> Result<ReceiptData> {
        // Extract text
        let text = self.tesseract.ocr_from_bytes(image_data)?;

        // Validate T-Bank receipt
        if !text.contains("Т-Банк") && !text.contains("Tinkoff") {
            return Err(AppError::InvalidReceipt("Not a T-Bank receipt".into()));
        }

        // Extract amount
        let amount = self.extract_amount(&text)?;

        // Validate amount matches
        if (amount - expected_amount).abs() > Decimal::from_str("0.01")? {
            return Err(AppError::AmountMismatch {
                expected: expected_amount,
                received: amount,
            });
        }

        Ok(ReceiptData {
            amount,
            bank: "T-Bank".to_string(),
            reference: self.extract_reference(&text)?,
            timestamp: self.extract_timestamp(&text)?,
        })
    }
}
```

### 8. Main Orchestrator

```rust
/// Core orchestrator that coordinates all components.
///
/// Manages the complete flow from Gate transaction to Bybit completion.
pub struct Orchestrator {
    state: Arc<AppState>,
    gate_manager: Arc<GateAccountManager>,
    bybit_manager: Arc<BybitAccountManager>,
    chat_manager: Arc<ChatManager>,
    receipt_processor: Arc<ReceiptProcessor>,
}

impl Orchestrator {
    /// Main processing loop for new transactions.
    pub async fn start_processing(&self) -> Result<()> {
        info!("Starting order processing loop");

        // Restore state from database
        self.restore_active_orders().await?;

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            // Check for new transactions
            if let Err(e) = self.process_new_transactions().await {
                error!("Error processing transactions: {}", e);
                // Continue loop, don't crash
            }
        }
    }

    /// Processes a single transaction from start to finish.
    async fn process_transaction(&self, tx: GateTransaction) -> Result<()> {
        span!(Level::INFO, "process_transaction", tx_id = %tx.id);

        info!("Processing new transaction: {}", tx.id);

        // 1. Accept on Gate
        self.gate_manager.accept_transaction(&tx).await
            .context("Failed to accept transaction")?;

        // 2. Find available Bybit account
        let bybit_account = self.bybit_manager
            .find_available_account()
            .await?
            .ok_or_else(|| AppError::NoAvailableAccounts)?;

        // 3. Calculate rate using your formula
        let rate = self.calculate_rate(&tx)?;

        // 4. Create Bybit advertisement
        let ad = self.bybit_manager
            .create_advertisement(&bybit_account, &tx, rate)
            .await
            .context("Failed to create advertisement")?;

        // 5. Save to database
        let order = self.save_order(&tx, &ad).await?;

        // 6. Start monitoring
        tokio::spawn({
            let orchestrator = self.clone();
            let order = order.clone();
            async move {
                if let Err(e) = orchestrator.monitor_order(order).await {
                    error!("Error monitoring order: {}", e);
                }
            }
        });

        Ok(())
    }
}
```

## 🔐 Security Considerations

1. **Encryption at Rest**
   ```rust
   // All sensitive data must be encrypted
   let encrypted_password = crypto::encrypt(&password, &encryption_key)?;
   let encrypted_api_secret = crypto::encrypt(&api_secret, &encryption_key)?;
   ```

2. **Environment Variables**
   ```bash
   # .env file (never commit!)
   DATABASE_URL=postgresql://user:pass@localhost/p2p_trading
   ENCRYPTION_KEY=your-32-byte-key-here
   OPENROUTER_API_KEY=your-api-key
   JWT_SECRET=your-jwt-secret
   ```

3. **Input Validation**
   ```rust
   // Always validate external input
   pub fn validate_transaction_amount(amount: Decimal) -> Result<()> {
       if amount <= Decimal::ZERO {
           return Err(AppError::InvalidInput("Amount must be positive"));
       }
       if amount > MAX_TRANSACTION_AMOUNT {
           return Err(AppError::InvalidInput("Amount exceeds maximum"));
       }
       Ok(())
   }
   ```

## 📡 WebSocket API Specification

```typescript
// WebSocket connection
const ws = new WebSocket('ws://localhost:8080/ws');

// Authentication
ws.send(JSON.stringify({
    type: 'auth',
    token: 'your-jwt-token'
}));

// Subscribe to updates
ws.send(JSON.stringify({
    type: 'subscribe',
    channels: ['orders', 'pools', 'metrics']
}));

// Receive updates
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    switch (data.type) {
        case 'order_update':
            // Handle order status change
            break;
        case 'pool_update':
            // Handle pool state change
            break;
        case 'metrics':
            // Handle system metrics
            break;
    }
};
```

## 🚀 Deployment

### Docker Configuration
```dockerfile
# Multi-stage build
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build with optimizations
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    tesseract-ocr \
    tesseract-ocr-rus \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/p2p-trading-bot /usr/local/bin/

# Create non-root user
RUN useradd -m -u 1000 botuser
USER botuser

CMD ["p2p-trading-bot"]
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: p2p_trading
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  bot:
    build: .
    depends_on:
      - postgres
      - redis
    environment:
      DATABASE_URL: postgresql://postgres:password@postgres/p2p_trading
      REDIS_URL: redis://redis:6379
      RUST_LOG: info,p2p_trading_bot=debug
    volumes:
      - ./config:/app/config
      - ./test_data:/app/test_data
    ports:
      - "8080:8080"
      - "9090:9090"  # Metrics

volumes:
  postgres_data:
```

## 📊 Monitoring & Metrics

### Prometheus Metrics
```rust
use prometheus::{IntCounter, IntGauge, Histogram, register_int_counter};

lazy_static! {
    static ref ORDERS_CREATED: IntCounter =
        register_int_counter!("orders_created_total", "Total orders created").unwrap();

    static ref ORDERS_COMPLETED: IntCounter =
        register_int_counter!("orders_completed_total", "Total orders completed").unwrap();

    static ref ACTIVE_ORDERS: IntGauge =
        register_int_gauge!("active_orders", "Currently active orders").unwrap();

    static ref ORDER_PROCESSING_TIME: Histogram =
        register_histogram!("order_processing_seconds", "Order processing time").unwrap();
}
```

### Health Check Endpoint
```rust
async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
        .is_ok();

    let redis_healthy = state.redis_pool
        .lock()
        .await
        .ping()
        .await
        .is_ok();

    let status = if db_healthy && redis_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(json!({
        "status": if status.is_success() { "healthy" } else { "unhealthy" },
        "database": db_healthy,
        "redis": redis_healthy,
        "uptime": state.start_time.elapsed().as_secs(),
    })))
}
```

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_calculation() {
        let tx = create_test_transaction();
        let rate = calculate_rate(&tx).unwrap();
        assert_eq!(rate, Decimal::from_str("91.50").unwrap());
    }

    #[tokio::test]
    async fn test_amount_validation() {
        assert!(validate_amount(Decimal::from(100)).is_ok());
        assert!(validate_amount(Decimal::from(-100)).is_err());
        assert!(validate_amount(Decimal::from(1_000_001)).is_err());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_complete_flow() {
    // Setup test database
    let db = setup_test_db().await;

    // Create test accounts
    let gate_account = create_test_gate_account(&db).await;
    let bybit_account = create_test_bybit_account(&db).await;

    // Simulate transaction
    let tx = create_test_transaction();

    // Process through orchestrator
    let orchestrator = create_test_orchestrator(db).await;
    orchestrator.process_transaction(tx).await.unwrap();

    // Verify order created
    let order = db.get_order_by_tx_id(&tx.id).await.unwrap();
    assert_eq!(order.status, "active");
}
```

## 🐛 Common Issues & Solutions

### 1. Cloudflare 403 Errors
```rust
// Implement retry with different headers
if response.status() == StatusCode::FORBIDDEN {
    // Try with different user agent
    // Add more browser-like headers
    // Consider using a different proxy
}
```

### 2. Session Expiration
```rust
// Auto-refresh sessions before expiry
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(1200)).await; // 20 minutes
        if let Err(e) = refresh_all_sessions().await {
            error!("Failed to refresh sessions: {}", e);
        }
    }
});
```

### 3. Rate Limiting
```rust
// Implement exponential backoff
let mut retry_delay = Duration::from_secs(1);
for attempt in 0..MAX_RETRIES {
    match make_request().await {
        Ok(response) => return Ok(response),
        Err(e) if e.is_rate_limit() => {
            tokio::time::sleep(retry_delay).await;
            retry_delay *= 2;
        }
        Err(e) => return Err(e),
    }
}
```

## 📝 Development Workflow

1. **Always test with test data first**
2. **Document every function and module**
3. **Handle all error cases explicitly**
4. **Log important operations with context**
5. **Write tests for critical paths**
6. **Monitor resource usage**
7. **Implement graceful shutdown**

## 🎯 Key Business Logic

### Rate Calculation Formula
Based on your provided diagram:
1. Check order amount and time (Moscow time)
2. Apply tiered pricing based on amount and time
3. Take last X pages from SEP\Tinkoff filter
4. Calculate final rate

### Order State Machine
```
PENDING → ACCEPTED → ADVERTISED → BUYER_FOUND → CHATTING →
PAYMENT_RECEIVED → VERIFIED → COMPLETED
                                    ↓
                                 FAILED
```

### Pool Management
- **Pending Pool**: New transactions from Gate
- **Active Pool**: Orders with active Bybit ads
- **Chat Pool**: Orders in conversation
- **Verification Pool**: Awaiting receipt verification
- **Completed Pool**: Successfully completed orders

## 🚦 Getting Started

1. Clone repository and install Rust
2. Set up PostgreSQL and Redis
3. Copy `.env.example` to `.env` and configure
4. Copy test credentials to `test_data/`
5. Run migrations: `cargo sqlx migrate run`
6. Test with: `cargo test`
7. Run development: `cargo run`
8. Monitor logs: `RUST_LOG=debug cargo run`

## ⚠️ Important Reminders

1. **NEVER commit real credentials**
2. **Always test with test accounts first**
3. **Monitor rate limits carefully**
4. **Implement proper error recovery**
5. **Keep audit logs of all transactions**
6. **Set up alerts for failures**
7. **Regularly backup database**
8. **Update dependencies regularly**

This system handles financial transactions - accuracy and reliability are critical!
