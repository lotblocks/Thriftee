-- Add additional constraints and relationships
-- This migration adds proper constraints, foreign key relationships, and validation rules

-- Add check constraints for data validation
ALTER TABLE users 
ADD CONSTRAINT check_credit_balance_non_negative CHECK (credit_balance >= 0),
ADD CONSTRAINT check_email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
ADD CONSTRAINT check_username_length CHECK (char_length(username) >= 3);

-- Add constraints for seller subscriptions
ALTER TABLE seller_subscriptions
ADD CONSTRAINT check_monthly_fee_non_negative CHECK (monthly_fee >= 0),
ADD CONSTRAINT check_listing_fee_percentage CHECK (listing_fee_percentage >= 0 AND listing_fee_percentage <= 100),
ADD CONSTRAINT check_transaction_fee_percentage CHECK (transaction_fee_percentage >= 0 AND transaction_fee_percentage <= 100);

-- Add constraints for items
ALTER TABLE items
ADD CONSTRAINT check_retail_price_positive CHECK (retail_price > 0),
ADD CONSTRAINT check_cost_of_goods_non_negative CHECK (cost_of_goods >= 0),
ADD CONSTRAINT check_stock_quantity_non_negative CHECK (stock_quantity >= 0),
ADD CONSTRAINT check_images_not_empty CHECK (array_length(images, 1) > 0);

-- Add constraints for raffles
ALTER TABLE raffles
ADD CONSTRAINT check_total_boxes_positive CHECK (total_boxes > 0),
ADD CONSTRAINT check_box_price_positive CHECK (box_price > 0),
ADD CONSTRAINT check_boxes_sold_valid CHECK (boxes_sold >= 0 AND boxes_sold <= total_boxes),
ADD CONSTRAINT check_total_winners_valid CHECK (total_winners > 0 AND total_winners <= total_boxes),
ADD CONSTRAINT check_grid_dimensions CHECK (grid_rows > 0 AND grid_cols > 0 AND grid_rows * grid_cols = total_boxes);

-- Add constraints for box purchases
ALTER TABLE box_purchases
ADD CONSTRAINT check_box_number_positive CHECK (box_number > 0),
ADD CONSTRAINT check_purchase_price_positive CHECK (purchase_price_in_credits > 0);

-- Add constraints for user credits
ALTER TABLE user_credits
ADD CONSTRAINT check_credit_amount_positive CHECK (amount > 0);

-- Add constraints for transactions
ALTER TABLE transactions
ADD CONSTRAINT check_transaction_amount_not_zero CHECK (amount != 0);

-- Add constraints for free redeemable items
ALTER TABLE free_redeemable_items
ADD CONSTRAINT check_required_credit_amount_positive CHECK (required_credit_amount > 0),
ADD CONSTRAINT check_available_quantity_non_negative CHECK (available_quantity >= 0);

-- Create additional indexes for complex queries
CREATE INDEX idx_raffles_status_created_at ON raffles(status, created_at);
CREATE INDEX idx_user_credits_user_expires ON user_credits(user_id, expires_at) WHERE NOT is_used;
CREATE INDEX idx_box_purchases_raffle_user ON box_purchases(raffle_id, user_id);
CREATE INDEX idx_transactions_status_created ON transactions(status, created_at);
CREATE INDEX idx_items_seller_status ON items(seller_id, status);

-- Create composite indexes for performance
CREATE INDEX idx_raffles_item_status ON raffles(item_id, status);
CREATE INDEX idx_sellers_subscription_verified ON sellers(current_subscription_id, is_verified);

-- Add partial indexes for active records
CREATE INDEX idx_users_active_email ON users(email) WHERE is_active = true;
CREATE INDEX idx_items_available ON items(id) WHERE status = 'available';
CREATE INDEX idx_raffles_open ON raffles(id) WHERE status = 'open';