#!/bin/bash
# P2P Trading System - Startup Script

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Project directory
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

# Print banner
echo -e "${BLUE}"
echo "╔══════════════════════════════════════════╗"
echo "║     P2P Trading Automation System        ║"
echo "║        Gate.io → Bybit Trading           ║"
echo "╚══════════════════════════════════════════╝"
echo -e "${NC}"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 not found. Please install Python 3.8+${NC}"
    exit 1
fi

# Check UV
if ! command -v uv &> /dev/null; then
    echo -e "${YELLOW}⚠️  UV not found. Installing UV...${NC}"
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
fi

# Create virtual environment if needed
if [ ! -d ".venv" ]; then
    echo -e "${YELLOW}Creating virtual environment...${NC}"
    uv venv
fi

# Activate virtual environment
source .venv/bin/activate

# Install dependencies
echo -e "${BLUE}Installing dependencies...${NC}"
uv pip sync requirements.txt 2>/dev/null || uv pip install -r requirements.txt

# Check if PostgreSQL is running
if ! command -v psql &> /dev/null; then
    echo -e "${RED}❌ PostgreSQL client not found. Please install PostgreSQL.${NC}"
    echo -e "   Ubuntu/Debian: sudo apt-get install postgresql postgresql-client"
    echo -e "   MacOS: brew install postgresql"
    exit 1
fi

# Try different connection methods
DB_EXISTS=false
DATABASE_URL=""

# Method 1: Try with .pgpass file
if [ -f "$HOME/.pgpass" ]; then
    if psql -U postgres -h localhost -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw p2p_trading; then
        DB_EXISTS=true
        DATABASE_URL="postgresql://postgres@localhost:5432/p2p_trading"
    fi
fi

# Method 2: Try with PGPASSWORD environment variable
if [ "$DB_EXISTS" = false ] && [ -n "$PGPASSWORD" ]; then
    export PGPASSWORD
    if psql -U postgres -h localhost -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw p2p_trading; then
        DB_EXISTS=true
        DATABASE_URL="postgresql://postgres:$PGPASSWORD@localhost:5432/p2p_trading"
    fi
fi

# Method 3: Try with peer authentication (local socket)
if [ "$DB_EXISTS" = false ]; then
    if psql -U $USER -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw p2p_trading; then
        DB_EXISTS=true
        DATABASE_URL="postgresql://$USER@localhost:5432/p2p_trading"
    fi
fi

# Method 4: Try to use SQLite as fallback
if [ "$DB_EXISTS" = false ]; then
    echo -e "${YELLOW}⚠️  Cannot connect to PostgreSQL.${NC}"
    echo -e "${BLUE}Options:${NC}"
    echo -e "1. Setup PostgreSQL (recommended)"
    echo -e "2. Use SQLite (limited functionality)"
    echo -e "3. Enter custom database URL"
    echo -e "4. Exit"
    
    read -p "Select option (1-4): " choice
    
    case $choice in
        1)
            echo -e "${BLUE}To setup PostgreSQL:${NC}"
            echo -e "1. Install PostgreSQL: sudo apt-get install postgresql"
            echo -e "2. Start service: sudo systemctl start postgresql"
            echo -e "3. Create user: sudo -u postgres createuser -s $USER"
            echo -e "4. Run this script again"
            exit 0
            ;;
        2)
            echo -e "${YELLOW}Using SQLite database (some features may be limited)${NC}"
            DATABASE_URL="sqlite:///p2p_trading.db"
            DB_EXISTS=true
            ;;
        3)
            read -p "Enter database URL: " custom_url
            DATABASE_URL="$custom_url"
            DB_EXISTS=true
            ;;
        4)
            exit 0
            ;;
    esac
fi

# If database doesn't exist, offer to create it
if [ "$DB_EXISTS" = false ]; then
    echo -e "${YELLOW}Database 'p2p_trading' not found.${NC}"
    echo -e "Would you like to create it? (y/n)"
    read -r response
    if [[ "$response" == "y" ]]; then
        # Try to create database
        if [ -n "$PGPASSWORD" ]; then
            createdb -U postgres -h localhost p2p_trading 2>/dev/null
        else
            createdb -U $USER p2p_trading 2>/dev/null
        fi
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ Database created${NC}"
            DB_EXISTS=true
            
            # Run migrations
            echo -e "${BLUE}Running migrations...${NC}"
            for migration in migrations/*.sql; do
                if [[ ! "$DATABASE_URL" =~ sqlite ]]; then
                    psql "$DATABASE_URL" -f "$migration" 2>&1 | grep -v "already exists" | grep -v "NOTICE"
                fi
            done
            echo -e "${GREEN}✅ Migrations completed${NC}"
        else
            echo -e "${RED}Failed to create database.${NC}"
            echo -e "Please create it manually: createdb p2p_trading"
            exit 1
        fi
    else
        echo -e "${YELLOW}Database required. Exiting.${NC}"
        exit 1
    fi
fi

# Export database URL
export DATABASE_URL="${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/p2p_trading}"

# Run the launcher
echo -e "${GREEN}Starting P2P Trading System...${NC}"
echo

# Make sure we're in virtual environment
if [[ "$VIRTUAL_ENV" != "" ]]; then
    python src/main_launcher.py
else
    .venv/bin/python src/main_launcher.py
fi