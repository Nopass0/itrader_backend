#!/bin/bash

# iTrader Backend Installation Script
# For Ubuntu/Debian systems

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
POSTGRES_USER="postgres"
POSTGRES_PASSWORD="root"
DATABASE_NAME="itrader"

echo -e "${CYAN}================================================"
echo "iTrader Backend - Complete System Installation"
echo -e "================================================${NC}"
echo

# Function to check if running as root
check_root() {
    if [ "$EUID" -eq 0 ]; then
        echo -e "${YELLOW}Warning: Running as root. Some operations may require sudo later.${NC}"
    fi
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install system dependencies
install_system_deps() {
    echo -e "${BLUE}Installing system dependencies...${NC}"
    
    # Update package list
    sudo apt-get update -y
    
    # Install essential packages
    sudo apt-get install -y \
        curl \
        wget \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        libpq-dev \
        python3 \
        python3-pip \
        python3-dev \
        python3-venv \
        postgresql \
        postgresql-contrib \
        redis-server \
        tesseract-ocr \
        tesseract-ocr-rus \
        libpoppler-cpp-dev \
        poppler-utils
    
    echo -e "${GREEN}✓ System dependencies installed${NC}"
}

# Function to install Rust
install_rust() {
    if ! command_exists rustc; then
        echo -e "${BLUE}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}✓ Rust installed${NC}"
    else
        echo -e "${GREEN}✓ Rust already installed ($(rustc --version))${NC}"
    fi
    
    # Install sqlx-cli
    if ! command_exists sqlx; then
        echo -e "${BLUE}Installing sqlx-cli...${NC}"
        cargo install sqlx-cli --no-default-features --features postgres
        echo -e "${GREEN}✓ sqlx-cli installed${NC}"
    else
        echo -e "${GREEN}✓ sqlx-cli already installed${NC}"
    fi
}

# Function to setup PostgreSQL
setup_postgresql() {
    echo -e "${BLUE}Setting up PostgreSQL...${NC}"
    
    # Start PostgreSQL service
    sudo systemctl start postgresql
    sudo systemctl enable postgresql
    
    # Create postgres user with password
    sudo -u postgres psql << EOF 2>/dev/null || true
ALTER USER postgres PASSWORD '${POSTGRES_PASSWORD}';
EOF
    
    # Create database
    sudo -u postgres psql << EOF 2>/dev/null || true
CREATE DATABASE ${DATABASE_NAME};
EOF
    
    # Grant all privileges
    sudo -u postgres psql << EOF 2>/dev/null || true
GRANT ALL PRIVILEGES ON DATABASE ${DATABASE_NAME} TO ${POSTGRES_USER};
EOF
    
    echo -e "${GREEN}✓ PostgreSQL configured${NC}"
}

# Function to setup Redis
setup_redis() {
    echo -e "${BLUE}Setting up Redis...${NC}"
    
    # Start Redis service
    sudo systemctl start redis-server
    sudo systemctl enable redis-server
    
    # Test Redis connection
    if redis-cli ping > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Redis is running${NC}"
    else
        echo -e "${RED}✗ Redis connection failed${NC}"
        exit 1
    fi
}

# Function to create .env file
create_env_file() {
    if [ ! -f .env ]; then
        echo -e "${BLUE}Creating .env file...${NC}"
        
        # Generate random JWT secret
        JWT_SECRET=$(openssl rand -base64 32)
        
        cat > .env << EOF
# Database Configuration
DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost/${DATABASE_NAME}
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=1

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=10

# API Keys
OPENROUTER_API_KEY=your-openrouter-api-key
JWT_SECRET=${JWT_SECRET}

# Email Configuration (optional)
EMAIL_ADDRESS=your-email@gmail.com
EMAIL_PASSWORD=your-app-password

# Admin Token
ADMIN_TOKEN=dev-token-123

# Python Configuration
PYTHONPATH=/usr/lib/python3/dist-packages
EOF
        
        echo -e "${GREEN}✓ Created .env file${NC}"
        echo -e "${YELLOW}⚠️  Please update API keys in .env file${NC}"
    else
        echo -e "${GREEN}✓ .env file already exists${NC}"
    fi
}

# Function to create directory structure
create_directories() {
    echo -e "${BLUE}Creating directory structure...${NC}"
    
    mkdir -p db/gate db/bybit db/gmail db/transactions db/checks
    mkdir -p data logs
    mkdir -p libs
    
    # Set permissions
    chmod -R 755 db data logs
    
    echo -e "${GREEN}✓ Directories created${NC}"
}

# Function to install Python dependencies
install_python_deps() {
    echo -e "${BLUE}Installing Python dependencies...${NC}"
    
    # Create virtual environment if it doesn't exist
    if [ ! -d "venv" ]; then
        python3 -m venv venv
    fi
    
    # Activate virtual environment and install dependencies
    source venv/bin/activate
    pip install --upgrade pip
    pip install -r requirements.txt
    
    # Create Python library symlink for Rust integration
    PYTHON_VERSION=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
    PYTHON_LIB_PATH=$(python3 -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))')
    
    if [ -f "$PYTHON_LIB_PATH/libpython${PYTHON_VERSION}.so" ]; then
        ln -sf "$PYTHON_LIB_PATH/libpython${PYTHON_VERSION}.so" libs/libpython${PYTHON_VERSION}.so.1.0
        echo -e "${GREEN}✓ Python library linked${NC}"
    else
        echo -e "${YELLOW}⚠️  Python library not found, Bybit P2P features may be limited${NC}"
    fi
    
    deactivate
    echo -e "${GREEN}✓ Python dependencies installed${NC}"
}

