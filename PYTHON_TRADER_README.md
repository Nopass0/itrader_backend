# Auto-Trader System for Gate.io ↔ Bybit P2P

A fully automated trading system that monitors Gate.io for pending transactions and creates corresponding Bybit P2P advertisements with intelligent rate calculation and automated chat flow management.

## Features

### Core Functionality
- **Gate.io Monitoring**: Continuously monitors pending transactions using cookie authentication
- **Bybit P2P Integration**: Creates advertisements with calculated rates
- **Automated Chat Flow**: Manages buyer interactions in Russian
- **Receipt Validation**: OCR processing of PDF receipts from Tinkoff
- **Email Monitoring**: Monitors email for payment receipts from noreply@tinkoff.ru
- **Rate Calculation**: Intelligent pricing based on market rates and profit margins

### Chat Flow
1. Greets buyer: "Добрый день! Вы прочитали условия объявления и правила P2P?"
2. Waits for agreement (keywords: да, согласен, прочитал, etc.)
3. Shows payment details (bank, phone, amount)
4. Reminds to send receipt to specified email
5. Sends reminders at 5 and 10 minutes
6. Completes transaction after receipt validation

### Operation Modes
- **Manual Mode**: User confirmation required for each action
- **Automatic Mode**: Fully automated operation

## Installation

### Prerequisites
- Python 3.8 or higher
- Gate.io cookies (`.gate_cookies.json`)
- Bybit API credentials
- Email account for receipt monitoring
- Tesseract OCR (optional, for image receipts)

### Setup

1. **Clone and setup**:
```bash
chmod +x setup.sh
./setup.sh
```

2. **Configure credentials**:
```bash
cp .env.example .env
# Edit .env with your credentials
```

3. **Install Tesseract OCR** (optional):
```bash
# Ubuntu/Debian
sudo apt-get install tesseract-ocr tesseract-ocr-rus

# macOS
brew install tesseract tesseract-lang
```

## Configuration

### Environment Variables (.env)
```env
# Bybit API
BYBIT_API_KEY=your_api_key
BYBIT_API_SECRET=your_api_secret
BYBIT_TESTNET=false

# Payment Details
PAYMENT_BANK=Тинькофф
PAYMENT_PHONE=+7 XXX XXX-XX-XX
RECEIPT_EMAIL=your-receipt@email.com

# Email Monitoring
EMAIL_USERNAME=your-email@gmail.com
EMAIL_PASSWORD=your-app-password
```

### Configuration File (config/default.toml)
- Gate.io polling interval
- Trading parameters (margins, limits)
- Email monitoring settings
- Logging configuration

### Gate.io Cookies
Create `.gate_cookies.json` with cookies exported from your browser:
```json
[
  {
    "domain": ".panel.gate.cx",
    "name": "sid",
    "value": "your_session_id",
    ...
  }
]
```

## Usage

### Run in Manual Mode (default)
```bash
./run.sh
# or
./run.sh --mode manual
```

### Run in Automatic Mode
```bash
./run.sh --mode automatic
```

### Command Line Options
```bash
python trader_system.py --help

Options:
  --mode [manual|automatic]  Operation mode (default: manual)
  --config TEXT             Configuration file path
```

## File Structure

```
├── trader_system.py      # Main application
├── gate_client.py        # Gate.io API client
├── bybit_client.py       # Bybit P2P client
├── chat_manager.py       # Automated chat management
├── email_monitor.py      # Email receipt monitoring
├── ocr_processor.py      # OCR for receipt validation
├── config.py            # Configuration management
├── models.py            # Data models
├── utils.py             # Utility functions
├── requirements.txt     # Python dependencies
├── setup.sh            # Setup script
├── run.sh              # Run script
├── .env.example        # Environment template
└── config/
    └── default.toml    # Default configuration
```

## Workflow

1. **Transaction Detection**:
   - Monitors Gate.io for pending transactions
   - Extracts amount, currency, rate, buyer info

2. **Rate Calculation**:
   - Fetches current Bybit P2P market rates
   - Applies profit margin
   - Calculates optimal price

3. **Advertisement Creation**:
   - Creates P2P ad on Bybit
   - Sets payment methods (Tinkoff, SBP)
   - Adds remarks about email receipt

4. **Chat Management**:
   - Sends greeting message
   - Waits for buyer agreement
   - Shows payment details after confirmation
   - Sends periodic reminders

5. **Receipt Processing**:
   - Monitors email for receipts
   - Validates sender (noreply@tinkoff.ru)
   - Extracts amount from PDF
   - Matches with active orders

6. **Transaction Completion**:
   - Releases funds on Bybit
   - Completes transaction on Gate.io
   - Sends completion message

## Security

- API credentials stored in environment variables
- Gate.io authentication via secure cookies
- Email app passwords (not main password)
- Rate limiting to prevent API abuse

## Logging

- Console output with colors
- File logging to `trader.log`
- Configurable log levels

## Error Handling

- Graceful degradation on component failures
- Automatic reconnection for email/API
- Transaction rollback on errors
- Detailed error logging

## Development

### Running Tests
```bash
source venv/bin/activate
pytest tests/
```

### Code Style
```bash
black .
flake8 .
```

## Troubleshooting

### Gate.io Connection Issues
- Ensure cookies are fresh (re-export if needed)
- Check cookie format in `.gate_cookies.json`

### Bybit API Errors
- Verify API key permissions
- Check testnet vs mainnet settings
- Ensure time sync (NTP)

### Email Monitoring
- Use app-specific password for Gmail
- Enable IMAP in email settings
- Check firewall for port 993

### OCR Issues
- Install Tesseract with Russian language support
- Ensure PDF receipts are readable
- Check receipt format matches patterns

## Support

For issues or questions:
1. Check logs in `trader.log`
2. Verify configuration
3. Test components individually
4. Enable debug logging