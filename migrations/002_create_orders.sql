-- Orders tracking
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gate_transaction_id VARCHAR(255) UNIQUE NOT NULL,
    bybit_order_id VARCHAR(255),
    gate_account_id INTEGER REFERENCES gate_accounts(id),
    bybit_account_id INTEGER REFERENCES bybit_accounts(id),
    amount DECIMAL(20, 8) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    fiat_currency VARCHAR(10) NOT NULL,
    rate DECIMAL(10, 4) NOT NULL,
    total_fiat DECIMAL(20, 2) NOT NULL,
    status VARCHAR(50) NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Order status history
CREATE TABLE order_status_history (
    id SERIAL PRIMARY KEY,
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL,
    details JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);
CREATE INDEX idx_orders_gate_transaction_id ON orders(gate_transaction_id);
CREATE INDEX idx_orders_bybit_order_id ON orders(bybit_order_id);
CREATE INDEX idx_order_status_history_order_id ON order_status_history(order_id);

-- Create trigger for updated_at
CREATE TRIGGER update_orders_updated_at BEFORE UPDATE
    ON orders FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();