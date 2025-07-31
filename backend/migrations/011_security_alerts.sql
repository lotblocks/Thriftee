-- Migration: Create security alerts table
-- Description: Store and manage security alerts and incidents

-- Create security alerts table
CREATE TABLE security_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    
    -- Affected entities
    affected_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    source_ip INET,
    
    -- Alert metadata
    metadata JSONB DEFAULT '{}',
    
    -- Alert lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'Open',
    
    -- Resolution details
    resolution_notes TEXT,
    false_positive BOOLEAN DEFAULT FALSE,
    
    -- Tracking
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX idx_security_alerts_created_at ON security_alerts(created_at DESC);
CREATE INDEX idx_security_alerts_severity ON security_alerts(severity);
CREATE INDEX idx_security_alerts_status ON security_alerts(status);
CREATE INDEX idx_security_alerts_alert_type ON security_alerts(alert_type);
CREATE INDEX idx_security_alerts_affected_user ON security_alerts(affected_user_id) WHERE affected_user_id IS NOT NULL;
CREATE INDEX idx_security_alerts_source_ip ON security_alerts(source_ip) WHERE source_ip IS NOT NULL;
CREATE INDEX idx_security_alerts_unresolved ON security_alerts(created_at DESC) WHERE status IN ('Open', 'InProgress');

-- Create composite indexes
CREATE INDEX idx_security_alerts_severity_status ON security_alerts(severity, status, created_at DESC);
CREATE INDEX idx_security_alerts_type_severity ON security_alerts(alert_type, severity, created_at DESC);

-- Create GIN index for metadata
CREATE INDEX idx_security_alerts_metadata ON security_alerts USING GIN(metadata);

-- Create IP blocklist table
CREATE TABLE ip_blocklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL UNIQUE,
    reason TEXT NOT NULL,
    blocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blocked_by UUID REFERENCES users(id) ON DELETE SET NULL,
    expires_at TIMESTAMPTZ,
    is_permanent BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for IP blocklist
CREATE INDEX idx_ip_blocklist_ip_address ON ip_blocklist(ip_address);
CREATE INDEX idx_ip_blocklist_expires_at ON ip_blocklist(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_ip_blocklist_active ON ip_blocklist(blocked_at) WHERE is_permanent = TRUE OR expires_at > NOW();

-- Create user risk profiles table
CREATE TABLE user_risk_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    risk_score INTEGER NOT NULL DEFAULT 0 CHECK (risk_score >= 0 AND risk_score <= 100),
    risk_factors JSONB DEFAULT '[]',
    monitoring_level TEXT NOT NULL DEFAULT 'Normal',
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id)
);

-- Create indexes for user risk profiles
CREATE INDEX idx_user_risk_profiles_user_id ON user_risk_profiles(user_id);
CREATE INDEX idx_user_risk_profiles_risk_score ON user_risk_profiles(risk_score DESC);
CREATE INDEX idx_user_risk_profiles_monitoring_level ON user_risk_profiles(monitoring_level);
CREATE INDEX idx_user_risk_profiles_last_updated ON user_risk_profiles(last_updated DESC);

-- Create threat intelligence table
CREATE TABLE threat_intelligence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL,
    threat_level TEXT NOT NULL,
    categories TEXT[] NOT NULL DEFAULT '{}',
    reputation_score INTEGER NOT NULL DEFAULT 0 CHECK (reputation_score >= -100 AND reputation_score <= 100),
    source TEXT NOT NULL,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(ip_address, source)
);

-- Create indexes for threat intelligence
CREATE INDEX idx_threat_intelligence_ip_address ON threat_intelligence(ip_address);
CREATE INDEX idx_threat_intelligence_threat_level ON threat_intelligence(threat_level);
CREATE INDEX idx_threat_intelligence_reputation_score ON threat_intelligence(reputation_score);
CREATE INDEX idx_threat_intelligence_last_seen ON threat_intelligence(last_seen DESC);
CREATE INDEX idx_threat_intelligence_categories ON threat_intelligence USING GIN(categories);

-- Create security incidents table for major incidents
CREATE TABLE security_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    
    -- Incident details
    affected_systems TEXT[],
    affected_users UUID[],
    impact_assessment TEXT,
    
    -- Timeline
    detected_at TIMESTAMPTZ NOT NULL,
    reported_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    
    -- Assignment
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    reporter_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Status tracking
    status TEXT NOT NULL DEFAULT 'Open',
    priority TEXT NOT NULL DEFAULT 'Medium',
    
    -- Resolution
    resolution_summary TEXT,
    lessons_learned TEXT,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for security incidents
CREATE INDEX idx_security_incidents_detected_at ON security_incidents(detected_at DESC);
CREATE INDEX idx_security_incidents_severity ON security_incidents(severity);
CREATE INDEX idx_security_incidents_status ON security_incidents(status);
CREATE INDEX idx_security_incidents_priority ON security_incidents(priority);
CREATE INDEX idx_security_incidents_assigned_to ON security_incidents(assigned_to) WHERE assigned_to IS NOT NULL;
CREATE INDEX idx_security_incidents_open ON security_incidents(detected_at DESC) WHERE status IN ('Open', 'InProgress');

