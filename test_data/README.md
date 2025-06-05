# Test Data Directory

This directory contains test credentials and data for running integration tests.

## Bybit Credentials

### Mainnet Credentials
- File: `bybit_creditials.json`
- Used by default for Bybit tests
- Generate API keys from: https://www.bybit.com/app/user/api-management

### Testnet Credentials
- File: `bybit_testnet_creditials.json`
- Used when running tests with `--testnet` flag
- Generate testnet API keys from: https://testnet.bybit.com/app/user/api-management

### Running Tests

```bash
# Run tests with mainnet credentials
./test.sh bybit-all

# Run tests with testnet credentials
./test.sh bybit-all --testnet

# Run specific test with testnet
./test.sh bybit-auth --testnet
```

## Gate.io Credentials

- File: `gate_creditials.json` - Contains Gate.io login credentials
- File: `gate_cookie.json` - Stores authentication cookies after successful login

## Important Notes

1. **Never commit real credentials to version control**
2. All credential files in this directory should be added to `.gitignore`
3. For testnet testing, you need to create separate API keys on Bybit's testnet platform
4. Testnet and mainnet API keys are not interchangeable