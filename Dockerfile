# Multi-stage build for iTrader Backend
FROM rust:1.75-bullseye as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    python3 \
    python3-dev \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli for migrations
RUN cargo install sqlx-cli --no-default-features --features postgres

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code and other files
COPY src ./src
COPY migrations ./migrations
COPY config ./config
COPY requirements.txt ./
COPY python_modules ./python_modules

# Install Python dependencies
RUN pip3 install --no-cache-dir -r requirements.txt

# Build the application
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    python3 \
    python3-pip \
    tesseract-ocr \
    tesseract-ocr-rus \
    poppler-utils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary and necessary files
COPY --from=builder /app/target/release/itrader-backend /app/
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/
COPY config ./config
COPY requirements.txt ./
COPY python_modules ./python_modules

# Install Python dependencies
RUN pip3 install --no-cache-dir -r requirements.txt

# Create necessary directories
RUN mkdir -p db/gate db/bybit db/gmail db/transactions db/checks data logs libs

# Create non-root user
RUN useradd -m -u 1001 app && chown -R app:app /app
USER app

# Environment variables
ENV RUST_LOG=info,itrader_backend=debug
ENV LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:/usr/local/lib:/app/libs
ENV PYTHONPATH=/app/python_modules

# Expose port
EXPOSE 8080

# Run migrations and start the application
CMD sqlx migrate run && ./itrader-backend