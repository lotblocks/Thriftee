use actix_web::web;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::security::audit_logging::{AuditLogger, AuditEventType, AuditSeverity, AuditContext};
use crate::utils::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub affected_user_id: Option<Uuid>,
    pub source_ip: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub status: AlertStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    BruteForceAttack,
    SuspiciousLogin,
    UnauthorizedAccess,
    DataExfiltration,
    AnomalousActivity,
    SystemCompromise,
    PolicyViolation,
    RateLimitExceeded,
    MultipleFailedPayments,
    SuspiciousRaffleActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Open,
    InProgress,
    Resolved,
    FalsePositive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelligence {
    pub ip_address: String,
    pub threat_level: ThreatLevel,
    pub categories: Vec<ThreatCategory>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub reputation_score: i32, // -100 to 100
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatCategory {
    Malware,
    Botnet,
    Scanner,
    Spam,
    Phishing,
    Tor,
    Proxy,
    VPN,
    Suspicious,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRiskProfile {
    pub user_id: Uuid,
    pub risk_score: i32, // 0-100
    pub risk_factors: Vec<RiskFactor>,
    pub last_updated: DateTime<Utc>,
    pub monitoring_level: MonitoringLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: RiskFactorType,
    pub weight: f32,
    pub description: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactorType {
    MultipleFailedLogins,
    UnusualLoginLocation,
    UnusualLoginTime,
    MultipleDevices,
    SuspiciousPaymentPattern,
    RapidRaffleParticipation,
    AccountSharingIndicators,
    VPNUsage,
    TorUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringLevel {
    Normal,
    Enhanced,
    Strict,
}

pub struct SecurityMonitor {
    pool: PgPool,
    audit_logger: AuditLogger,
    threat_intel_cache: HashMap<String, ThreatIntelligence>,
    user_risk_cache: HashMap<Uuid, UserRiskProfile>,
}

impl SecurityMonitor {
    pub fn new(pool: PgPool, audit_logger: AuditLogger) -> Self {
        Self {
            pool,
            audit_logger,
            threat_intel_cache: HashMap::new(),
            user_risk_cache: HashMap::new(),
        }
    }

    /// Monitor for brute force attacks
    pub async fn check_brute_force_attacks(&self) -> Result<Vec<SecurityAlert>, AppError> {
        let mut alerts = Vec::new();

        // Query for potential brute force attempts
        let brute_force_attempts = sqlx::query!(
            r#"
            SELECT ip_address, attempt_count, last_attempt
            FROM detect_brute_force_attempts()
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for attempt in brute_force_attempts {
            if let Some(ip) = attempt.ip_address {
                let alert = SecurityAlert {
                    id: Uuid::new_v4(),
                    alert_type: AlertType::BruteForceAttack,
                    severity: if attempt.attempt_count > 10 {
                        AlertSeverity::Critical
                    } else {
                        AlertSeverity::High
                    },
                    title: "Brute Force Attack Detected".to_string(),
                    description: format!(
                        "Detected {} failed login attempts from IP {} in the last 15 minutes",
                        attempt.attempt_count, ip
                    ),
                    affected_user_id: None,
                    source_ip: Some(ip.to_string()),
                    metadata: serde_json::json!({
                        "attempt_count": attempt.attempt_count,
                        "last_attempt": attempt.last_attempt,
                        "detection_window": "15 minutes"
                    }),
                    created_at: Utc::now(),
                    resolved_at: None,
                    resolved_by: None,
                    status: AlertStatus::Open,
                };

                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    /// Monitor for suspicious login patterns
    pub async fn check_suspicious_logins(&self) -> Result<Vec<SecurityAlert>, AppError> {
        let mut alerts = Vec::new();

        // Check for logins from unusual locations
        let unusual_logins = sqlx::query!(
            r#"
            WITH user_locations AS (
                SELECT 
                    user_id,
                    ip_address,
                    COUNT(*) as login_count,
                    MAX(timestamp) as last_login
                FROM audit_logs
                WHERE event_type = 'UserLogin'
                    AND success = true
                    AND timestamp >= NOW() - INTERVAL '30 days'
                    AND user_id IS NOT NULL
                GROUP BY user_id, ip_address
            ),
            recent_logins AS (
                SELECT 
                    user_id,
                    ip_address,
                    timestamp
                FROM audit_logs
                WHERE event_type = 'UserLogin'
                    AND success = true
                    AND timestamp >= NOW() - INTERVAL '24 hours'
                    AND user_id IS NOT NULL
            )
            SELECT 
                rl.user_id,
                rl.ip_address,
                rl.timestamp,
                COALESCE(ul.login_count, 0) as historical_count
            FROM recent_logins rl
            LEFT JOIN user_locations ul ON rl.user_id = ul.user_id AND rl.ip_address = ul.ip_address
            WHERE COALESCE(ul.login_count, 0) = 0
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for login in unusual_logins {
            if let (Some(user_id), Some(ip)) = (login.user_id, login.ip_address) {
                let alert = SecurityAlert {
                    id: Uuid::new_v4(),
                    alert_type: AlertType::SuspiciousLogin,
                    severity: AlertSeverity::Medium,
                    title: "Login from New Location".to_string(),
                    description: format!(
                        "User logged in from a new IP address: {}",
                        ip
                    ),
                    affected_user_id: Some(user_id),
                    source_ip: Some(ip.to_string()),
                    metadata: serde_json::json!({
                        "login_time": login.timestamp,
                        "is_new_location": true
                    }),
                    created_at: Utc::now(),
                    resolved_at: None,
                    resolved_by: None,
                    status: AlertStatus::Open,
                };

                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    /// Monitor for anomalous user behavior
    pub async fn check_anomalous_activity(&self) -> Result<Vec<SecurityAlert>, AppError> {
        let mut alerts = Vec::new();

        // Check for rapid raffle participation
        let rapid_participation = sqlx::query!(
            r#"
            SELECT 
                user_id,
                COUNT(*) as purchase_count,
                SUM(CAST(metadata->>'amount' AS DECIMAL)) as total_amount
            FROM audit_logs
            WHERE event_type = 'BoxPurchased'
                AND timestamp >= NOW() - INTERVAL '1 hour'
                AND user_id IS NOT NULL
            GROUP BY user_id
            HAVING COUNT(*) > 20 OR SUM(CAST(metadata->>'amount' AS DECIMAL)) > 1000
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for activity in rapid_participation {
            if let Some(user_id) = activity.user_id {
                let alert = SecurityAlert {
                    id: Uuid::new_v4(),
                    alert_type: AlertType::AnomalousActivity,
                    severity: AlertSeverity::Medium,
                    title: "Rapid Raffle Participation".to_string(),
                    description: format!(
                        "User made {} raffle purchases totaling ${:.2} in the last hour",
                        activity.purchase_count,
                        activity.total_amount.unwrap_or_default()
                    ),
                    affected_user_id: Some(user_id),
                    source_ip: None,
                    metadata: serde_json::json!({
                        "purchase_count": activity.purchase_count,
                        "total_amount": activity.total_amount,
                        "time_window": "1 hour"
                    }),
                    created_at: Utc::now(),
                    resolved_at: None,
                    resolved_by: None,
                    status: AlertStatus::Open,
                };

                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    /// Calculate user risk score
    pub async fn calculate_user_risk_score(&self, user_id: Uuid) -> Result<UserRiskProfile, AppError> {
        let risk_score = sqlx::query_scalar!(
            "SELECT calculate_user_risk_score($1)",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // Get risk factors
        let mut risk_factors = Vec::new();

        // Check for failed logins
        let failed_logins = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::INTEGER
            FROM audit_logs
            WHERE user_id = $1
                AND event_type = 'UserLogin'
                AND success = false
                AND timestamp >= NOW() - INTERVAL '24 hours'
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        if failed_logins > 0 {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::MultipleFailedLogins,
                weight: (failed_logins as f32) * 0.1,
                description: format!("{} failed login attempts in last 24 hours", failed_logins),
                detected_at: Utc::now(),
            });
        }

        // Check for multiple devices
        let device_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT user_agent)::INTEGER
            FROM audit_logs
            WHERE user_id = $1
                AND timestamp >= NOW() - INTERVAL '30 days'
                AND user_agent IS NOT NULL
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        if device_count > 5 {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::MultipleDevices,
                weight: 0.2,
                description: format!("Used {} different devices in last 30 days", device_count),
                detected_at: Utc::now(),
            });
        }

        let monitoring_level = match risk_score {
            0..=30 => MonitoringLevel::Normal,
            31..=70 => MonitoringLevel::Enhanced,
            _ => MonitoringLevel::Strict,
        };

        Ok(UserRiskProfile {
            user_id,
            risk_score,
            risk_factors,
            last_updated: Utc::now(),
            monitoring_level,
        })
    }

    /// Check IP against threat intelligence
    pub async fn check_threat_intelligence(&self, ip_address: &str) -> Result<Option<ThreatIntelligence>, AppError> {
        // Check cache first
        if let Some(threat_intel) = self.threat_intel_cache.get(ip_address) {
            return Ok(Some(threat_intel.clone()));
        }

        // In a real implementation, this would query external threat intelligence APIs
        // For now, we'll implement basic checks

        // Check if IP is in known bad ranges
        let is_suspicious = self.is_suspicious_ip(ip_address).await?;

        if is_suspicious {
            let threat_intel = ThreatIntelligence {
                ip_address: ip_address.to_string(),
                threat_level: ThreatLevel::Medium,
                categories: vec![ThreatCategory::Suspicious],
                first_seen: Utc::now(),
                last_seen: Utc::now(),
                reputation_score: -50,
                source: "internal_analysis".to_string(),
            };

            Ok(Some(threat_intel))
        } else {
            Ok(None)
        }
    }

    /// Process and store security alerts
    pub async fn process_alert(&self, alert: SecurityAlert) -> Result<(), AppError> {
        // Store alert in database
        sqlx::query!(
            r#"
            INSERT INTO security_alerts (
                id, alert_type, severity, title, description,
                affected_user_id, source_ip, metadata, created_at, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            alert.id,
            serde_json::to_string(&alert.alert_type)?,
            serde_json::to_string(&alert.severity)?,
            alert.title,
            alert.description,
            alert.affected_user_id,
            alert.source_ip,
            alert.metadata,
            alert.created_at,
            serde_json::to_string(&alert.status)?
        )
        .execute(&self.pool)
        .await?;

        // Log to audit system
        let context = AuditContext {
            user_id: alert.affected_user_id,
            session_id: None,
            ip_address: alert.source_ip.clone(),
            user_agent: None,
            request_id: None,
            endpoint: None,
            method: None,
            additional_data: HashMap::new(),
        };

        self.audit_logger.log_security_event(
            AuditEventType::SuspiciousActivity,
            context,
            alert.description.clone(),
            match alert.severity {
                AlertSeverity::Low => AuditSeverity::Low,
                AlertSeverity::Medium => AuditSeverity::Medium,
                AlertSeverity::High => AuditSeverity::High,
                AlertSeverity::Critical => AuditSeverity::Critical,
            },
            Some(alert.metadata.clone()),
        ).await?;

        // Send notifications for high-severity alerts
        if matches!(alert.severity, AlertSeverity::High | AlertSeverity::Critical) {
            self.send_alert_notification(&alert).await?;
        }

        // Take automated actions for critical alerts
        if matches!(alert.severity, AlertSeverity::Critical) {
            self.take_automated_action(&alert).await?;
        }

        Ok(())
    }

    /// Run comprehensive security monitoring
    pub async fn run_security_scan(&self) -> Result<Vec<SecurityAlert>, AppError> {
        let mut all_alerts = Vec::new();

        // Check for brute force attacks
        let brute_force_alerts = self.check_brute_force_attacks().await?;
        all_alerts.extend(brute_force_alerts);

        // Check for suspicious logins
        let suspicious_login_alerts = self.check_suspicious_logins().await?;
        all_alerts.extend(suspicious_login_alerts);

        // Check for anomalous activity
        let anomalous_activity_alerts = self.check_anomalous_activity().await?;
        all_alerts.extend(anomalous_activity_alerts);

        // Process all alerts
        for alert in &all_alerts {
            if let Err(e) = self.process_alert(alert.clone()).await {
                error!("Failed to process security alert {}: {}", alert.id, e);
            }
        }

        info!("Security scan completed. Found {} alerts", all_alerts.len());

        Ok(all_alerts)
    }

    // Private helper methods

    async fn is_suspicious_ip(&self, ip_address: &str) -> Result<bool, AppError> {
        // Check if IP has had multiple failed attempts
        let failed_attempts = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::INTEGER
            FROM audit_logs
            WHERE ip_address = $1::INET
                AND event_type = 'UserLogin'
                AND success = false
                AND timestamp >= NOW() - INTERVAL '24 hours'
            "#,
            ip_address
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(failed_attempts > 10)
    }

    async fn send_alert_notification(&self, alert: &SecurityAlert) -> Result<(), AppError> {
        // In a real implementation, this would send notifications via:
        // - Email to security team
        // - Slack/Teams webhook
        // - PagerDuty for critical alerts
        // - SMS for critical alerts

        warn!(
            "SECURITY ALERT [{}]: {} - {}",
            serde_json::to_string(&alert.severity).unwrap_or_default(),
            alert.title,
            alert.description
        );

        Ok(())
    }

    async fn take_automated_action(&self, alert: &SecurityAlert) -> Result<(), AppError> {
        match alert.alert_type {
            AlertType::BruteForceAttack => {
                if let Some(ip) = &alert.source_ip {
                    // Temporarily block IP
                    self.block_ip_address(ip, Duration::hours(1)).await?;
                }
            }
            AlertType::SystemCompromise => {
                // Trigger emergency procedures
                self.trigger_emergency_response().await?;
            }
            _ => {
                // No automated action for other alert types
            }
        }

        Ok(())
    }

    async fn block_ip_address(&self, ip_address: &str, duration: Duration) -> Result<(), AppError> {
        // Store IP block in database or cache
        info!("Blocking IP address {} for {} minutes", ip_address, duration.num_minutes());
        
        // This would integrate with your rate limiting system
        // to temporarily block the IP address
        
        Ok(())
    }

    async fn trigger_emergency_response(&self) -> Result<(), AppError> {
        // Trigger emergency response procedures
        error!("EMERGENCY: System compromise detected - triggering emergency response");
        
        // This would:
        // - Send immediate alerts to security team
        // - Potentially disable certain system functions
        // - Increase logging and monitoring
        // - Trigger incident response procedures
        
        Ok(())
    }
}

// Background task for continuous monitoring
pub async fn start_security_monitoring(monitor: SecurityMonitor) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

    loop {
        interval.tick().await;

        if let Err(e) = monitor.run_security_scan().await {
            error!("Security monitoring scan failed: {}", e);
        }
    }
}