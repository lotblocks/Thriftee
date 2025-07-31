#!/bin/bash

# Database migration script with rollback capabilities
# This script handles database migrations safely with backup and rollback options

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MIGRATION_DIR="${PROJECT_ROOT}/backend/migrations"
BACKUP_DIR="${PROJECT_ROOT}/database/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Validate environment
validate_environment() {
    log_info "Validating environment..."
    
    if [ -z "${DATABASE_URL:-}" ]; then
        log_error "DATABASE_URL environment variable is not set"
        exit 1
    fi
    
    # Test database connection
    if ! psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1; then
        log_error "Cannot connect to database"
        exit 1
    fi
    
    log_success "Environment validation completed"
}

# Create backup
create_backup() {
    log_info "Creating database backup..."
    
    mkdir -p "$BACKUP_DIR"
    local backup_file="${BACKUP_DIR}/backup_${TIMESTAMP}.sql"
    
    pg_dump "$DATABASE_URL" > "$backup_file"
    
    if [ -f "$backup_file" ]; then
        log_success "Backup created: $backup_file"
        echo "$backup_file"
    else
        log_error "Failed to create backup"
        exit 1
    fi
}

# Run migrations
run_migrations() {
    log_info "Running database migrations..."
    
    cd "$PROJECT_ROOT/backend"
    
    # Run migrations using sqlx
    if command -v sqlx &> /dev/null; then
        sqlx migrate run --database-url "$DATABASE_URL"
    else
        # Fallback to manual migration execution
        for migration_file in "$MIGRATION_DIR"/*.sql; do
            if [ -f "$migration_file" ]; then
                log_info "Applying migration: $(basename "$migration_file")"
                psql "$DATABASE_URL" -f "$migration_file"
            fi
        done
    fi
    
    log_success "Migrations completed successfully"
}

# Rollback to backup
rollback_to_backup() {
    local backup_file="$1"
    
    log_warning "Rolling back to backup: $backup_file"
    
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        exit 1
    fi
    
    # Drop and recreate database
    local db_name=$(echo "$DATABASE_URL" | sed 's/.*\///')
    local base_url=$(echo "$DATABASE_URL" | sed 's/\/[^\/]*$//')
    
    psql "$base_url/postgres" -c "DROP DATABASE IF EXISTS $db_name;"
    psql "$base_url/postgres" -c "CREATE DATABASE $db_name;"
    
    # Restore from backup
    psql "$DATABASE_URL" < "$backup_file"
    
    log_success "Rollback completed"
}

# Main function
main() {
    local action="${1:-migrate}"
    local backup_file="${2:-}"
    
    case "$action" in
        "migrate")
            validate_environment
            backup_file=$(create_backup)
            
            if run_migrations; then
                log_success "Migration completed successfully"
                log_info "Backup available at: $backup_file"
            else
                log_error "Migration failed, rolling back..."
                rollback_to_backup "$backup_file"
                exit 1
            fi
            ;;
        "rollback")
            if [ -z "$backup_file" ]; then
                log_error "Backup file required for rollback"
                exit 1
            fi
            rollback_to_backup "$backup_file"
            ;;
        *)
            echo "Usage: $0 {migrate|rollback} [backup_file]"
            exit 1
            ;;
    esac
}

main "$@"