# Function to run database migrations
run_migrations() {
    echo -e "${BLUE}Running database migrations...${NC}"
    
    # Source cargo env if needed
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    # Run migrations
    if command_exists sqlx; then
        sqlx migrate run || {
            echo -e "${YELLOW}⚠️  Migration failed, trying manual approach...${NC}"
            
            # Manual migration as fallback
            for migration in migrations/*.sql; do
                echo "Running $migration..."
                PGPASSWORD=${POSTGRES_PASSWORD} psql -h localhost -U ${POSTGRES_USER} -d ${DATABASE_NAME} -f "$migration" || true
            done
        }
        echo -e "${GREEN}✓ Database migrations completed${NC}"
    else
        echo -e "${RED}✗ sqlx not found, please install it first${NC}"
        exit 1
    fi
}

# Function to build Rust project
build_rust_project() {
    echo -e "${BLUE}Building Rust project...${NC}"
    
    # Source cargo env if needed
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    # Build in release mode
    cargo build --release
    
    echo -e "${GREEN}✓ Rust project built${NC}"
}

# Function to create systemd service
create_systemd_service() {
    echo -e "${BLUE}Creating systemd service...${NC}"
    
    sudo tee /etc/systemd/system/itrader-backend.service > /dev/null << EOF
[Unit]
Description=iTrader Backend Service
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=$USER
WorkingDirectory=$(pwd)
Environment="PATH=$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
Environment="LD_LIBRARY_PATH=$(pwd)/libs:/usr/lib/x86_64-linux-gnu:/usr/local/lib"
ExecStart=$(pwd)/target/release/itrader-backend
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
    
    sudo systemctl daemon-reload
    echo -e "${GREEN}✓ Systemd service created${NC}"
    echo -e "${YELLOW}To start the service: sudo systemctl start itrader-backend${NC}"
    echo -e "${YELLOW}To enable on boot: sudo systemctl enable itrader-backend${NC}"
}

# Main installation flow
main() {
    check_root
    
    echo -e "${CYAN}This script will install:${NC}"
    echo "  • PostgreSQL and create database '${DATABASE_NAME}'"
    echo "  • Redis server"
    echo "  • Rust toolchain and sqlx-cli"
    echo "  • Python 3 and required packages"
    echo "  • All system dependencies"
    echo "  • Run database migrations"
    echo "  • Build the Rust project"
    echo
    read -p "Continue with installation? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 1
    fi
    
    # Run installation steps
    install_system_deps
    install_rust
    setup_postgresql
    setup_redis
    create_env_file
    create_directories
    install_python_deps
    run_migrations
    build_rust_project
    
    # Optional: Create systemd service
    read -p "Create systemd service? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        create_systemd_service
    fi
    
    echo
    echo -e "${GREEN}================================================"
    echo "Installation completed successfully!"
    echo -e "================================================${NC}"
    echo
    echo "Next steps:"
    echo "1. Update API keys in .env file"
    echo "2. Run './run.sh' to start the development server"
    echo "3. Or use 'sudo systemctl start itrader-backend' for production"
    echo
    echo -e "${CYAN}For account management: ./run.sh --settings${NC}"
}

# Run main function
main "$@"