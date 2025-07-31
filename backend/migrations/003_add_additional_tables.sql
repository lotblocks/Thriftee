-- Add additional tables for complete platform functionality
-- This migration adds tables for sessions, notifications, audit logs, and other supporting features

-- User sessions table for JWT refresh token management
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    refresh_token_hash VARCHAR(255) NOT NULL,
    device_info JSONB,
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Email verification tokens
CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    is_used BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Password reset tokens
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    is_used BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    type VARCHAR(50) NOT NULL,
    data JSONB,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Audit logs for security and compliance
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- System settings table
CREATE TABLE system_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(100) UNIQUE NOT NULL,
    value JSONB NOT NULL,
    description TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Raffle participants view for easier querying
CREATE VIEW raffle_participants AS
SELECT 
    r.id as raffle_id,
    r.item_id,
    r.status as raffle_status,
    bp.user_id,
    u.username,
    u.email,
    COUNT(bp.id) as boxes_purchased,
    SUM(bp.purchase_price_in_credits) as total_spent,
    CASE WHEN r.winner_user_ids @> ARRAY[bp.user_id] THEN true ELSE false END as is_winner
FROM raffles r
JOIN box_purchases bp ON r.id = bp.raffle_id
JOIN users u ON bp.user_id = u.id
GROUP BY r.id, r.item_id, r.status, bp.user_id, u.username, u.email, r.winner_user_ids;

-- User credit summary view
CREATE VIEW user_credit_summary AS
SELECT 
    u.id as user_id,
    u.username,
    u.credit_balance,
    COALESCE(SUM(CASE WHEN uc.is_used = false AND uc.credit_type = 'general' THEN uc.amount ELSE 0 END), 0) as available_general_credits,
    COALESCE(SUM(CASE WHEN uc.is_used = false AND uc.credit_type = 'item_specific' THEN uc.amount ELSE 0 END), 0) as available_item_specific_credits,
    COALESCE(SUM(CASE WHEN uc.is_used = false AND uc.expires_at < NOW() + INTERVAL '7 days' THEN uc.amount ELSE 0 END), 0) as expiring_soon_credits,
    COUNT(CASE WHEN uc.is_used = false AND uc.expires_at < NOW() THEN 1 END) as expired_credits_count
FROM users u
LEFT JOIN user_credits uc ON u.id = uc.user_id
GROUP BY u.id, u.username, u.credit_balance;

-- Seller performance view
CREATE VIEW seller_performance AS
SELECT 
    s.id as seller_id,
    s.user_id,
    u.username as seller_username,
    s.company_name,
    COUNT(DISTINCT i.id) as total_items_listed,
    COUNT(DISTINCT CASE WHEN r.status = 'completed' THEN r.id END) as completed_raffles,
    COUNT(DISTINCT CASE WHEN r.status = 'open' THEN r.id END) as active_raffles,
    COALESCE(SUM(CASE WHEN r.status = 'completed' THEN r.total_boxes * r.box_price END), 0) as total_revenue,
    COALESCE(AVG(CASE WHEN r.status = 'completed' THEN r.boxes_sold::DECIMAL / r.total_boxes END), 0) as avg_completion_rate
FROM sellers s
JOIN users u ON s.user_id = u.id
LEFT JOIN items i ON s.id = i.seller_id
LEFT JOIN raffles r ON i.id = r.item_id
GROUP BY s.id, s.user_id, u.username, s.company_name;

-- Create indexes for new tables
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_refresh_token ON user_sessions(refresh_token_hash);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_email_verification_tokens_user_id ON email_verification_tokens(user_id);
CREATE INDEX idx_email_verification_tokens_token ON email_verification_tokens(token);
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token ON password_reset_tokens(token);
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_user_read ON notifications(user_id, is_read);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_system_settings_key ON system_settings(key);

-- Add triggers for updated_at columns
CREATE TRIGGER update_user_sessions_updated_at BEFORE UPDATE ON user_sessions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_system_settings_updated_at BEFORE UPDATE ON system_settings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default system settings
INSERT INTO system_settings (key, value, description, is_public) VALUES
('platform_name', '"Raffle Shopping Platform"', 'The name of the platform', true),
('max_boxes_per_raffle', '10000', 'Maximum number of boxes allowed per raffle', false),
('min_box_price', '1.00', 'Minimum price per box in credits', false),
('credit_expiration_days', '365', 'Default number of days before credits expire', false),
('free_item_credit_threshold', '10.00', 'Minimum credit amount required for free item redemption', false),
('platform_fee_percentage', '5.0', 'Platform fee percentage on completed raffles', false);

-- Insert default seller subscription tiers
INSERT INTO seller_subscriptions (name, monthly_fee, listing_fee_percentage, transaction_fee_percentage, max_listings, features) VALUES
('Basic', 29.99, 2.0, 3.0, 10, '{"analytics": false, "priority_support": false, "custom_branding": false}'),
('Professional', 79.99, 1.5, 2.5, 50, '{"analytics": true, "priority_support": false, "custom_branding": false}'),
('Enterprise', 199.99, 1.0, 2.0, null, '{"analytics": true, "priority_support": true, "custom_branding": true}');