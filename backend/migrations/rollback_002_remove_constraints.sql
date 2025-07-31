-- Rollback migration for 002_add_constraints_and_relationships.sql
-- This script removes all constraints and indexes added in migration 002

-- Drop partial indexes
DROP INDEX IF EXISTS idx_raffles_open;
DROP INDEX IF EXISTS idx_items_available;
DROP INDEX IF EXISTS idx_users_active_email;

-- Drop composite indexes
DROP INDEX IF EXISTS idx_sellers_subscription_verified;
DROP INDEX IF EXISTS idx_raffles_item_status;

-- Drop additional indexes
DROP INDEX IF EXISTS idx_items_seller_status;
DROP INDEX IF EXISTS idx_transactions_status_created;
DROP INDEX IF EXISTS idx_box_purchases_raffle_user;
DROP INDEX IF EXISTS idx_user_credits_user_expires;
DROP INDEX IF EXISTS idx_raffles_status_created_at;

-- Drop check constraints
ALTER TABLE free_redeemable_items
DROP CONSTRAINT IF EXISTS check_available_quantity_non_negative,
DROP CONSTRAINT IF EXISTS check_required_credit_amount_positive;

ALTER TABLE transactions
DROP CONSTRAINT IF EXISTS check_transaction_amount_not_zero;

ALTER TABLE user_credits
DROP CONSTRAINT IF EXISTS check_credit_amount_positive;

ALTER TABLE box_purchases
DROP CONSTRAINT IF EXISTS check_purchase_price_positive,
DROP CONSTRAINT IF EXISTS check_box_number_positive;

ALTER TABLE raffles
DROP CONSTRAINT IF EXISTS check_grid_dimensions,
DROP CONSTRAINT IF EXISTS check_total_winners_valid,
DROP CONSTRAINT IF EXISTS check_boxes_sold_valid,
DROP CONSTRAINT IF EXISTS check_box_price_positive,
DROP CONSTRAINT IF EXISTS check_total_boxes_positive;

ALTER TABLE items
DROP CONSTRAINT IF EXISTS check_images_not_empty,
DROP CONSTRAINT IF EXISTS check_stock_quantity_non_negative,
DROP CONSTRAINT IF EXISTS check_cost_of_goods_non_negative,
DROP CONSTRAINT IF EXISTS check_retail_price_positive;

ALTER TABLE seller_subscriptions
DROP CONSTRAINT IF EXISTS check_transaction_fee_percentage,
DROP CONSTRAINT IF EXISTS check_listing_fee_percentage,
DROP CONSTRAINT IF EXISTS check_monthly_fee_non_negative;

ALTER TABLE users
DROP CONSTRAINT IF EXISTS check_username_length,
DROP CONSTRAINT IF EXISTS check_email_format,
DROP CONSTRAINT IF EXISTS check_credit_balance_non_negative;