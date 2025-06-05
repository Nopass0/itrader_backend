# iTrader Backend Installation Guide

This guide provides instructions for setting up the iTrader Backend on a fresh Ubuntu/Debian system.

## Quick Start (Recommended)

For a complete one-command installation on a fresh machine:

```bash
git clone <repository-url>
cd itrader_backend
./quick-setup.sh
```

This will install all dependencies, set up the database, and optionally start the server.

## Manual Installation

If you prefer to run the installation step by step:

### 1. Install System Dependencies

```bash
./install.sh
```

This script will:
- Install PostgreSQL and create the database
- Install Redis server
- Install Rust toolchain and sqlx-cli
- Install Python 3 and required packages
- Install system dependencies (Tesseract OCR, etc.)
- Create necessary directories
- Run database migrations
- Build the Rust project

### 2. Configure Environment

After installation, update the `.env` file with your API keys:

```bash
nano .env
```

Update these values:
- `OPENROUTER_API_KEY`: Your OpenRouter API key
- `EMAIL_ADDRESS`: Your Gmail address (optional)
- `EMAIL_PASSWORD`: Your Gmail app password (optional)
- `ADMIN_TOKEN`: Change from default for production

### 3. Start the Server

```bash
./run.sh
```

The script will check all dependencies before starting and warn you if anything is missing.

## System Requirements

- Ubuntu/Debian-based Linux distribution
- At least 2GB RAM
- 10GB free disk space
- Internet connection for downloading dependencies

## Installed Components

### System Packages
- PostgreSQL (database)
- Redis (caching and pub/sub)
- Rust (application runtime)
- Python 3 (Bybit P2P integration)
- Tesseract OCR (receipt processing)

### Database
- Database name: `itrader`
- User: `postgres`
- Password: `root`

### Services
- Redis: `redis://localhost:6379`
- PostgreSQL: `postgresql://postgres:root@localhost/itrader`
- API Server: `http://localhost:8080`
- WebSocket: `ws://localhost:8080/ws`

## Running as a Service

To run iTrader Backend as a system service:

```bash
# Start the service
sudo systemctl start itrader-backend

# Enable on boot
sudo systemctl enable itrader-backend

# Check status
sudo systemctl status itrader-backend

# View logs
sudo journalctl -u itrader-backend -f
```

## Troubleshooting

### PostgreSQL Connection Issues

If you get PostgreSQL connection errors:

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Restart PostgreSQL
sudo systemctl restart postgresql

# Check PostgreSQL logs
sudo tail -f /var/log/postgresql/*.log
```

### Redis Connection Issues

If Redis is not accessible:

```bash
# Check if Redis is running
sudo systemctl status redis-server

# Restart Redis
sudo systemctl restart redis-server

# Test Redis connection
redis-cli ping
```

### Python Dependencies

If Bybit P2P features are not working:

```bash
# Activate virtual environment
source venv/bin/activate

# Reinstall dependencies
pip install -r requirements.txt

# Check Python library path
python3 -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))'
```

### Missing Dependencies

If `./run.sh` reports missing dependencies:

```bash
# Re-run the installation script
./install.sh

# Or install specific components:
# For Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# For sqlx: cargo install sqlx-cli --no-default-features --features postgres
```

## Development Mode

For development with auto-reload:

```bash
# Manual mode (requires confirmation for actions)
./run.sh

# Automatic mode (no confirmations)
./run.sh --auto

# Account management
./run.sh --settings
```

## Security Notes

1. Change default passwords in production
2. Update the `ADMIN_TOKEN` in `.env`
3. Use strong passwords for PostgreSQL
4. Consider using environment-specific config files in `config/`
5. Keep API keys secure and never commit them to git

## Next Steps

1. Configure accounts using `./run.sh --settings`
2. Test the WebSocket API at `ws://localhost:8080/ws`
3. Check admin endpoints at `http://localhost:8080/admin`
4. Review logs in the `logs/` directory