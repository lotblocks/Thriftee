#!/bin/bash

# Database migration script for Raffle Platform
# This script provides a convenient way to run database migrations

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the correct directory
if [ ! -f "backend/Cargo.toml" ]; then
    log_error "This script must be run from the project root directory"
    exit 1
fi

# Check if .env file exists
if [ ! -f ".env" ]; then
    log_error ".env file not found. Please create one with DATABASE_URL"
    exit 1
fi

# Load environment variables
source .env

# Check if DATABASE_URL is set
if [ -z "${DATABASE_URL:-}" ]; then
    log_error "DATABASE_URL environment variable is not set"
    exit 1
fi

# Change to backend directory
cd backend

# Parse command line arguments
COMMAND=${1:-"up"}

case $COMMAND in
    "up")
        log_info "Running database migrations..."
        cargo run --bin migrate up
        log_success "Database migrations completed successfully"
        ;;
    "down")
        STEPS=${2:-1}
        log_warning "Rolling back $STEPS migration(s)..."
        cargo run --bin migrate down $STEPS
        log_success "Database rollback completed successfully"
        ;;
    "status")
        log_info "Checking migration status..."
        cargo run --bin migrate status
        ;;
    "reset")
        log_warning "This will reset the entire database and delete all data!"
        read -p "Are you sure you want to continue? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            log_info "Resetting database..."
            cargo run --bin migrate reset
            log_success "Database reset completed"
        else
            log_info "Database reset cancelled"
        fi
        ;;
    "fresh")
        log_info "Running fresh migration (reset + up)..."
        log_warning "This will delete all existing data!"
        read -p "Are you sure you want to continue? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            cargo run --bin migrate reset
            cargo run --bin migrate up
            log_success "Fresh migration completed successfully"
        else
            log_info "Fresh migration cancelled"
        fi
        ;;
    "help"|"-h"|"--help")
        echo "Database Migration Tool"
        echo "======================="
        echo ""
        echo "Usage: $0 [command] [options]"
        echo ""
        echo "Commands:"
        echo "  up              Run all pending migrations (default)"
        echo "  down [steps]    Rollback migrations (default: 1 step)"
        echo "  status          Show migration status"
        echo "  reset           Drop all tables and reset database"
        echo "  fresh           Reset database and run all migrations"
        echo "  help            Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0                    # Run all pending migrations"
        echo "  $0 up                 # Run all pending migrations"
        echo "  $0 down               # Rollback 1 migration"
        echo "  $0 down 3             # Rollback 3 migrations"
        echo "  $0 status             # Show migration status"
        echo "  $0 reset              # Reset entire database"
        echo "  $0 fresh              # Reset and run all migrations"
        ;;
    *)
        log_error "Unknown command: $COMMAND"
        echo "Use '$0 help' to see available commands"
        exit 1
        ;;
esac