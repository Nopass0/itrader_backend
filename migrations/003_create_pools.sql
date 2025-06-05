-- Order pools for state recovery
CREATE TABLE order_pools (
    id SERIAL PRIMARY KEY,
    pool_type VARCHAR(50) NOT NULL, -- 'pending', 'active', 'chat', 'verification', 'completed'
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    data JSONB NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- AI conversation history
CREATE TABLE ai_conversations (
    id SERIAL PRIMARY KEY,
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    messages JSONB DEFAULT '[]',
    customer_language VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Email receipts
CREATE TABLE email_receipts (
    id SERIAL PRIMARY KEY,
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    email_from VARCHAR(255) NOT NULL,
    email_subject TEXT,
    receipt_data JSONB NOT NULL,
    ocr_result JSONB,
    is_valid BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_pools_pool_type ON order_pools(pool_type);
CREATE INDEX idx_pools_order_id ON order_pools(order_id);
CREATE INDEX idx_pools_status ON order_pools(status);
CREATE INDEX idx_ai_conversations_order_id ON ai_conversations(order_id);
CREATE INDEX idx_email_receipts_order_id ON email_receipts(order_id);

-- Create triggers
CREATE TRIGGER update_order_pools_updated_at BEFORE UPDATE
    ON order_pools FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ai_conversations_updated_at BEFORE UPDATE
    ON ai_conversations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();