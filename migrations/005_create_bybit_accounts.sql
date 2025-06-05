-- Create Bybit accounts table
CREATE TABLE IF NOT EXISTS bybit_accounts (
    id SERIAL PRIMARY KEY,
    account_name VARCHAR(255) NOT NULL UNIQUE,
    api_key VARCHAR(255) NOT NULL,
    api_secret VARCHAR(255) NOT NULL,
    active_ads INTEGER DEFAULT 0,
    status VARCHAR(50) DEFAULT 'available',
    last_used TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index on account_name for faster lookups
CREATE INDEX idx_bybit_accounts_name ON bybit_accounts(account_name);

-- Create index on status for filtering available accounts
CREATE INDEX idx_bybit_accounts_status ON bybit_accounts(status);

-- Create index on active_ads for finding accounts with capacity
CREATE INDEX idx_bybit_accounts_active_ads ON bybit_accounts(active_ads);