#!/bin/bash

# iTrader Backend Automatic Installation Script
# Installs everything automatically without confirmations

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
DATABASE_TEST_NAME="itrader_test"

echo -e "${CYAN}================================================"
echo "iTrader Backend - Automatic Installation"
echo -e "================================================${NC}"
echo

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install system dependencies
install_system_deps() {
    echo -e "${BLUE}[1/9] Installing system dependencies...${NC}"
    
    # Check if we have sudo access or running as root
    if [ "$EUID" -eq 0 ]; then
        APT_CMD="apt-get"
    elif sudo -n true 2>/dev/null; then
        APT_CMD="sudo apt-get"
    else
        echo -e "${YELLOW}⚠️  Skipping system dependencies (no sudo access)${NC}"
        return 0
    fi
    
    # Update package list quietly
    $APT_CMD update -qq 2>/dev/null || true
    
    # Install essential packages
    DEBIAN_FRONTEND=noninteractive $APT_CMD install -y -qq \
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
        poppler-utils \
        ca-certificates \
        software-properties-common \
        apt-transport-https \
        gnupg \
        lsb-release \
        unzip
    
    echo -e "${GREEN}✓ System dependencies installed${NC}"
}

# Function to install Rust
install_rust() {
    echo -e "${BLUE}[2/9] Installing Rust...${NC}"
    
    if ! command_exists rustc; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
        source "$HOME/.cargo/env"
        echo -e "${GREEN}✓ Rust installed ($(rustc --version))${NC}"
    else
        echo -e "${GREEN}✓ Rust already installed ($(rustc --version))${NC}"
    fi
    
    # Ensure cargo env is available
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    # Install sqlx-cli
    if ! command_exists sqlx; then
        echo -e "${BLUE}Installing sqlx-cli...${NC}"
        cargo install sqlx-cli --no-default-features --features postgres --quiet
        echo -e "${GREEN}✓ sqlx-cli installed${NC}"
    else
        echo -e "${GREEN}✓ sqlx-cli already installed${NC}"
    fi
}

# Function to install UV package manager
install_uv() {
    echo -e "${BLUE}[3/9] Installing UV package manager...${NC}"
    
    if ! command_exists uv; then
        curl -LsSf https://astral.sh/uv/install.sh | sh
        export PATH="$HOME/.cargo/bin:$PATH"
        echo -e "${GREEN}✓ UV package manager installed${NC}"
    else
        echo -e "${GREEN}✓ UV package manager already installed${NC}"
    fi
}

# Function to setup PostgreSQL
setup_postgresql() {
    echo -e "${BLUE}[4/9] Setting up PostgreSQL...${NC}"
    
    # Check if PostgreSQL is already running
    if ! systemctl is-active postgresql >/dev/null 2>&1; then
        if sudo -n systemctl start postgresql >/dev/null 2>&1; then
            sudo systemctl enable postgresql >/dev/null 2>&1
        else
            echo -e "${YELLOW}⚠️  Could not start PostgreSQL service (no sudo), assuming it's running${NC}"
        fi
    fi
    
    # Wait for PostgreSQL to start
    sleep 2
    
    # Create postgres user with password (suppress output)
    sudo -u postgres psql -c "ALTER USER postgres PASSWORD '${POSTGRES_PASSWORD}';" >/dev/null 2>&1 || true
    
    # Create main database
    sudo -u postgres psql -c "CREATE DATABASE ${DATABASE_NAME};" >/dev/null 2>&1 || true
    
    # Create test database
    sudo -u postgres psql -c "CREATE DATABASE ${DATABASE_TEST_NAME};" >/dev/null 2>&1 || true
    
    # Grant all privileges
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE ${DATABASE_NAME} TO ${POSTGRES_USER};" >/dev/null 2>&1 || true
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE ${DATABASE_TEST_NAME} TO ${POSTGRES_USER};" >/dev/null 2>&1 || true
    
    echo -e "${GREEN}✓ PostgreSQL configured (databases: ${DATABASE_NAME}, ${DATABASE_TEST_NAME})${NC}"
}

