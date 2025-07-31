-- Add free item redemptions table
CREATE TABLE free_item_redemptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    free_item_id UUID REFERENCES free_redeemable_items(id) ON DELETE CASCADE NOT NULL,
    credits_used DECIMAL(10,2) NOT NULL,
    quantity_redeemed INTEGER NOT NULL DEFAULT 1,
    shipping_address JSONB,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add indexes for free item redemptions
CREATE INDEX idx_free_item_redemptions_user_id ON free_item_redemptions(user_id);
CREATE INDEX idx_free_item_redemptions_free_item_id ON free_item_redemptions(free_item_id);
CREATE INDEX idx_free_item_redemptions_status ON free_item_redemptions(status);
CREATE INDEX idx_free_item_redemptions_created_at ON free_item_redemptions(created_at);

-- Add trigger for updated_at
CREATE TRIGGER update_free_item_redemptions_updated_at 
    BEFORE UPDATE ON free_item_redemptions 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();