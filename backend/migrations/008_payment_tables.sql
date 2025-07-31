-- Migration for payment processing tables

-- Payment status enum
CREATE TYPE payment_status AS ENUM (
    'pending',
    'processing', 
    'succeeded',
    'failed',
    'cancelled',
    'refunded'
);

-- Subscription status enum (matching Stripe's statuses)
CREATE TYPE subscription_status AS ENUM (
    'incomplete',
    'incomplete_expired',
    'trialing',
    'active',
    'past_due',
    'canceled',
    'unpaid'
);

-- User Stripe customers table
CREATE TABLE IF NOT EXISTS user_stripe_customers (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    stripe_customer_id VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Payments table
CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_payment_intent_id VARCHAR(255) NOT NULL UNIQUE,
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    status payment_status NOT NULL DEFAULT 'pending',
    description TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,
    failure_reason TEXT,
    
    -- Constraints
    CONSTRAINT positive_amount CHECK (amount > 0),
    CONSTRAINT valid_currency CHECK (currency IN ('USD', 'EUR', 'GBP'))
);

-- Subscriptions table
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_subscription_id VARCHAR(255) NOT NULL UNIQUE,
    stripe_customer_id VARCHAR(255) NOT NULL,
    status subscription_status NOT NULL DEFAULT 'incomplete',
    current_period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    current_period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    trial_end TIMESTAMP WITH TIME ZONE,
    cancelled_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key to customer table
    FOREIGN KEY (stripe_customer_id) REFERENCES user_stripe_customers(stripe_customer_id)
);

-- Payment methods table (for saved payment methods)
CREATE TABLE IF NOT EXISTS user_payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_payment_method_id VARCHAR(255) NOT NULL UNIQUE,
    type VARCHAR(50) NOT NULL, -- card, bank_account, etc.
    card_brand VARCHAR(50), -- visa, mastercard, etc.
    card_last4 VARCHAR(4),
    card_exp_month INTEGER,
    card_exp_year INTEGER,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT valid_card_exp_month CHECK (card_exp_month BETWEEN 1 AND 12),
    CONSTRAINT valid_card_exp_year CHECK (card_exp_year > EXTRACT(YEAR FROM CURRENT_DATE)),
    CONSTRAINT valid_last4 CHECK (card_last4 ~ '^[0-9]{4}$')
);

-- Refunds table
CREATE TABLE IF NOT EXISTS refunds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL REFERENCES payments(id) ON DELETE CASCADE,
    stripe_refund_id VARCHAR(255) NOT NULL UNIQUE,
    amount DECIMAL(10,2) NOT NULL,
    reason VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT positive_refund_amount CHECK (amount > 0)
);