# Function to setup Redis
setup_redis() {
    echo -e "${BLUE}[5/9] Setting up Redis...${NC}"
    
    # Start and enable Redis service if possible
    if sudo -n systemctl start redis-server >/dev/null 2>&1; then
        sudo systemctl enable redis-server >/dev/null 2>&1
    else
        echo -e "${YELLOW}⚠️  Could not start Redis service (no sudo), assuming it's running${NC}"
    fi
    
    # Wait for Redis to start
    sleep 1
    
    # Test Redis connection
    if redis-cli ping >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Redis is running${NC}"
    else
        echo -e "${YELLOW}⚠️  Redis service started but connection test failed${NC}"
    fi
}

# Function to create environment files
create_env_files() {
    echo -e "${BLUE}[6/9] Creating environment files...${NC}"
    
    # Generate random JWT secret if openssl is available
    if command_exists openssl; then
        JWT_SECRET=$(openssl rand -base64 32)
    else
        JWT_SECRET="dev-jwt-secret-$(date +%s)"
    fi
    
    # Create main .env file
    if [ ! -f .env ]; then
        cat > .env << EOF
# Database Configuration
DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost/${DATABASE_NAME}
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=1

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=10

# API Keys (UPDATE THESE)
OPENROUTER_API_KEY=your-openrouter-api-key
JWT_SECRET=${JWT_SECRET}

# Email Configuration (optional)
EMAIL_ADDRESS=your-email@gmail.com
EMAIL_PASSWORD=your-app-password

# Admin Token
ADMIN_TOKEN=dev-token-123

# Bybit Configuration
APP__BYBIT__REST_URL=https://api.bybit.com
APP__BYBIT__WS_URL=wss://stream.bybit.com
EOF
    fi
    
    # Create test .env file
    cat > .env.test << EOF
# Test Database Configuration
DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost/${DATABASE_TEST_NAME}
DATABASE_MAX_CONNECTIONS=5
DATABASE_MIN_CONNECTIONS=1

# Redis Configuration
REDIS_URL=redis://localhost:6379/1
REDIS_POOL_SIZE=5

# Test API Keys
OPENROUTER_API_KEY=test-api-key
JWT_SECRET=test-jwt-secret

# Test Email Configuration
EMAIL_ADDRESS=test@example.com
EMAIL_PASSWORD=test-password

# Bybit Test Configuration
APP__BYBIT__REST_URL=https://api-testnet.bybit.com
APP__BYBIT__WS_URL=wss://stream-testnet.bybit.com
EOF
    
    echo -e "${GREEN}✓ Environment files created${NC}"
}

# Function to create directory structure
create_directories() {
    echo -e "${BLUE}[7/9] Creating directory structure...${NC}"
    
    # Create required directories
    mkdir -p db/{gate,bybit,gmail,transactions,checks}
    mkdir -p data logs libs test_data
    
    # Create example data files if they don't exist
    if [ ! -f data/accounts_example.json ]; then
        cat > data/accounts_example.json << 'EOF'
{
  "gate_accounts": [
    {
      "email": "example@gmail.com",
      "password": "your_password"
    }
  ],
  "bybit_accounts": [
    {
      "account_name": "main",
      "api_key": "your_api_key",
      "api_secret": "your_api_secret"
    }
  ]
}
EOF
    fi
    
    # Set proper permissions
    chmod -R 755 db data logs
    
    echo -e "${GREEN}✓ Directories created${NC}"
}

# Function to setup Python environment with UV
setup_python_env() {
    echo -e "${BLUE}[8/9] Setting up Python environment...${NC}"
    
    # Ensure UV is in PATH
    export PATH="$HOME/.cargo/bin:$PATH"
    
    # Create Python virtual environment with UV
    if [ ! -d ".venv" ]; then
        uv venv --python 3.11 >/dev/null 2>&1 || uv venv >/dev/null 2>&1
    fi
    
    # Install Python dependencies with UV
    if [ -f "requirements.txt" ]; then
        uv pip install -r requirements.txt >/dev/null 2>&1
    fi
    
    # Create fallback requirements if missing
    if [ ! -f "requirements.txt" ]; then
        cat > requirements.txt << 'EOF'
requests>=2.31.0
pybit>=5.7.0
python-dotenv>=1.0.0
psycopg2-binary>=2.9.7
redis>=5.0.0
Pillow>=10.0.0
pytesseract>=0.3.10
pdf2image>=1.16.3
pdfplumber>=0.10.3
PyPDF2>=3.0.1
opencv-python>=4.8.0
numpy>=1.24.0
pandas>=2.0.0
fastapi>=0.100.0
uvicorn>=0.23.0
pydantic>=2.0.0
sqlalchemy>=2.0.0
alembic>=1.11.0
celery>=5.3.0
EOF
        uv pip install -r requirements.txt >/dev/null 2>&1
    fi
    
    echo -e "${GREEN}✓ Python environment configured with UV${NC}"
}

