#!/usr/bin/env python3
"""
Simple startup script for P2P Trading System
"""

import os
import sys

# Add project root to path
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

# Set default database URL if not set
if not os.environ.get('DATABASE_URL'):
    os.environ['DATABASE_URL'] = 'postgresql://postgres:postgres@localhost:5432/p2p_trading'

# Import and run the launcher
from src.main_launcher import main

if __name__ == "__main__":
    main()