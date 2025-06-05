#!/bin/bash

echo "=== Setting up Python environment for Bybit SDK ==="

# Check if Python 3.9+ is installed
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is not installed"
    exit 1
fi

PYTHON_VERSION=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
REQUIRED_VERSION="3.9"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$PYTHON_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    echo "Error: Python $REQUIRED_VERSION or higher is required, but $PYTHON_VERSION is installed"
    exit 1
fi

echo "Python $PYTHON_VERSION detected"

# Check if we can use venv, otherwise use system packages
if python3 -m venv --help &> /dev/null; then
    # Create virtual environment
    if [ ! -d "venv" ]; then
        echo "Creating Python virtual environment..."
        python3 -m venv venv --without-pip
        
        # Install pip in venv
        curl -sS https://bootstrap.pypa.io/get-pip.py | venv/bin/python3
    else
        echo "Virtual environment already exists"
    fi
    
    # Use venv pip
    PIP_CMD="venv/bin/pip"
    PYTHON_CMD="venv/bin/python3"
else
    echo "Warning: Cannot create virtual environment, using user install"
    PIP_CMD="python3 -m pip"
    PYTHON_CMD="python3"
    
    # Ensure pip is installed for user
    python3 -m ensurepip --user 2>/dev/null || true
fi

# Upgrade pip
echo "Upgrading pip..."
$PIP_CMD install --user --upgrade pip 2>/dev/null || $PIP_CMD install --upgrade pip

# Install Bybit SDK and other required packages
echo "Installing Bybit SDK and dependencies..."
$PIP_CMD install --user pybit==5.7.0 2>/dev/null || $PIP_CMD install pybit==5.7.0
$PIP_CMD install --user python-dotenv==1.0.0 2>/dev/null || $PIP_CMD install python-dotenv==1.0.0
$PIP_CMD install --user aiohttp==3.9.1 2>/dev/null || $PIP_CMD install aiohttp==3.9.1

# Create requirements.txt for future use
cat > requirements.txt << EOF
pybit==5.7.0
python-dotenv==1.0.0
aiohttp==3.9.1
EOF

echo "Python requirements saved to requirements.txt"

# Create Python modules directory
mkdir -p python_modules

echo "=== Python environment setup complete ==="
echo ""
echo "To activate the virtual environment, run:"
echo "  source venv/bin/activate"
echo ""
echo "To install requirements in the future, run:"
echo "  pip install -r requirements.txt"