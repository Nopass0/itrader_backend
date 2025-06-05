-- Gmail accounts and credentials
CREATE TABLE IF NOT EXISTS gmail_accounts (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    credentials TEXT NOT NULL,  -- OAuth credentials JSON
    token TEXT,                 -- Current access token
    refresh_token TEXT,         -- Refresh token
    token_expiry TIMESTAMP,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Transactions table with all states
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    gate_transaction_id VARCHAR(255) UNIQUE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- Status values: pending, processing, waiting_response, waiting_payment, 
    -- waiting_receipt, approved, released, fool_pool, error
    
    -- Gate.io data
    gate_account_id INTEGER REFERENCES gate_accounts(id),
    amount_rub DECIMAL(20, 2) NOT NULL,
    wallet VARCHAR(255) NOT NULL,  -- Phone or card number
    bank_label VARCHAR(100),
    bank_code VARCHAR(50),
    
    -- Bybit data
    bybit_account_id INTEGER REFERENCES bybit_accounts(id),
    bybit_ad_id VARCHAR(100),
    bybit_order_id VARCHAR(100),
    
    -- Chat progress
    chat_stage VARCHAR(50),
    -- Stages: greeting, bank_confirm, receipt_confirm, kyc_confirm, 
    -- reqs_sent, waiting_receipt, completed
    
    -- Receipt data
    receipt_id INTEGER,
    
    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    approved_at TIMESTAMP,
    release_scheduled_at TIMESTAMP,
    released_at TIMESTAMP,
    
    -- Error tracking
    error_reason TEXT,
    
    CONSTRAINT fk_gate_account FOREIGN KEY (gate_account_id) 
        REFERENCES gate_accounts(id) ON DELETE SET NULL,
    CONSTRAINT fk_bybit_account FOREIGN KEY (bybit_account_id) 
        REFERENCES bybit_accounts(id) ON DELETE SET NULL
);

-- Receipts table
CREATE TABLE IF NOT EXISTS receipts (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER REFERENCES transactions(id),
    email_id VARCHAR(255) UNIQUE,  -- Gmail message ID
    sender_email VARCHAR(255),
    subject TEXT,
    pdf_content BYTEA,
    ocr_text TEXT,
    parsed_data JSONB,
    is_valid BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Chat messages log
CREATE TABLE IF NOT EXISTS chat_messages (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER REFERENCES transactions(id),
    order_id VARCHAR(100),
    direction VARCHAR(10),  -- 'in' or 'out'
    message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_gate_tx_id ON transactions(gate_transaction_id);
CREATE INDEX idx_transactions_bybit_order_id ON transactions(bybit_order_id);
CREATE INDEX idx_receipts_email_id ON receipts(email_id);
CREATE INDEX idx_chat_messages_transaction_id ON chat_messages(transaction_id);