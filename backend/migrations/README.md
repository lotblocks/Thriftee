# Database Migrations

This directory contains all database migration files for the Raffle Shopping Platform. The migrations are designed to be run in order and include both forward migrations and rollback scripts.

## Migration Files

### Forward Migrations
- `001_initial_schema.sql` - Creates the core database schema with all primary tables
- `002_add_constraints_and_relationships.sql` - Adds constraints, indexes, and validation rules
- `003_add_additional_tables.sql` - Adds supporting tables for sessions, notifications, and audit logs

### Rollback Migrations
- `rollback_001_drop_initial_schema.sql` - Removes all tables and types from initial schema
- `rollback_002_remove_constraints.sql` - Removes constraints and indexes added in migration 002
- `rollback_003_remove_additional_tables.sql` - Removes additional tables added in migration 003

## Database Schema Overview

### Core Tables

#### Users and Authentication
- `users` - User accounts with authentication details and internal wallet addresses
- `user_sessions` - JWT refresh token management
- `email_verification_tokens` - Email verification tokens
- `password_reset_tokens` - Password reset tokens

#### Seller Management
- `sellers` - Seller profiles and business information
- `seller_subscriptions` - Subscription tier definitions

#### Items and Raffles
- `items` - Product listings with pricing and inventory
- `raffles` - Raffle configurations and status
- `box_purchases` - Individual box purchases within raffles

#### Credits and Transactions
- `user_credits` - User credit balances and expiration tracking
- `transactions` - Financial transaction records
- `free_redeemable_items` - Items available for credit redemption

#### System Management
- `notifications` - User notifications
- `audit_logs` - Security and compliance audit trail
- `system_settings` - Platform configuration

### Database Views

#### `raffle_participants`
Provides a consolidated view of raffle participation including:
- User participation details
- Box purchase counts and spending
- Winner status

#### `user_credit_summary`
Summarizes user credit information including:
- Available general and item-specific credits
- Credits expiring soon
- Expired credits count

#### `seller_performance`
Tracks seller performance metrics including:
- Total items listed and raffles completed
- Revenue tracking and completion rates

## Custom Types

The database uses several custom ENUM types for data consistency:

- `user_role` - User permission levels (user, seller, admin, operator)
- `item_status` - Item availability status (available, sold, inactive)
- `raffle_status` - Raffle lifecycle status (open, full, drawing, completed, cancelled)
- `credit_source` - Source of credit issuance (raffle_loss, deposit, refund, bonus)
- `credit_type` - Credit usage scope (general, item_specific)
- `transaction_type` - Financial transaction categories

## Running Migrations

### Using the Migration Scripts

#### On Unix/Linux/macOS:
```bash
# Run all pending migrations
./scripts/run-migrations.sh up

# Check migration status
./scripts/run-migrations.sh status

# Rollback 1 migration
./scripts/run-migrations.sh down

# Rollback 3 migrations
./scripts/run-migrations.sh down 3

# Reset entire database
./scripts/run-migrations.sh reset

# Fresh migration (reset + up)
./scripts/run-migrations.sh fresh
```

#### On Windows (PowerShell):
```powershell
# Run all pending migrations
.\scripts\run-migrations.ps1 up

# Check migration status
.\scripts\run-migrations.ps1 status

# Rollback 1 migration
.\scripts\run-migrations.ps1 down

# Rollback 3 migrations
.\scripts\run-migrations.ps1 down 3

# Reset entire database
.\scripts\run-migrations.ps1 reset

# Fresh migration (reset + up)
.\scripts\run-migrations.ps1 fresh
```

### Using Cargo Directly

```bash
cd backend

# Run migrations
cargo run --bin migrate up

# Check status
cargo run --bin migrate status

# Rollback
cargo run --bin migrate down [steps]

# Reset database
cargo run --bin migrate reset
```

## Migration Best Practices

### Creating New Migrations

1. **Naming Convention**: Use the format `XXX_descriptive_name.sql` where XXX is a zero-padded number
2. **Rollback Scripts**: Always create a corresponding `rollback_XXX_descriptive_name.sql` file
3. **Atomic Operations**: Each migration should be atomic and reversible
4. **Data Safety**: Never delete data without proper backup procedures

### Migration Guidelines

1. **Test Thoroughly**: Test both forward and rollback migrations on development data
2. **Backup First**: Always backup production data before running migrations
3. **Monitor Performance**: Large migrations should be tested for performance impact
4. **Document Changes**: Include comments explaining complex changes

### Example Migration Structure

```sql
-- Forward migration: 004_add_new_feature.sql
-- Description: Add support for new feature X

-- Create new table
CREATE TABLE new_feature (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_new_feature_name ON new_feature(name);

-- Add constraints
ALTER TABLE existing_table 
ADD COLUMN new_feature_id UUID REFERENCES new_feature(id);
```

```sql
-- Rollback migration: rollback_004_add_new_feature.sql
-- Description: Remove new feature X support

-- Remove constraints
ALTER TABLE existing_table 
DROP COLUMN IF EXISTS new_feature_id;

-- Drop indexes
DROP INDEX IF EXISTS idx_new_feature_name;

-- Drop table
DROP TABLE IF EXISTS new_feature;
```

## Troubleshooting

### Common Issues

1. **Connection Errors**: Ensure DATABASE_URL is correctly set in .env file
2. **Permission Errors**: Ensure database user has CREATE/DROP privileges
3. **Migration Conflicts**: Check for conflicting schema changes
4. **Rollback Failures**: Verify rollback scripts match forward migration changes

### Recovery Procedures

1. **Failed Migration**: Check error logs and fix the migration file
2. **Partial Application**: Use rollback to revert and fix the issue
3. **Data Corruption**: Restore from backup and replay migrations
4. **Schema Mismatch**: Use `status` command to verify current state

## Security Considerations

1. **Sensitive Data**: Never include sensitive data in migration files
2. **Access Control**: Limit migration execution to authorized personnel
3. **Audit Trail**: All migrations are logged in the `schema_migrations` table
4. **Backup Strategy**: Implement automated backups before production migrations

## Performance Considerations

1. **Index Creation**: Create indexes concurrently when possible
2. **Large Tables**: Consider batched operations for large data changes
3. **Downtime**: Plan for maintenance windows during major schema changes
4. **Monitoring**: Monitor database performance during and after migrations