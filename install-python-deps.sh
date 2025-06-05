#!/bin/bash

echo "=== Installing Python dependencies for Bybit SDK ==="

# Install using pip with --break-system-packages flag for modern systems
echo "Installing Bybit SDK and dependencies..."

python3 -m pip install --user --break-system-packages pybit==5.7.0
python3 -m pip install --user --break-system-packages python-dotenv==1.0.0
python3 -m pip install --user --break-system-packages aiohttp==3.9.1

# Create requirements.txt
cat > requirements.txt << EOF
pybit==5.7.0
python-dotenv==1.0.0
aiohttp==3.9.1
EOF

# Create Python modules directory
mkdir -p python_modules

echo "=== Python dependencies installed ==="
echo ""
echo "Dependencies have been installed to user site-packages"
echo "Python modules directory created at: python_modules/"