-- Webhook events table (for idempotency and debugging)
CREATE TABLE IF NOT EXISTS webhook_events (
    id BIGSERIAL PRIMARY KEY,
    stripe_event_id VARCHAR(255) NOT NULL UNIQUE,
    event_type VARCHAR(100) NOT NULL,
    processed BOOLEAN DEFAULT FALSE,
    payload JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_payments_user_id ON payments(user_id);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
CREATE INDEX IF NOT EXISTS idx_payments_created_at ON payments(created_at);
CREATE INDEX IF NOT EXISTS idx_payments_stripe_payment_intent_id ON payments(stripe_payment_intent_id);

CREATE INDEX IF NOT EXISTS idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_subscriptions_stripe_subscription_id ON subscriptions(stripe_subscription_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_current_period_end ON subscriptions(current_period_end);

CREATE INDEX IF NOT EXISTS idx_user_payment_methods_user_id ON user_payment_methods(user_id);
CREATE INDEX IF NOT EXISTS idx_user_payment_methods_is_default ON user_payment_methods(user_id, is_default) WHERE is_default = TRUE;

CREATE INDEX IF NOT EXISTS idx_refunds_payment_id ON refunds(payment_id);
CREATE INDEX IF NOT EXISTS idx_refunds_stripe_refund_id ON refunds(stripe_refund_id);

CREATE INDEX IF NOT EXISTS idx_webhook_events_stripe_event_id ON webhook_events(stripe_event_id);
CREATE INDEX IF NOT EXISTS idx_webhook_events_processed ON webhook_events(processed) WHERE processed = FALSE;
CREATE INDEX IF NOT EXISTS idx_webhook_events_event_type ON webhook_events(event_type);

CREATE INDEX IF NOT EXISTS idx_user_stripe_customers_user_id ON user_stripe_customers(user_id);
CREATE INDEX IF NOT EXISTS idx_user_stripe_customers_stripe_customer_id ON user_stripe_customers(stripe_customer_id);

-- Add comments for documentation
COMMENT ON TABLE user_stripe_customers IS 'Maps users to their Stripe customer IDs';
COMMENT ON TABLE payments IS 'Records of all payment transactions through Stripe';
COMMENT ON TABLE subscriptions IS 'User subscriptions for seller plans';
COMMENT ON TABLE user_payment_methods IS 'Saved payment methods for users';
COMMENT ON TABLE refunds IS 'Refund transactions';
COMMENT ON TABLE webhook_events IS 'Stripe webhook events for idempotency and debugging';

COMMENT ON COLUMN payments.amount IS 'Payment amount in dollars (not cents)';
COMMENT ON COLUMN payments.metadata IS 'Additional payment metadata from Stripe';
COMMENT ON COLUMN subscriptions.current_period_start IS 'Start of current billing period';
COMMENT ON COLUMN subscriptions.current_period_end IS 'End of current billing period';
COMMENT ON COLUMN user_payment_methods.is_default IS 'Whether this is the users default payment method';

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers to automatically update updated_at
CREATE TRIGGER update_user_stripe_customers_updated_at BEFORE UPDATE ON user_stripe_customers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_payments_updated_at BEFORE UPDATE ON payments FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_subscriptions_updated_at BEFORE UPDATE ON subscriptions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_payment_methods_updated_at BEFORE UPDATE ON user_payment_methods FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_refunds_updated_at BEFORE UPDATE ON refunds FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create a view for payment summary
CREATE OR REPLACE VIEW user_payment_summary AS
SELECT 
    u.id as user_id,
    u.email,
    COALESCE(SUM(CASE WHEN p.status = 'succeeded' THEN p.amount ELSE 0 END), 0) as total_paid,
    COUNT(CASE WHEN p.status = 'succeeded' THEN 1 END) as successful_payments,
    COUNT(CASE WHEN p.status = 'failed' THEN 1 END) as failed_payments,
    COUNT(CASE WHEN s.status = 'active' THEN 1 END) as active_subscriptions,
    MAX(p.completed_at) as last_payment_date,
    MIN(p.created_at) as first_payment_date
FROM users u
LEFT JOIN payments p ON u.id = p.user_id
LEFT JOIN subscriptions s ON u.id = s.user_id
GROUP BY u.id, u.email;

COMMENT ON VIEW user_payment_summary IS 'Summary view of user payment activity and subscriptions';

-- Create a function to get user payment statistics
CREATE OR REPLACE FUNCTION get_user_payment_stats(p_user_id UUID)
RETURNS TABLE (
    total_amount DECIMAL(10,2),
    successful_payments BIGINT,
    failed_payments BIGINT,
    pending_payments BIGINT,
    last_payment_date TIMESTAMP WITH TIME ZONE,
    has_active_subscription BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COALESCE(SUM(CASE WHEN p.status = 'succeeded' THEN p.amount ELSE 0 END), 0) as total_amount,
        COUNT(CASE WHEN p.status = 'succeeded' THEN 1 END) as successful_payments,
        COUNT(CASE WHEN p.status = 'failed' THEN 1 END) as failed_payments,
        COUNT(CASE WHEN p.status = 'pending' THEN 1 END) as pending_payments,
        MAX(p.completed_at) as last_payment_date,
        EXISTS(SELECT 1 FROM subscriptions s WHERE s.user_id = p_user_id AND s.status = 'active') as has_active_subscription
    FROM payments p
    WHERE p.user_id = p_user_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_user_payment_stats IS 'Function to get comprehensive payment statistics for a user';

-- Create a function to check if user has active subscription
CREATE OR REPLACE FUNCTION user_has_active_subscription(p_user_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    has_subscription BOOLEAN;
BEGIN
    SELECT EXISTS(
        SELECT 1 FROM subscriptions 
        WHERE user_id = p_user_id 
        AND status IN ('active', 'trialing')
        AND current_period_end > NOW()
    ) INTO has_subscription;
    
    RETURN has_subscription;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION user_has_active_subscription IS 'Function to check if user has an active subscription';

-- Create a materialized view for payment analytics
CREATE MATERIALIZED VIEW IF NOT EXISTS payment_analytics AS
SELECT 
    DATE_TRUNC('day', created_at) as payment_date,
    COUNT(*) as total_payments,
    COUNT(CASE WHEN status = 'succeeded' THEN 1 END) as successful_payments,
    COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_payments,
    COALESCE(SUM(CASE WHEN status = 'succeeded' THEN amount ELSE 0 END), 0) as total_revenue,
    COALESCE(AVG(CASE WHEN status = 'succeeded' THEN amount END), 0) as avg_payment_amount,
    COUNT(DISTINCT user_id) as unique_users
FROM payments
WHERE created_at >= CURRENT_DATE - INTERVAL '90 days'
GROUP BY DATE_TRUNC('day', created_at)
ORDER BY payment_date DESC;

-- Create unique index for materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_payment_analytics_date ON payment_analytics(payment_date);

COMMENT ON MATERIALIZED VIEW payment_analytics IS 'Daily payment analytics for the last 90 days';

-- Create a function to refresh payment analytics
CREATE OR REPLACE FUNCTION refresh_payment_analytics()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY payment_analytics;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION refresh_payment_analytics IS 'Function to refresh the payment analytics materialized view';