-- Create views for security dashboard

-- Active security alerts view
CREATE VIEW active_security_alerts AS
SELECT 
    id,
    alert_type,
    severity,
    title,
    description,
    affected_user_id,
    source_ip,
    created_at,
    status,
    EXTRACT(EPOCH FROM (NOW() - created_at))/3600 as hours_open
FROM security_alerts
WHERE status IN ('Open', 'InProgress')
ORDER BY 
    CASE severity 
        WHEN 'Critical' THEN 1 
        WHEN 'High' THEN 2 
        WHEN 'Medium' THEN 3 
        WHEN 'Low' THEN 4 
    END,
    created_at DESC;

-- Security metrics summary view
CREATE VIEW security_metrics_summary AS
SELECT 
    COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '24 hours') as alerts_24h,
    COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '7 days') as alerts_7d,
    COUNT(*) FILTER (WHERE severity = 'Critical' AND created_at >= NOW() - INTERVAL '24 hours') as critical_24h,
    COUNT(*) FILTER (WHERE severity = 'High' AND created_at >= NOW() - INTERVAL '24 hours') as high_24h,
    COUNT(*) FILTER (WHERE status IN ('Open', 'InProgress')) as open_alerts,
    AVG(EXTRACT(EPOCH FROM (COALESCE(resolved_at, NOW()) - created_at))/3600) FILTER (WHERE resolved_at IS NOT NULL) as avg_resolution_hours
FROM security_alerts;

-- Top threat IPs view
CREATE VIEW top_threat_ips AS
SELECT 
    source_ip,
    COUNT(*) as alert_count,
    MAX(created_at) as last_alert,
    array_agg(DISTINCT alert_type) as alert_types,
    MAX(CASE severity 
        WHEN 'Critical' THEN 4 
        WHEN 'High' THEN 3 
        WHEN 'Medium' THEN 2 
        WHEN 'Low' THEN 1 
    END) as max_severity_level
FROM security_alerts
WHERE source_ip IS NOT NULL
    AND created_at >= NOW() - INTERVAL '7 days'
GROUP BY source_ip
ORDER BY alert_count DESC, max_severity_level DESC
LIMIT 50;

-- User risk summary view
CREATE VIEW user_risk_summary AS
SELECT 
    urp.user_id,
    u.email,
    urp.risk_score,
    urp.monitoring_level,
    urp.last_updated,
    COUNT(sa.id) as recent_alerts,
    MAX(sa.created_at) as last_alert
FROM user_risk_profiles urp
JOIN users u ON urp.user_id = u.id
LEFT JOIN security_alerts sa ON urp.user_id = sa.affected_user_id 
    AND sa.created_at >= NOW() - INTERVAL '30 days'
GROUP BY urp.user_id, u.email, urp.risk_score, urp.monitoring_level, urp.last_updated
ORDER BY urp.risk_score DESC, recent_alerts DESC;

-- Create functions for security operations

-- Function to check if IP is blocked
CREATE OR REPLACE FUNCTION is_ip_blocked(check_ip INET)
RETURNS BOOLEAN AS $
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM ip_blocklist
        WHERE ip_address = check_ip
            AND (is_permanent = TRUE OR expires_at > NOW())
    );
END;
$ LANGUAGE plpgsql;

-- Function to block IP address
CREATE OR REPLACE FUNCTION block_ip_address(
    target_ip INET,
    block_reason TEXT,
    blocking_user_id UUID DEFAULT NULL,
    expiry_duration INTERVAL DEFAULT NULL
)
RETURNS UUID AS $
DECLARE
    block_id UUID;
BEGIN
    INSERT INTO ip_blocklist (
        ip_address,
        reason,
        blocked_by,
        expires_at,
        is_permanent
    ) VALUES (
        target_ip,
        block_reason,
        blocking_user_id,
        CASE WHEN expiry_duration IS NOT NULL THEN NOW() + expiry_duration ELSE NULL END,
        expiry_duration IS NULL
    )
    ON CONFLICT (ip_address) DO UPDATE SET
        reason = EXCLUDED.reason,
        blocked_at = NOW(),
        blocked_by = EXCLUDED.blocked_by,
        expires_at = EXCLUDED.expires_at,
        is_permanent = EXCLUDED.is_permanent
    RETURNING id INTO block_id;
    
    -- Log the blocking action
    INSERT INTO audit_logs (event_type, severity, description, success, metadata)
    VALUES (
        'SecurityPolicyChanged',
        'High',
        format('IP address %s blocked: %s', target_ip, block_reason),
        true,
        jsonb_build_object(
            'ip_address', target_ip,
            'reason', block_reason,
            'blocked_by', blocking_user_id,
            'expires_at', CASE WHEN expiry_duration IS NOT NULL THEN NOW() + expiry_duration ELSE NULL END
        )
    );
    
    RETURN block_id;
