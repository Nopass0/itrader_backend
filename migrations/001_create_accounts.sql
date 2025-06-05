-- Gate.io accounts
CREATE TABLE gate_accounts (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    cookies JSONB,
    last_auth TIMESTAMP WITH TIME ZONE,
    balance DECIMAL(20, 2) DEFAULT 0,
    status VARCHAR(50) DEFAULT 'inactive',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Bybit accounts
CREATE TABLE bybit_accounts (
    id SERIAL PRIMARY KEY,
    account_name VARCHAR(255) UNIQUE NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    api_secret TEXT NOT NULL,
    active_ads INTEGER DEFAULT 0,
    status VARCHAR(50) DEFAULT 'available',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_gate_accounts_status ON gate_accounts(status);
CREATE INDEX idx_gate_accounts_email ON gate_accounts(email);
CREATE INDEX idx_bybit_accounts_status ON bybit_accounts(status);
CREATE INDEX idx_bybit_accounts_active_ads ON bybit_accounts(active_ads);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers
CREATE TRIGGER update_gate_accounts_updated_at BEFORE UPDATE
    ON gate_accounts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_bybit_accounts_updated_at BEFORE UPDATE
    ON bybit_accounts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();