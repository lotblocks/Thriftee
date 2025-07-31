-- Rollback migration for 003_add_additional_tables.sql
-- This script removes all tables and objects created in migration 003

-- Drop views first (they depend on tables)
DROP VIEW IF EXISTS seller_performance;
DROP VIEW IF EXISTS user_credit_summary;
DROP VIEW IF EXISTS raffle_participants;

-- Drop triggers
DROP TRIGGER IF EXISTS update_system_settings_updated_at ON system_settings;
DROP TRIGGER IF EXISTS update_user_sessions_updated_at ON user_sessions;

-- Drop indexes
DROP INDEX IF EXISTS idx_system_settings_key;
DROP INDEX IF EXISTS idx_audit_logs_created_at;
DROP INDEX IF EXISTS idx_audit_logs_action;
DROP INDEX IF EXISTS idx_audit_logs_user_id;
DROP INDEX IF EXISTS idx_notifications_user_read;
DROP INDEX IF EXISTS idx_notifications_user_id;
DROP INDEX IF EXISTS idx_password_reset_tokens_token;
DROP INDEX IF EXISTS idx_password_reset_tokens_user_id;
DROP INDEX IF EXISTS idx_email_verification_tokens_token;
DROP INDEX IF EXISTS idx_email_verification_tokens_user_id;
DROP INDEX IF EXISTS idx_user_sessions_expires_at;
DROP INDEX IF EXISTS idx_user_sessions_refresh_token;
DROP INDEX IF EXISTS idx_user_sessions_user_id;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS system_settings;
DROP TABLE IF EXISTS audit_logs;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS password_reset_tokens;
DROP TABLE IF EXISTS email_verification_tokens;
DROP TABLE IF EXISTS user_sessions;