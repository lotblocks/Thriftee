-- Add audit logging for security and compliance
CREATE TYPE audit_action AS ENUM (
    'create', 'update', 'delete', 'login', 'logout', 
    'password_change', 'email_change', 'role_change',
    'payment', 'refund', 'withdrawal', 'credit_issue',
    'raffle_create', 'raffle_complete', 'box_purchase',
    'admin_action', 'security_event'
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action audit_action NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    session_id UUID,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add indexes for audit logs
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_session_id ON audit_logs(session_id);

-- Add notification preferences table
CREATE TABLE user_notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    email_notifications BOOLEAN DEFAULT TRUE,
    sms_notifications BOOLEAN DEFAULT FALSE,
    push_notifications BOOLEAN DEFAULT TRUE,
    raffle_updates BOOLEAN DEFAULT TRUE,
    winner_announcements BOOLEAN DEFAULT TRUE,
    credit_expiry_alerts BOOLEAN DEFAULT TRUE,
    marketing_emails BOOLEAN DEFAULT FALSE,
    security_alerts BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Add trigger for notification preferences
CREATE TRIGGER update_user_notification_preferences_updated_at 
    BEFORE UPDATE ON user_notification_preferences 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add indexes for notification preferences
CREATE INDEX idx_user_notification_preferences_user_id ON user_notification_preferences(user_id);