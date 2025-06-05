# Building iTrader Backend

## Required System Dependencies

The project requires the following system libraries:

### Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install -y \
    libssl-dev \           # OpenSSL development libraries
    pkg-config \           # Package configuration tool
    redis-server \         # Redis server
    libtesseract-dev \     # Tesseract OCR development files
    libleptonica-dev \     # Leptonica image processing library
    clang                  # C compiler for some dependencies
```

### Quick Installation:
```bash
# Run the quick fix script
./quick-fix.sh

# Or full installation
./install-deps.sh
```

## Common Build Errors

### 1. OpenSSL Error
```
error: failed to run custom build command for `openssl-sys`
```
**Solution:** Install libssl-dev:
```bash
sudo apt-get install libssl-dev pkg-config
```

### 2. Leptonica Error
```
error: failed to run custom build command for `leptonica-sys`
```
**Solution:** Install Tesseract and Leptonica development libraries:
```bash
sudo apt-get install libtesseract-dev libleptonica-dev
```

### 3. PostgreSQL Connection Error
```
Failed to connect to PostgreSQL
```
**Solution:** Ensure PostgreSQL is running and user is configured:
```bash
sudo systemctl start postgresql
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'root';"
```

### 4. Redis Connection Error
```
Failed to connect to Redis
```
**Solution:** Start Redis server:
```bash
sudo systemctl start redis-server
```

## Building Without OCR

If you want to build without OCR support temporarily:

1. Comment out tesseract in Cargo.toml:
```toml
# OCR
# tesseract = "0.15"
```

2. The OCR module will use mock data instead

## Build Commands

### Development Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### Run with Hot Reload
```bash
./dev.sh
```

## Verifying Installation

Check all dependencies are installed:
```bash
# Check OpenSSL
pkg-config --modversion openssl

# Check Tesseract
pkg-config --modversion tesseract

# Check Leptonica
pkg-config --modversion lept

# Check Redis
redis-cli ping

# Check PostgreSQL
psql --version
```

If all commands return versions or PONG (for Redis), you're ready to build!