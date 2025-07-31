-- Migration: Create audit logs table
-- Description: Comprehensive audit logging for security and compliance

-- Create audit logs table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- User and session context
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    session_id TEXT,
    
    -- Request context
    ip_address INET,
    user_agent TEXT,
    request_id TEXT,
    endpoint TEXT,
    method TEXT,
    
    -- Event details
    description TEXT NOT NULL,
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,
    
    -- Structured data
    metadata JSONB DEFAULT '{}',
    additional_data JSONB DEFAULT '{}',
    
    -- Audit trail
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity);
CREATE INDEX idx_audit_logs_ip_address ON audit_logs(ip_address) WHERE ip_address IS NOT NULL;
CREATE INDEX idx_audit_logs_success ON audit_logs(success) WHERE success = false;
CREATE INDEX idx_audit_logs_endpoint ON audit_logs(endpoint) WHERE endpoint IS NOT NULL;

-- Create composite indexes for common queries
CREATE INDEX idx_audit_logs_user_timestamp ON audit_logs(user_id, timestamp DESC) WHERE user_id IS NOT NULL;
CREATE INDEX idx_audit_logs_type_timestamp ON audit_logs(event_type, timestamp DESC);
CREATE INDEX idx_audit_logs_severity_timestamp ON audit_logs(severity, timestamp DESC);

-- Create GIN index for JSONB fields
CREATE INDEX idx_audit_logs_metadata ON audit_logs USING GIN(metadata);
CREATE INDEX idx_audit_logs_additional_data ON audit_logs USING GIN(additional_data);

-- Create security events summary view
CREATE VIEW security_events_summary AS
SELECT 
    event_type,
    severity,
    COUNT(*) as event_count,
    COUNT(*) FILTER (WHERE success = false) as failed_count,
    COUNT(DISTINCT user_id) as unique_users,
    COUNT(DISTINCT ip_address) as unique_ips,
    MIN(timestamp) as first_occurrence,
    MAX(timestamp) as last_occurrence
FROM audit_logs
WHERE timestamp >= NOW() - INTERVAL '24 hours'
GROUP BY event_type, severity
ORDER BY event_count DESC;

-- Create failed authentication attempts view
CREATE VIEW failed_auth_attempts AS
SELECT 
    ip_address,
    user_agent,
    COUNT(*) as attempt_count,
    MAX(timestamp) as last_attempt,
    array_agg(DISTINCT user_id) FILTER (WHERE user_id IS NOT NULL) as attempted_user_ids
FROM audit_logs
WHERE event_type IN ('UserLogin', 'BruteForceAttempt')
    AND success = false
    AND timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY ip_address, user_agent
HAVING COUNT(*) >= 3
ORDER BY attempt_count DESC, last_attempt DESC;

-- Create suspicious activity view
CREATE VIEW suspicious_activities AS
SELECT 
    id,
    event_type,
    severity,
    timestamp,
    user_id,
    ip_address,
    description,
    metadata
FROM audit_logs
WHERE severity IN ('High', 'Critical')
    OR (event_type IN ('UnauthorizedAccess', 'SuspiciousActivity', 'AccessDenied') AND timestamp >= NOW() - INTERVAL '24 hours')
ORDER BY timestamp DESC;

-- Create user activity summary view
CREATE VIEW user_activity_summary AS
SELECT 
    u.id as user_id,
    u.email,
    COUNT(al.*) as total_events,
    COUNT(*) FILTER (WHERE al.event_type = 'UserLogin' AND al.success = true) as successful_logins,
    COUNT(*) FILTER (WHERE al.event_type = 'UserLogin' AND al.success = false) as failed_logins,
    COUNT(*) FILTER (WHERE al.event_type LIKE '%Purchase%') as purchase_events,
    COUNT(*) FILTER (WHERE al.severity = 'Critical') as critical_events,
    MAX(al.timestamp) as last_activity,
    COUNT(DISTINCT al.ip_address) as unique_ips
FROM users u
LEFT JOIN audit_logs al ON u.id = al.user_id
WHERE al.timestamp >= NOW() - INTERVAL '30 days' OR al.timestamp IS NULL
GROUP BY u.id, u.email
ORDER BY total_events DESC;

-- Create function to automatically clean old audit logs
CREATE OR REPLACE FUNCTION cleanup_old_audit_logs()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    -- Keep audit logs for 7 years for compliance
    -- But archive logs older than 1 year to separate table
    DELETE FROM audit_logs 
    WHERE timestamp < NOW() - INTERVAL '7 years';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Log the cleanup operation
    INSERT INTO audit_logs (event_type, severity, description, success, metadata)
    VALUES (
        'DatabaseMaintenance',
        'Low',
        'Automated cleanup of old audit logs',
        true,
        jsonb_build_object('deleted_count', deleted_count)
    );
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Create function to detect brute force attempts
CREATE OR REPLACE FUNCTION detect_brute_force_attempts()
RETURNS TABLE(ip_address INET, attempt_count BIGINT, last_attempt TIMESTAMPTZ) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        al.ip_address,
        COUNT(*) as attempt_count,
        MAX(al.timestamp) as last_attempt
    FROM audit_logs al
    WHERE al.event_type = 'UserLogin'
        AND al.success = false
        AND al.timestamp >= NOW() - INTERVAL '15 minutes'
        AND al.ip_address IS NOT NULL
    GROUP BY al.ip_address
    HAVING COUNT(*) >= 5
    ORDER BY attempt_count DESC;
END;
$$ LANGUAGE plpgsql;

