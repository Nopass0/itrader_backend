-- Create settings table
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY DEFAULT 1,
    admin_token VARCHAR(255),
    balance_update_interval INTEGER DEFAULT 14400,
    gate_relogin_interval INTEGER DEFAULT 1800,
    rate_limit_per_minute INTEGER DEFAULT 240,
    payment_methods TEXT[] DEFAULT ARRAY['SBP', 'Tinkoff'],
    alternate_payments BOOLEAN DEFAULT true,
    ocr_validation BOOLEAN DEFAULT true,
    cleanup_days INTEGER DEFAULT 30,
    receipt_email VARCHAR(255) DEFAULT 'noreply@tinkoff.ru',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT settings_single_row CHECK (id = 1)
);

-- Create gate_cookies table for storing session data
CREATE TABLE IF NOT EXISTS gate_cookies (
    id VARCHAR(50) PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    password_encrypted TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'inactive',
    cookies TEXT,
    last_auth TIMESTAMP,
    balance DECIMAL(15,2) DEFAULT 10000000.00,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create bybit_sessions table for storing session data
CREATE TABLE IF NOT EXISTS bybit_sessions (
    id VARCHAR(50) PRIMARY KEY,
    api_key VARCHAR(255) NOT NULL,
    api_secret TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    testnet BOOLEAN DEFAULT false,
    active_ads INTEGER DEFAULT 0,
    last_error TEXT,
    last_login TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indices
CREATE INDEX IF NOT EXISTS idx_gate_cookies_email ON gate_cookies(email);
CREATE INDEX IF NOT EXISTS idx_bybit_sessions_api_key ON bybit_sessions(api_key);

-- Create update trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_settings_updated_at BEFORE UPDATE ON settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_gate_cookies_updated_at BEFORE UPDATE ON gate_cookies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_bybit_sessions_updated_at BEFORE UPDATE ON bybit_sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();