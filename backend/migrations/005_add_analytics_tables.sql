-- Add analytics and reporting tables

-- User activity tracking
CREATE TABLE user_activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    session_id UUID,
    activity_type VARCHAR(50) NOT NULL,
    page_url TEXT,
    referrer TEXT,
    ip_address INET,
    user_agent TEXT,
    device_type VARCHAR(20),
    browser VARCHAR(50),
    os VARCHAR(50),
    country VARCHAR(2),
    city VARCHAR(100),
    duration_seconds INTEGER,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add indexes for analytics
CREATE INDEX idx_user_activity_logs_user_id ON user_activity_logs(user_id);
CREATE INDEX idx_user_activity_logs_activity_type ON user_activity_logs(activity_type);
CREATE INDEX idx_user_activity_logs_created_at ON user_activity_logs(created_at);
CREATE INDEX idx_user_activity_logs_session_id ON user_activity_logs(session_id);

-- Raffle performance metrics
CREATE TABLE raffle_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    raffle_id UUID REFERENCES raffles(id) ON DELETE CASCADE NOT NULL,
    views_count INTEGER DEFAULT 0,
    unique_viewers INTEGER DEFAULT 0,
    conversion_rate DECIMAL(5,4),
    average_boxes_per_user DECIMAL(8,2),
    time_to_completion_minutes INTEGER,
    peak_concurrent_users INTEGER DEFAULT 0,
    total_revenue DECIMAL(12,2),
    platform_fee DECIMAL(12,2),
    seller_payout DECIMAL(12,2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(raffle_id)
);

-- Add trigger for raffle metrics
CREATE TRIGGER update_raffle_metrics_updated_at 
    BEFORE UPDATE ON raffle_metrics 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Daily platform statistics
CREATE TABLE daily_platform_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    total_users INTEGER DEFAULT 0,
    new_users INTEGER DEFAULT 0,
    active_users INTEGER DEFAULT 0,
    total_sellers INTEGER DEFAULT 0,
    new_sellers INTEGER DEFAULT 0,
    active_sellers INTEGER DEFAULT 0,
    total_raffles INTEGER DEFAULT 0,
    completed_raffles INTEGER DEFAULT 0,
    total_revenue DECIMAL(12,2) DEFAULT 0,
    total_credits_issued DECIMAL(12,2) DEFAULT 0,
    total_credits_redeemed DECIMAL(12,2) DEFAULT 0,
    average_raffle_completion_time INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(date)
);

-- Add seller performance metrics
CREATE TABLE seller_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id UUID REFERENCES sellers(id) ON DELETE CASCADE NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    total_items_listed INTEGER DEFAULT 0,
    total_raffles_completed INTEGER DEFAULT 0,
    total_revenue DECIMAL(12,2) DEFAULT 0,
    total_fees_paid DECIMAL(12,2) DEFAULT 0,
    average_completion_time INTEGER,
    conversion_rate DECIMAL(5,4),
    customer_satisfaction_score DECIMAL(3,2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(seller_id, period_start, period_end)
);

-- Add trigger for seller metrics
CREATE TRIGGER update_seller_metrics_updated_at 
    BEFORE UPDATE ON seller_metrics 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add indexes for analytics tables
CREATE INDEX idx_daily_platform_stats_date ON daily_platform_stats(date);
CREATE INDEX idx_seller_metrics_seller_period ON seller_metrics(seller_id, period_start, period_end);
CREATE INDEX idx_raffle_metrics_raffle_id ON raffle_metrics(raffle_id);