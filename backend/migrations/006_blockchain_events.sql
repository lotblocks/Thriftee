-- Migration for blockchain event processing tables

-- Event processor state table to track last processed block
CREATE TABLE IF NOT EXISTS event_processor_state (
    id INTEGER PRIMARY KEY DEFAULT 1,
    last_processed_block BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT single_row CHECK (id = 1)
);

-- Blockchain events table for storing all processed events
CREATE TABLE IF NOT EXISTS blockchain_events (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    raffle_id BIGINT,
    block_number BIGINT NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    data JSONB NOT NULL,
    processed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint to prevent duplicate events
    UNIQUE(transaction_hash, event_type, raffle_id)
);

-- Box purchases table for tracking individual box purchases
CREATE TABLE IF NOT EXISTS box_purchases (
    id BIGSERIAL PRIMARY KEY,
    raffle_id BIGINT NOT NULL REFERENCES raffles(id) ON DELETE CASCADE,
    buyer_address VARCHAR(42) NOT NULL,
    box_number INTEGER NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    purchased_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint to prevent duplicate box purchases
    UNIQUE(raffle_id, box_number)
);

-- Raffle winners table for tracking selected winners
CREATE TABLE IF NOT EXISTS raffle_winners (
    id BIGSERIAL PRIMARY KEY,
    raffle_id BIGINT NOT NULL REFERENCES raffles(id) ON DELETE CASCADE,
    winner_address VARCHAR(42) NOT NULL,
    winner_index INTEGER NOT NULL,
    selected_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint to prevent duplicate winners
    UNIQUE(raffle_id, winner_index)
);

-- Add blockchain_raffle_id to raffles table if it doesn't exist
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'raffles' AND column_name = 'blockchain_raffle_id') THEN
        ALTER TABLE raffles ADD COLUMN blockchain_raffle_id BIGINT UNIQUE;
    END IF;
END $$;

-- Add indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_blockchain_events_raffle_id ON blockchain_events(raffle_id);
CREATE INDEX IF NOT EXISTS idx_blockchain_events_block_number ON blockchain_events(block_number);
CREATE INDEX IF NOT EXISTS idx_blockchain_events_timestamp ON blockchain_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_blockchain_events_event_type ON blockchain_events(event_type);

CREATE INDEX IF NOT EXISTS idx_box_purchases_raffle_id ON box_purchases(raffle_id);
CREATE INDEX IF NOT EXISTS idx_box_purchases_buyer_address ON box_purchases(buyer_address);
CREATE INDEX IF NOT EXISTS idx_box_purchases_purchased_at ON box_purchases(purchased_at);

CREATE INDEX IF NOT EXISTS idx_raffle_winners_raffle_id ON raffle_winners(raffle_id);
CREATE INDEX IF NOT EXISTS idx_raffle_winners_winner_address ON raffle_winners(winner_address);
CREATE INDEX IF NOT EXISTS idx_raffle_winners_selected_at ON raffle_winners(selected_at);

CREATE INDEX IF NOT EXISTS idx_raffles_blockchain_raffle_id ON raffles(blockchain_raffle_id) WHERE blockchain_raffle_id IS NOT NULL;

-- Insert initial state if not exists
INSERT INTO event_processor_state (id, last_processed_block) 
VALUES (1, 0) 
ON CONFLICT (id) DO NOTHING;

-- Add comments for documentation
COMMENT ON TABLE event_processor_state IS 'Tracks the last processed blockchain block for event monitoring';
COMMENT ON TABLE blockchain_events IS 'Stores all processed blockchain events from the raffle contract';
COMMENT ON TABLE box_purchases IS 'Tracks individual box purchases in raffles';
COMMENT ON TABLE raffle_winners IS 'Tracks selected winners for completed raffles';

COMMENT ON COLUMN blockchain_events.event_type IS 'Type of event: raffle_created, box_purchased, winner_selected, etc.';
COMMENT ON COLUMN blockchain_events.data IS 'Full event data in JSON format';
COMMENT ON COLUMN box_purchases.buyer_address IS 'Ethereum address of the box buyer';
COMMENT ON COLUMN raffle_winners.winner_address IS 'Ethereum address of the raffle winner';
COMMENT ON COLUMN raffle_winners.winner_index IS 'Index of the winner (for multiple winner raffles)';

-- Create a view for easy event querying
CREATE OR REPLACE VIEW raffle_events AS
SELECT 
    r.id as raffle_id,
    r.item_id,
    r.title,
    be.event_type,
    be.block_number,
    be.transaction_hash,
    be.timestamp,
    be.data,
    be.processed_at
FROM raffles r
JOIN blockchain_events be ON r.blockchain_raffle_id = be.raffle_id
ORDER BY be.timestamp DESC;

COMMENT ON VIEW raffle_events IS 'Convenient view joining raffles with their blockchain events';