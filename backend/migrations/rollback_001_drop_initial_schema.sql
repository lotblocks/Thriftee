-- Rollback migration for 001_initial_schema.sql
-- This script removes all tables and objects created in the initial schema

-- Drop triggers first
DROP TRIGGER IF EXISTS update_transactions_updated_at ON transactions;
DROP TRIGGER IF EXISTS update_raffles_updated_at ON raffles;
DROP TRIGGER IF EXISTS update_items_updated_at ON items;
DROP TRIGGER IF EXISTS update_sellers_updated_at ON sellers;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS idx_transactions_type;
DROP INDEX IF EXISTS idx_transactions_seller_id;
DROP INDEX IF EXISTS idx_transactions_user_id;
DROP INDEX IF EXISTS idx_user_credits_expires_at;
DROP INDEX IF EXISTS idx_user_credits_user_id;
DROP INDEX IF EXISTS idx_box_purchases_user_id;
DROP INDEX IF EXISTS idx_box_purchases_raffle_id;
DROP INDEX IF EXISTS idx_raffles_status;
DROP INDEX IF EXISTS idx_raffles_item_id;
DROP INDEX IF EXISTS idx_items_status;
DROP INDEX IF EXISTS idx_items_seller_id;
DROP INDEX IF EXISTS idx_sellers_user_id;
DROP INDEX IF EXISTS idx_users_wallet_address;
DROP INDEX IF EXISTS idx_users_username;
DROP INDEX IF EXISTS idx_users_email;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS free_redeemable_items;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS user_credits;
DROP TABLE IF EXISTS box_purchases;
DROP TABLE IF EXISTS raffles;
DROP TABLE IF EXISTS items;
DROP TABLE IF EXISTS sellers;
DROP TABLE IF EXISTS seller_subscriptions;
DROP TABLE IF EXISTS users;

-- Drop custom types
DROP TYPE IF EXISTS transaction_type;
DROP TYPE IF EXISTS credit_type;
DROP TYPE IF EXISTS credit_source;
DROP TYPE IF EXISTS raffle_status;
DROP TYPE IF EXISTS item_status;
DROP TYPE IF EXISTS user_role;