-- Create Gate.io accounts table
CREATE TABLE IF NOT EXISTS gate_accounts (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    balance DECIMAL(20, 2) DEFAULT 0.00,
    status VARCHAR(50) DEFAULT 'active',
    cookies JSONB,
    last_auth TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index on email for faster lookups
CREATE INDEX idx_gate_accounts_email ON gate_accounts(email);

-- Create index on status for filtering active accounts
CREATE INDEX idx_gate_accounts_status ON gate_accounts(status);