END;
$ LANGUAGE plpgsql;

-- Function to update user risk score
CREATE OR REPLACE FUNCTION update_user_risk_score(
    target_user_id UUID,
    new_risk_score INTEGER,
    risk_factors_json JSONB DEFAULT '[]'
)
RETURNS VOID AS $
DECLARE
    monitoring_level TEXT;
BEGIN
    -- Determine monitoring level based on risk score
    monitoring_level := CASE 
        WHEN new_risk_score <= 30 THEN 'Normal'
        WHEN new_risk_score <= 70 THEN 'Enhanced'
        ELSE 'Strict'
    END;
    
    INSERT INTO user_risk_profiles (
        user_id,
        risk_score,
        risk_factors,
        monitoring_level
    ) VALUES (
        target_user_id,
        new_risk_score,
        risk_factors_json,
        monitoring_level
    )
    ON CONFLICT (user_id) DO UPDATE SET
        risk_score = EXCLUDED.risk_score,
        risk_factors = EXCLUDED.risk_factors,
        monitoring_level = EXCLUDED.monitoring_level,
        last_updated = NOW();
    
    -- Log risk score update
    INSERT INTO audit_logs (event_type, severity, description, success, metadata)
    VALUES (
        'SecurityPolicyChanged',
        'Medium',
        format('User risk score updated to %s', new_risk_score),
        true,
        jsonb_build_object(
            'user_id', target_user_id,
            'risk_score', new_risk_score,
            'monitoring_level', monitoring_level
        )
    );
END;
$ LANGUAGE plpgsql;

-- Function to clean up expired blocks and old alerts
CREATE OR REPLACE FUNCTION cleanup_security_data()
RETURNS INTEGER AS $
DECLARE
    cleaned_count INTEGER := 0;
BEGIN
    -- Remove expired IP blocks
    DELETE FROM ip_blocklist 
    WHERE is_permanent = FALSE AND expires_at < NOW();
    
    GET DIAGNOSTICS cleaned_count = ROW_COUNT;
    
    -- Archive old resolved alerts (keep for 1 year)
    DELETE FROM security_alerts 
    WHERE status = 'Resolved' 
        AND resolved_at < NOW() - INTERVAL '1 year';
    
    -- Update threat intelligence last_seen for active IPs
    UPDATE threat_intelligence 
    SET last_seen = NOW()
    WHERE ip_address IN (
        SELECT DISTINCT ip_address 
        FROM audit_logs 
        WHERE timestamp >= NOW() - INTERVAL '24 hours'
            AND ip_address IS NOT NULL
    );
    
    RETURN cleaned_count;
END;
$ LANGUAGE plpgsql;

-- Create triggers for automatic updates

-- Update timestamp trigger for security_alerts
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER update_security_alerts_updated_at
    BEFORE UPDATE ON security_alerts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_risk_profiles_updated_at
    BEFORE UPDATE ON user_risk_profiles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_threat_intelligence_updated_at
    BEFORE UPDATE ON threat_intelligence
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_security_incidents_updated_at
    BEFORE UPDATE ON security_incidents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Grant appropriate permissions
GRANT SELECT ON security_alerts TO readonly_user;
GRANT SELECT ON active_security_alerts TO readonly_user;
GRANT SELECT ON security_metrics_summary TO readonly_user;
GRANT SELECT ON top_threat_ips TO readonly_user;
GRANT SELECT ON user_risk_summary TO readonly_user;

-- Create comments for documentation
COMMENT ON TABLE security_alerts IS 'Security alerts and incidents detected by the monitoring system';
COMMENT ON TABLE ip_blocklist IS 'IP addresses that are temporarily or permanently blocked';
COMMENT ON TABLE user_risk_profiles IS 'Risk assessment profiles for users based on their behavior';
COMMENT ON TABLE threat_intelligence IS 'Threat intelligence data for IP addresses and other indicators';
COMMENT ON TABLE security_incidents IS 'Major security incidents requiring investigation and response';

COMMENT ON VIEW active_security_alerts IS 'Currently active security alerts requiring attention';
COMMENT ON VIEW security_metrics_summary IS 'Summary metrics for security dashboard';
COMMENT ON VIEW top_threat_ips IS 'IP addresses with the most security alerts in the last 7 days';
COMMENT ON VIEW user_risk_summary IS 'Summary of user risk profiles and recent security events';

COMMENT ON FUNCTION is_ip_blocked(INET) IS 'Check if an IP address is currently blocked';
COMMENT ON FUNCTION block_ip_address(INET, TEXT, UUID, INTERVAL) IS 'Block an IP address with optional expiration';
COMMENT ON FUNCTION update_user_risk_score(UUID, INTEGER, JSONB) IS 'Update user risk score and monitoring level';
COMMENT ON FUNCTION cleanup_security_data() IS 'Clean up expired security data and update threat intelligence';