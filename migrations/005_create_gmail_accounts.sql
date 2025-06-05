-- Gmail accounts table
CREATE TABLE IF NOT EXISTS gmail_accounts (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    credentials_json TEXT NOT NULL,
    token_json TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    gate_transaction_id VARCHAR(255) UNIQUE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, processing, waiting_payment, waiting_receipt, approved, released, fool_pool, error
    gate_account_id INTEGER REFERENCES gate_accounts(id),
    bybit_account_id INTEGER REFERENCES bybit_accounts(id),
    amount_rub DECIMAL(18, 2) NOT NULL,
    amount_usdt DECIMAL(18, 8),
    wallet VARCHAR(255) NOT NULL, -- Gate wallet/phone/card
    bank_label VARCHAR(255),
    bank_code VARCHAR(50),
    bybit_ad_id VARCHAR(255),
    bybit_order_id VARCHAR(255),
    receipt_id INTEGER,
    chat_stage VARCHAR(50), -- greeting, bank_confirm, receipt_confirm, kyc_confirm, reqs_sent, completed
    error_reason TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    approved_at TIMESTAMP,
    released_at TIMESTAMP,
    release_scheduled_at TIMESTAMP
);

-- Receipts table
CREATE TABLE IF NOT EXISTS receipts (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER REFERENCES transactions(id),
    email_id VARCHAR(255) NOT NULL,
    sender_email VARCHAR(255) NOT NULL,
    subject TEXT,
    pdf_content BYTEA,
    ocr_text TEXT,
    parsed_data JSONB, -- {amount, status, bank, phone, card_last4, datetime}
    is_valid BOOLEAN DEFAULT false,
    validation_errors TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Chat messages log
CREATE TABLE IF NOT EXISTS chat_messages (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER REFERENCES transactions(id),
    order_id VARCHAR(255) NOT NULL,
    direction VARCHAR(10) NOT NULL, -- in/out
    message TEXT NOT NULL,
    message_type VARCHAR(20) DEFAULT 'text', -- text/image/pdf
    sender_nickname VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_gate_id ON transactions(gate_transaction_id);
CREATE INDEX idx_transactions_bybit_order ON transactions(bybit_order_id);
CREATE INDEX idx_receipts_email_id ON receipts(email_id);
CREATE INDEX idx_chat_messages_order ON chat_messages(order_id);