-- Create function to get user risk score
CREATE OR REPLACE FUNCTION calculate_user_risk_score(target_user_id UUID)
RETURNS INTEGER AS $$
DECLARE
    risk_score INTEGER := 0;
    failed_logins INTEGER;
    suspicious_events INTEGER;
    unique_ips INTEGER;
    recent_activity TIMESTAMPTZ;
BEGIN
    -- Count failed login attempts in last 24 hours
    SELECT COUNT(*) INTO failed_logins
    FROM audit_logs
    WHERE user_id = target_user_id
        AND event_type = 'UserLogin'
        AND success = false
        AND timestamp >= NOW() - INTERVAL '24 hours';
    
    -- Count suspicious events in last 7 days
    SELECT COUNT(*) INTO suspicious_events
    FROM audit_logs
    WHERE user_id = target_user_id
        AND severity IN ('High', 'Critical')
        AND timestamp >= NOW() - INTERVAL '7 days';
    
    -- Count unique IP addresses in last 30 days
    SELECT COUNT(DISTINCT ip_address) INTO unique_ips
    FROM audit_logs
    WHERE user_id = target_user_id
        AND ip_address IS NOT NULL
        AND timestamp >= NOW() - INTERVAL '30 days';
    
    -- Get last activity
    SELECT MAX(timestamp) INTO recent_activity
    FROM audit_logs
    WHERE user_id = target_user_id;
    
    -- Calculate risk score
    risk_score := risk_score + (failed_logins * 10);
    risk_score := risk_score + (suspicious_events * 25);
    risk_score := risk_score + CASE 
        WHEN unique_ips > 10 THEN 20
        WHEN unique_ips > 5 THEN 10
        ELSE 0
    END;
    
    -- Reduce score for recent activity
    IF recent_activity >= NOW() - INTERVAL '7 days' THEN
        risk_score := risk_score - 5;
    END IF;
    
    RETURN GREATEST(0, risk_score);
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically log database changes to sensitive tables
CREATE OR REPLACE FUNCTION audit_sensitive_table_changes()
RETURNS TRIGGER AS $$
BEGIN
    -- Log changes to users table
    IF TG_TABLE_NAME = 'users' THEN
        INSERT INTO audit_logs (event_type, severity, description, success, metadata)
        VALUES (
            CASE TG_OP
                WHEN 'INSERT' THEN 'UserRegistration'
                WHEN 'UPDATE' THEN 'UserModified'
                WHEN 'DELETE' THEN 'UserDeleted'
            END,
            'Medium',
            format('User record %s: %s', TG_OP, COALESCE(NEW.email, OLD.email)),
            true,
            jsonb_build_object(
                'table_name', TG_TABLE_NAME,
                'operation', TG_OP,
                'user_id', COALESCE(NEW.id, OLD.id),
                'changed_by', current_user
            )
        );
    END IF;
    
    -- Log changes to credit_transactions table
    IF TG_TABLE_NAME = 'credit_transactions' THEN
        INSERT INTO audit_logs (event_type, severity, description, success, metadata)
        VALUES (
            CASE 
                WHEN NEW.transaction_type = 'purchase' THEN 'CreditPurchase'
                WHEN NEW.transaction_type = 'deduction' THEN 'CreditDeduction'
                ELSE 'CreditTransaction'
            END,
            'Low',
            format('Credit transaction %s: $%.2f', TG_OP, COALESCE(NEW.amount, OLD.amount)),
            true,
            jsonb_build_object(
                'table_name', TG_TABLE_NAME,
                'operation', TG_OP,
                'transaction_id', COALESCE(NEW.id, OLD.id),
                'user_id', COALESCE(NEW.user_id, OLD.user_id),
                'amount', COALESCE(NEW.amount, OLD.amount)
            )
        );
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create triggers for sensitive tables
CREATE TRIGGER audit_users_changes
    AFTER INSERT OR UPDATE OR DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION audit_sensitive_table_changes();

CREATE TRIGGER audit_credit_transactions_changes
    AFTER INSERT OR UPDATE OR DELETE ON credit_transactions
    FOR EACH ROW EXECUTE FUNCTION audit_sensitive_table_changes();

-- Create scheduled job to run cleanup (requires pg_cron extension)
-- SELECT cron.schedule('cleanup-audit-logs', '0 2 * * 0', 'SELECT cleanup_old_audit_logs();');

-- Grant appropriate permissions
GRANT SELECT ON audit_logs TO readonly_user;
GRANT SELECT ON security_events_summary TO readonly_user;
GRANT SELECT ON failed_auth_attempts TO readonly_user;
GRANT SELECT ON suspicious_activities TO readonly_user;
GRANT SELECT ON user_activity_summary TO readonly_user;

-- Create comment for documentation
COMMENT ON TABLE audit_logs IS 'Comprehensive audit log for security events, user actions, and system changes';
COMMENT ON VIEW security_events_summary IS 'Summary of security events in the last 24 hours';
COMMENT ON VIEW failed_auth_attempts IS 'Failed authentication attempts grouped by IP address';
COMMENT ON VIEW suspicious_activities IS 'High-priority security events requiring attention';
COMMENT ON VIEW user_activity_summary IS 'User activity summary for the last 30 days';
COMMENT ON FUNCTION cleanup_old_audit_logs() IS 'Automated cleanup of audit logs older than 7 years';
COMMENT ON FUNCTION detect_brute_force_attempts() IS 'Detect potential brute force attacks based on failed login patterns';
COMMENT ON FUNCTION calculate_user_risk_score(UUID) IS 'Calculate risk score for a user based on their activity patterns';