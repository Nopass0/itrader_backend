#!/bin/bash
# Database setup script for P2P Trading System

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}P2P Trading System - Database Setup${NC}"
echo

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo -e "${RED}❌ PostgreSQL not found!${NC}"
    echo
    echo -e "${YELLOW}Please install PostgreSQL first:${NC}"
    echo -e "Ubuntu/Debian: sudo apt-get install postgresql postgresql-client"
    echo -e "MacOS: brew install postgresql"
    echo -e "Then run: brew services start postgresql (MacOS)"
    echo -e "Or: sudo systemctl start postgresql (Linux)"
    exit 1
fi

# Check if PostgreSQL is running
if ! pg_isready -q 2>/dev/null; then
    echo -e "${YELLOW}⚠️  PostgreSQL is not running.${NC}"
    echo -e "Start it with:"
    echo -e "  Linux: sudo systemctl start postgresql"
    echo -e "  MacOS: brew services start postgresql"
    exit 1
fi

echo -e "${GREEN}✅ PostgreSQL is installed and running${NC}"

# Create database
echo -e "\n${BLUE}Creating database...${NC}"

# Try different methods to create database
if sudo -u postgres createdb p2p_trading 2>/dev/null; then
    echo -e "${GREEN}✅ Database created successfully (method 1)${NC}"
elif createdb -U postgres p2p_trading 2>/dev/null; then
    echo -e "${GREEN}✅ Database created successfully (method 2)${NC}"
elif createdb p2p_trading 2>/dev/null; then
    echo -e "${GREEN}✅ Database created successfully (method 3)${NC}"
else
    echo -e "${YELLOW}Database might already exist or require different credentials${NC}"
fi

# Create .pgpass file for passwordless access
echo -e "\n${BLUE}Setting up passwordless access...${NC}"
PGPASS_FILE="$HOME/.pgpass"

if [ ! -f "$PGPASS_FILE" ]; then
    echo -e "${YELLOW}Creating .pgpass file...${NC}"
    echo "localhost:5432:p2p_trading:postgres:postgres" > "$PGPASS_FILE"
    chmod 600 "$PGPASS_FILE"
    echo -e "${GREEN}✅ Created .pgpass file${NC}"
else
    echo -e "${GREEN}✅ .pgpass file already exists${NC}"
fi

# Run migrations
echo -e "\n${BLUE}Running migrations...${NC}"

for migration in migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo -n "  Running $(basename $migration)... "
        if psql -U postgres -d p2p_trading -f "$migration" &>/dev/null; then
            echo -e "${GREEN}✓${NC}"
        else
            echo -e "${YELLOW}already applied or error${NC}"
        fi
    fi
done

# Test connection
echo -e "\n${BLUE}Testing database connection...${NC}"
if psql -U postgres -d p2p_trading -c "SELECT 1" &>/dev/null; then
    echo -e "${GREEN}✅ Database connection successful!${NC}"
    echo -e "\n${GREEN}Database setup complete!${NC}"
    echo -e "You can now run: ${BLUE}./p2p_system.sh${NC}"
else
    echo -e "${RED}❌ Could not connect to database${NC}"
    echo -e "\nTry manual setup:"
    echo -e "1. sudo -u postgres psql"
    echo -e "2. CREATE DATABASE p2p_trading;"
    echo -e "3. \\q"
fi