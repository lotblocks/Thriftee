-- Migration for credit transactions audit table

-- Credit transactions table for audit trail
CREATE TABLE IF NOT EXISTS credit_transactions (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount DECIMAL(10,2) NOT NULL,
    transaction_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    credit_id UUID REFERENCES user_credits(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT positive_amount CHECK (amount > 0)
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_credit_transactions_user_id ON credit_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_credit_transactions_created_at ON credit_transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_credit_transactions_transaction_type ON credit_transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_credit_transactions_credit_id ON credit_transactions(credit_id) WHERE credit_id IS NOT NULL;

-- Add comments for documentation
COMMENT ON TABLE credit_transactions IS 'Audit trail for all credit-related transactions';
COMMENT ON COLUMN credit_transactions.transaction_type IS 'Type of transaction: credit_issued, credit_redeemed, etc.';
COMMENT ON COLUMN credit_transactions.description IS 'Human-readable description of the transaction';
COMMENT ON COLUMN credit_transactions.credit_id IS 'Reference to the specific credit record if applicable';

-- Create a view for credit transaction summary
CREATE OR REPLACE VIEW user_credit_summary AS
SELECT 
    u.id as user_id,
    u.email,
    COALESCE(SUM(CASE WHEN uc.is_used = false AND (uc.expires_at IS NULL OR uc.expires_at > NOW()) THEN uc.amount ELSE 0 END), 0) as available_credits,
    COALESCE(SUM(CASE WHEN uc.is_used = true THEN uc.amount ELSE 0 END), 0) as used_credits,
    COALESCE(SUM(CASE WHEN uc.expires_at < NOW() AND uc.is_used = false THEN uc.amount ELSE 0 END), 0) as expired_credits,
    COALESCE(SUM(CASE WHEN uc.is_used = false AND uc.expires_at IS NOT NULL AND uc.expires_at <= NOW() + INTERVAL '30 days' AND uc.expires_at > NOW() THEN uc.amount ELSE 0 END), 0) as expiring_soon_credits,
    COUNT(CASE WHEN uc.is_used = false AND (uc.expires_at IS NULL OR uc.expires_at > NOW()) THEN 1 END) as active_credit_count,
    MAX(uc.created_at) as last_credit_received,
    MAX(uc.used_at) as last_credit_used
FROM users u
LEFT JOIN user_credits uc ON u.id = uc.user_id
GROUP BY u.id, u.email;

COMMENT ON VIEW user_credit_summary IS 'Summary view of user credit balances and activity';

-- Create a function to get user credit balance
CREATE OR REPLACE FUNCTION get_user_credit_balance(p_user_id UUID, p_credit_type TEXT DEFAULT NULL, p_item_id UUID DEFAULT NULL)
RETURNS DECIMAL(10,2) AS $$
DECLARE
    balance DECIMAL(10,2);
BEGIN
    SELECT COALESCE(SUM(amount), 0) INTO balance
    FROM user_credits 
    WHERE user_id = p_user_id 
    AND is_used = false 
    AND (expires_at IS NULL OR expires_at > NOW())
    AND (p_credit_type IS NULL OR credit_type::TEXT = p_credit_type)
    AND (p_item_id IS NULL OR redeemable_on_item_id IS NULL OR redeemable_on_item_id = p_item_id);
    
    RETURN balance;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_user_credit_balance IS 'Function to calculate available credit balance for a user with optional filters';

-- Create a function to check if user has sufficient credits
CREATE OR REPLACE FUNCTION check_sufficient_credits(p_user_id UUID, p_required_amount DECIMAL(10,2), p_credit_type TEXT DEFAULT NULL, p_item_id UUID DEFAULT NULL)
RETURNS BOOLEAN AS $$
DECLARE
    available_balance DECIMAL(10,2);
BEGIN
    SELECT get_user_credit_balance(p_user_id, p_credit_type, p_item_id) INTO available_balance;
    RETURN available_balance >= p_required_amount;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION check_sufficient_credits IS 'Function to check if user has sufficient credits for a purchase';

-- Create indexes on user_credits for better performance
CREATE INDEX IF NOT EXISTS idx_user_credits_user_id_available ON user_credits(user_id) WHERE is_used = false AND (expires_at IS NULL OR expires_at > NOW());
CREATE INDEX IF NOT EXISTS idx_user_credits_expiring ON user_credits(expires_at) WHERE is_used = false AND expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_credits_credit_type ON user_credits(credit_type);
CREATE INDEX IF NOT EXISTS idx_user_credits_source ON user_credits(source);

-- Create a trigger to automatically log credit usage
CREATE OR REPLACE FUNCTION log_credit_usage()
RETURNS TRIGGER AS $$
BEGIN
    -- Log when a credit is marked as used
    IF OLD.is_used = false AND NEW.is_used = true THEN
        INSERT INTO credit_transactions (user_id, amount, transaction_type, description, credit_id)
        VALUES (NEW.user_id, NEW.amount, 'credit_used', 'Credit marked as used', NEW.id);
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create the trigger
DROP TRIGGER IF EXISTS trigger_log_credit_usage ON user_credits;
CREATE TRIGGER trigger_log_credit_usage
    AFTER UPDATE ON user_credits
    FOR EACH ROW
    EXECUTE FUNCTION log_credit_usage();

COMMENT ON FUNCTION log_credit_usage IS 'Trigger function to automatically log credit usage';

-- Create a materialized view for credit statistics (refreshed periodically)
CREATE MATERIALIZED VIEW IF NOT EXISTS credit_statistics AS
SELECT 
    COUNT(*) as total_credits,
    COUNT(CASE WHEN is_used = false AND (expires_at IS NULL OR expires_at > NOW()) THEN 1 END) as active_credits,
    COUNT(CASE WHEN is_used = true THEN 1 END) as used_credits,
    COUNT(CASE WHEN expires_at < NOW() AND is_used = false THEN 1 END) as expired_credits,
    COALESCE(SUM(amount), 0) as total_amount_issued,
    COALESCE(SUM(CASE WHEN is_used = false AND (expires_at IS NULL OR expires_at > NOW()) THEN amount ELSE 0 END), 0) as total_active_amount,
    COALESCE(SUM(CASE WHEN is_used = true THEN amount ELSE 0 END), 0) as total_used_amount,
    COALESCE(SUM(CASE WHEN expires_at < NOW() AND is_used = false THEN amount ELSE 0 END), 0) as total_expired_amount,
    COUNT(DISTINCT user_id) as users_with_credits,
    COUNT(DISTINCT CASE WHEN is_used = false AND (expires_at IS NULL OR expires_at > NOW()) THEN user_id END) as users_with_active_credits,
    NOW() as last_updated
FROM user_credits;

-- Create unique index for materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_credit_statistics_unique ON credit_statistics(last_updated);

COMMENT ON MATERIALIZED VIEW credit_statistics IS 'Materialized view with credit system statistics, refreshed periodically';

-- Create a function to refresh credit statistics
CREATE OR REPLACE FUNCTION refresh_credit_statistics()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY credit_statistics;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION refresh_credit_statistics IS 'Function to refresh the credit statistics materialized view';