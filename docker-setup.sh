#!/bin/bash

# Docker setup script for iTrader Backend

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}================================================"
echo "iTrader Backend - Docker Setup"
echo -e "================================================${NC}"
echo

# Check if Docker is installed
if ! command -v docker >/dev/null 2>&1; then
    echo -e "${YELLOW}Docker is not installed. Installing Docker...${NC}"
    
    # Install Docker
    curl -fsSL https://get.docker.com | sh
    
    # Add current user to docker group
    sudo usermod -aG docker $USER
    
    echo -e "${GREEN}✓ Docker installed${NC}"
    echo -e "${YELLOW}Please log out and back in for group changes to take effect${NC}"
fi

# Check if Docker Compose is installed
if ! command -v docker-compose >/dev/null 2>&1; then
    echo -e "${YELLOW}Docker Compose is not installed. Installing...${NC}"
    
    # Install Docker Compose
    sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    sudo chmod +x /usr/local/bin/docker-compose
    
    echo -e "${GREEN}✓ Docker Compose installed${NC}"
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo -e "${BLUE}Creating .env file...${NC}"
    
    # Generate random JWT secret
    JWT_SECRET=$(openssl rand -base64 32)
    
    cat > .env << EOF
# API Keys
OPENROUTER_API_KEY=your-openrouter-api-key
JWT_SECRET=${JWT_SECRET}
ADMIN_TOKEN=dev-token-123

# Email Configuration (optional)
EMAIL_ADDRESS=
EMAIL_PASSWORD=
EOF
    
    echo -e "${GREEN}✓ Created .env file${NC}"
    echo -e "${YELLOW}⚠️  Please update API keys in .env file${NC}"
fi

# Create necessary directories
echo -e "${BLUE}Creating directories...${NC}"
mkdir -p config logs db/gate db/bybit db/gmail db/transactions db/checks data

# Build and start services
echo -e "${BLUE}Building Docker images...${NC}"
docker-compose build

echo
echo -e "${GREEN}================================================"
echo "Docker setup completed!"
echo -e "================================================${NC}"
echo
echo "To start the services:"
echo "  docker-compose up -d"
echo
echo "To view logs:"
echo "  docker-compose logs -f"
echo
echo "To stop services:"
echo "  docker-compose down"
echo
echo "Service URLs:"
echo "  API: http://localhost:8080"
echo "  WebSocket: ws://localhost:8080/ws"
echo "  PostgreSQL: localhost:5432"
echo "  Redis: localhost:6379"