#!/bin/bash
# Main entry point for P2P Trading System

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

clear

echo -e "${BLUE}"
echo "╔══════════════════════════════════════════╗"
echo "║     P2P Trading Automation System        ║"
echo "║        Gate.io → Bybit Trading           ║"
echo "╚══════════════════════════════════════════╝"
echo -e "${NC}"

# Check if PostgreSQL is available
if command -v psql &> /dev/null && pg_isready -q 2>/dev/null; then
    echo -e "${GREEN}✅ PostgreSQL detected${NC}"
    echo
    
    # Check if database exists
    if psql -U postgres -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw p2p_trading || \
       psql -U $USER -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw p2p_trading; then
        echo -e "${GREEN}✅ Database 'p2p_trading' found${NC}"
        echo -e "\nStarting full system..."
        sleep 1
        exec ./p2p_system.sh
    else
        echo -e "${YELLOW}⚠️  Database 'p2p_trading' not found${NC}"
        echo
        echo "Options:"
        echo "1. Setup database automatically"
        echo "2. Run demo mode (no database needed)"
        echo "3. Exit"
        
        read -p "Select option (1-3): " choice
        
        case $choice in
            1)
                echo -e "\n${BLUE}Setting up database...${NC}"
                ./setup_database.sh
                echo -e "\n${GREEN}Starting system...${NC}"
                exec ./p2p_system.sh
                ;;
            2)
                echo -e "\n${YELLOW}Starting demo mode...${NC}"
                exec python demo_mode.py
                ;;
            3)
                exit 0
                ;;
        esac
    fi
else
    echo -e "${YELLOW}⚠️  PostgreSQL not detected${NC}"
    echo
    echo "Options:"
    echo "1. View PostgreSQL setup instructions"
    echo "2. Run demo mode (no database needed)"
    echo "3. Exit"
    
    read -p "Select option (1-3): " choice
    
    case $choice in
        1)
            echo -e "\n${BLUE}PostgreSQL Setup Instructions:${NC}"
            echo
            echo "Ubuntu/Debian:"
            echo "  sudo apt-get install postgresql postgresql-client"
            echo "  sudo systemctl start postgresql"
            echo
            echo "MacOS:"
            echo "  brew install postgresql"
            echo "  brew services start postgresql"
            echo
            echo "After installation, run this script again."
            ;;
        2)
            echo -e "\n${YELLOW}Starting demo mode...${NC}"
            exec python demo_mode.py
            ;;
        3)
            exit 0
            ;;
    esac
fi