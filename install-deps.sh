#!/bin/bash

echo "=== Installing iTrader Backend Dependencies ==="
echo

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    echo "Cannot detect OS"
    exit 1
fi

echo "Detected OS: $OS"
echo

# Install dependencies based on OS
case $OS in
    ubuntu|debian)
        echo "Installing dependencies for Ubuntu/Debian..."
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            pkg-config \
            libssl-dev \
            postgresql \
            postgresql-client \
            redis-server \
            tesseract-ocr \
            tesseract-ocr-rus \
            libtesseract-dev \
            libleptonica-dev \
            libpq-dev \
            clang
        
        # Start services
        echo "Starting PostgreSQL..."
        sudo systemctl start postgresql
        sudo systemctl enable postgresql
        
        echo "Starting Redis..."
        sudo systemctl start redis-server
        sudo systemctl enable redis-server
        ;;
    
    fedora|rhel|centos)
        echo "Installing dependencies for Fedora/RHEL/CentOS..."
        sudo dnf install -y \
            gcc \
            pkg-config \
            openssl-devel \
            postgresql \
            postgresql-server \
            redis \
            tesseract \
            tesseract-langpack-rus \
            postgresql-devel
        
        # Initialize PostgreSQL
        sudo postgresql-setup --initdb
        
        # Start services
        echo "Starting PostgreSQL..."
        sudo systemctl start postgresql
        sudo systemctl enable postgresql
        
        echo "Starting Redis..."
        sudo systemctl start redis
        sudo systemctl enable redis
        ;;
    
    arch|manjaro)
        echo "Installing dependencies for Arch/Manjaro..."
        sudo pacman -S --noconfirm \
            base-devel \
            pkg-config \
            openssl \
            postgresql \
            redis \
            tesseract \
            tesseract-data-rus \
            postgresql-libs
        
        # Initialize PostgreSQL
        sudo -u postgres initdb -D /var/lib/postgres/data
        
        # Start services
        echo "Starting PostgreSQL..."
        sudo systemctl start postgresql
        sudo systemctl enable postgresql
        
        echo "Starting Redis..."
        sudo systemctl start redis
        sudo systemctl enable redis
        ;;
    
    *)
        echo "Unsupported OS: $OS"
        echo "Please install manually:"
        echo "  - OpenSSL development libraries (libssl-dev or openssl-devel)"
        echo "  - PostgreSQL"
        echo "  - Redis"
        echo "  - Tesseract OCR with Russian language pack"
        exit 1
        ;;
esac

echo
echo "Setting up PostgreSQL user..."
# Wait for PostgreSQL to start
sleep 2

# Set password for postgres user
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'root';" 2>/dev/null || {
    echo "Note: Could not set postgres password. You may need to do this manually:"
    echo "  sudo -u postgres psql -c \"ALTER USER postgres PASSWORD 'root';\""
}

echo
echo "Checking services status..."
if systemctl is-active --quiet postgresql; then
    echo "✓ PostgreSQL is running"
else
    echo "✗ PostgreSQL is not running"
fi

if systemctl is-active --quiet redis || systemctl is-active --quiet redis-server; then
    echo "✓ Redis is running"
else
    echo "✗ Redis is not running"
fi

echo
echo "Installation complete!"
echo
echo "You can now run:"
echo "  ./dev.sh    # To start the development server"
echo "  ./test.sh   # To run tests"