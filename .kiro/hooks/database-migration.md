# Database Migration Hook

## Trigger
- **Event**: File Save
- **File Pattern**: `backend/migrations/*.sql`
- **Description**: Automatically run database migrations when migration files are modified

## Actions
1. **Validate Migration**: Check SQL syntax and migration format
2. **Run Migration**: Execute `sqlx migrate run` to apply changes
3. **Update Schema**: Regenerate database schema documentation
4. **Run Database Tests**: Execute tests that depend on database schema

## Benefits
- Keeps development database in sync with migration files
- Catches migration errors early
- Ensures database schema consistency across development team
- Automatically updates related documentation

## Implementation
```bash
# Navigate to backend directory
cd backend

# Validate migration syntax
sqlx migrate info

# Run pending migrations
sqlx migrate run

# Verify database state
sqlx migrate info

# Run database-related tests
cargo test --test database_tests
```

## Configuration
- **Auto-fix**: No
- **Show notifications**: Always
- **Run in background**: No (requires user attention)
- **Timeout**: 60 seconds

## Prerequisites
- PostgreSQL database running
- DATABASE_URL environment variable set
- sqlx-cli installed