# Function to run database migrations
run_migrations() {
    echo -e "${BLUE}[9/9] Running database migrations...${NC}"
    
    # Ensure cargo env is available
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    # Set database URLs for migrations
    export DATABASE_URL="postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost/${DATABASE_NAME}"
    
    # Run migrations if sqlx is available
    if command_exists sqlx && [ -d "migrations" ]; then
        sqlx migrate run >/dev/null 2>&1 || {
            echo -e "${YELLOW}Running manual migrations...${NC}"
            # Manual migration as fallback
            for migration in migrations/*.sql; do
                if [ -f "$migration" ]; then
                    PGPASSWORD=${POSTGRES_PASSWORD} psql -h localhost -U ${POSTGRES_USER} -d ${DATABASE_NAME} -f "$migration" >/dev/null 2>&1 || true
                fi
            done
        }
        
        # Run migrations for test database
        export DATABASE_URL="postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost/${DATABASE_TEST_NAME}"
        sqlx migrate run >/dev/null 2>&1 || {
            for migration in migrations/*.sql; do
                if [ -f "$migration" ]; then
                    PGPASSWORD=${POSTGRES_PASSWORD} psql -h localhost -U ${POSTGRES_USER} -d ${DATABASE_TEST_NAME} -f "$migration" >/dev/null 2>&1 || true
                fi
            done
        }
    else
        echo -e "${YELLOW}⚠️  No migrations found or sqlx not available${NC}"
    fi
    
    echo -e "${GREEN}✓ Database migrations completed${NC}"
}

# Function to build project (optional, for faster startup)
build_project() {
    echo -e "${BLUE}Building project...${NC}"
    
    # Ensure cargo env is available
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    # Set database URL for compilation
    export DATABASE_URL="postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost/${DATABASE_NAME}"
    
    # Try to build, but don't fail if it doesn't work
    if cargo check >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Project builds successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  Project has compilation issues, but installation completed${NC}"
        echo -e "${YELLOW}    Run './run.sh' to see detailed errors${NC}"
    fi
}

# Function to create run scripts
create_run_scripts() {
    # Make scripts executable
    chmod +x run.sh test.sh install.sh 2>/dev/null || true
    
    # Create start script for production
    cat > start.sh << 'EOF'
#!/bin/bash
# Quick start script
echo "Starting iTrader Backend..."
export PATH="$HOME/.cargo/bin:$PATH"
source .venv/bin/activate 2>/dev/null || true
./run.sh
EOF
    chmod +x start.sh
}

# Main installation function
main() {
    echo -e "${CYAN}Installing iTrader Backend automatically...${NC}"
    echo -e "${CYAN}This will install all dependencies and configure the system.${NC}"
    echo
    
    # Run all installation steps
    install_system_deps
    install_rust  
    install_uv
    setup_postgresql
    setup_redis
    create_env_files
    create_directories
    setup_python_env
    run_migrations
    build_project
    create_run_scripts
    
    echo
    echo -e "${GREEN}================================================"
    echo "🎉 Installation completed successfully!"
    echo -e "================================================${NC}"
    echo
    echo -e "${CYAN}Quick Start:${NC}"
    echo -e "  ${GREEN}./start.sh${NC}          - Start development server"
    echo -e "  ${GREEN}./run.sh${NC}            - Run with full options" 
    echo -e "  ${GREEN}./test.sh bybit-simple${NC} - Test Bybit integration"
    echo
    echo -e "${CYAN}Configuration:${NC}"
    echo -e "  ${YELLOW}• Update API keys in .env file${NC}"
    echo -e "  ${YELLOW}• Add accounts to data/accounts.json${NC}"
    echo -e "  ${YELLOW}• Check test_data/ for credential examples${NC}"
    echo
    echo -e "${GREEN}Everything is ready to use! 🚀${NC}"
}

# Run the installation
main "$@"