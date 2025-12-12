-- MMO Game Microtransaction Schema
-- Compatible with both Node.js and Rust versions

-- Create custom enum type for transaction status
CREATE TYPE transaction_status AS ENUM ('pending', 'completed', 'failed', 'refunded');

-- Main transactions table
CREATE TABLE IF NOT EXISTS microtransactions (
    -- Primary key: UUID for distributed systems compatibility
    transaction_id UUID PRIMARY KEY,
    
    -- Player reference (from game's player service)
    player_id UUID NOT NULL,
    
    -- Item information
    item_id VARCHAR(255) NOT NULL,
    item_name VARCHAR(255) NOT NULL,
    
    -- Pricing (stored in cents to avoid floating point issues)
    price_cents BIGINT NOT NULL CHECK (price_cents > 0 AND price_cents <= 99999999),
    currency CHAR(3) NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1 CHECK (quantity > 0 AND quantity <= 100),
    
    -- Transaction status
    status transaction_status NOT NULL DEFAULT 'pending',
    
    -- Payment processor reference
    processor_id VARCHAR(255),
    
    -- Flexible metadata (item stats, bonuses, etc.)
    metadata JSONB NOT NULL DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common query patterns
CREATE INDEX idx_microtx_player_id ON microtransactions(player_id);
CREATE INDEX idx_microtx_player_created ON microtransactions(player_id, created_at DESC);
CREATE INDEX idx_microtx_status ON microtransactions(status) WHERE status = 'pending';
CREATE INDEX idx_microtx_processor_id ON microtransactions(processor_id) WHERE processor_id IS NOT NULL;

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_microtransactions_updated_at
    BEFORE UPDATE ON microtransactions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE microtransactions IS 'MMO game micro-transaction records';
COMMENT ON COLUMN microtransactions.price_cents IS 'Price in smallest currency unit (cents)';
COMMENT ON COLUMN microtransactions.metadata IS 'Flexible JSON for item properties, bonuses, etc.';
