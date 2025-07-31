-- Create custom types
CREATE TYPE user_role AS ENUM ('user', 'seller', 'admin', 'operator');
CREATE TYPE item_status AS ENUM ('available', 'sold', 'inactive');
CREATE TYPE raffle_status AS ENUM ('open', 'full', 'drawing', 'completed', 'cancelled');
CREATE TYPE credit_source AS ENUM ('raffle_loss', 'deposit', 'refund', 'bonus');
CREATE TYPE credit_type AS ENUM ('general', 'item_specific');
CREATE TYPE transaction_type AS ENUM (
    'credit_deposit', 
    'credit_withdrawal', 
    'box_purchase_credit_deduction', 
    'item_purchase_credit_deduction', 
    'raffle_win_credit_addition', 
    'payout', 
    'free_item_redemption', 
    'seller_subscription_fee', 
    'seller_listing_fee', 
    'seller_transaction_fee'
);

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    role user_role NOT NULL DEFAULT 'user',
    credit_balance DECIMAL(10,2) DEFAULT 0.00,
    internal_wallet_address VARCHAR(42) UNIQUE NOT NULL,
    internal_wallet_private_key_encrypted TEXT NOT NULL,
    phone_number VARCHAR(20),
    google_id VARCHAR(255),
    apple_id VARCHAR(255),
    is_active BOOLEAN DEFAULT TRUE,
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Seller subscriptions table
CREATE TABLE seller_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    monthly_fee DECIMAL(10,2) NOT NULL,
    listing_fee_percentage DECIMAL(5,2) DEFAULT 0.00,
    transaction_fee_percentage DECIMAL(5,2) DEFAULT 0.00,
    max_listings INTEGER,
    features JSONB,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Sellers table
CREATE TABLE sellers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    company_name VARCHAR(255),
    description TEXT,
    payout_details JSONB,
    current_subscription_id UUID REFERENCES seller_subscriptions(id),
    subscription_expires_at TIMESTAMP WITH TIME ZONE,
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Items table
CREATE TABLE items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id UUID REFERENCES sellers(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    images TEXT[] NOT NULL,
    retail_price DECIMAL(10,2) NOT NULL,
    cost_of_goods DECIMAL(10,2) NOT NULL,
    status item_status DEFAULT 'available',
    stock_quantity INTEGER DEFAULT 1,
    listing_fee_applied DECIMAL(10,2),
    listing_fee_type VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Raffles table
CREATE TABLE raffles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID REFERENCES items(id) NOT NULL,
    total_boxes INTEGER NOT NULL,
    box_price DECIMAL(10,2) NOT NULL,
    boxes_sold INTEGER DEFAULT 0,
    total_winners INTEGER NOT NULL DEFAULT 1,
    status raffle_status DEFAULT 'open',
    winner_user_ids UUID[],
    blockchain_tx_hash VARCHAR(66),
    grid_rows INTEGER NOT NULL,
    grid_cols INTEGER NOT NULL,
    transaction_fee_applied DECIMAL(10,2),
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Box purchases table
CREATE TABLE box_purchases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    raffle_id UUID REFERENCES raffles(id) NOT NULL,
    user_id UUID REFERENCES users(id) NOT NULL,
    box_number INTEGER NOT NULL,
    purchase_price_in_credits DECIMAL(10,2) NOT NULL,
    transaction_id UUID,
    blockchain_tx_hash VARCHAR(66),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(raffle_id, box_number)
);

-- User credits table
CREATE TABLE user_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    amount DECIMAL(10,2) NOT NULL,
    source credit_source NOT NULL,
    credit_type credit_type DEFAULT 'general',
    redeemable_on_item_id UUID REFERENCES items(id),
    expires_at TIMESTAMP WITH TIME ZONE,
    is_transferable BOOLEAN DEFAULT FALSE,
    is_used BOOLEAN DEFAULT FALSE,
    used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    seller_id UUID REFERENCES sellers(id),
    amount DECIMAL(10,2) NOT NULL,
    type transaction_type NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    payment_gateway_ref VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Free redeemable items table
CREATE TABLE free_redeemable_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID REFERENCES items(id) NOT NULL,
    required_credit_amount DECIMAL(10,2) NOT NULL,
    available_quantity INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_wallet_address ON users(internal_wallet_address);
CREATE INDEX idx_sellers_user_id ON sellers(user_id);
CREATE INDEX idx_items_seller_id ON items(seller_id);
CREATE INDEX idx_items_status ON items(status);
CREATE INDEX idx_raffles_item_id ON raffles(item_id);
CREATE INDEX idx_raffles_status ON raffles(status);
CREATE INDEX idx_box_purchases_raffle_id ON box_purchases(raffle_id);
CREATE INDEX idx_box_purchases_user_id ON box_purchases(user_id);
CREATE INDEX idx_user_credits_user_id ON user_credits(user_id);
CREATE INDEX idx_user_credits_expires_at ON user_credits(expires_at);
CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_seller_id ON transactions(seller_id);
CREATE INDEX idx_transactions_type ON transactions(type);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_sellers_updated_at BEFORE UPDATE ON sellers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_items_updated_at BEFORE UPDATE ON items FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_raffles_updated_at BEFORE UPDATE ON raffles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_transactions_updated_at BEFORE UPDATE ON transactions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();