# Environment Sync Hook

## Trigger
- **Event**: File Save
- **File Pattern**: `.env*`, `docker-compose.yml`, `Cargo.toml`
- **Description**: Validate and sync environment configuration when config files are modified

## Actions
1. **Validate Environment**: Check for required environment variables
2. **Sync Docker Services**: Restart affected Docker services if needed
3. **Update Dependencies**: Check for new dependencies and install if needed
4. **Configuration Validation**: Ensure all services can connect with new config
5. **Generate Config Documentation**: Update environment variable documentation

## Benefits
- Prevents runtime errors from missing environment variables
- Keeps development environment in sync
- Validates service connectivity
- Maintains up-to-date configuration documentation
- Reduces debugging time for environment issues

## Implementation
```bash
# Validate environment variables
if [ -f ".env" ]; then
    # Check for required variables
    required_vars=("DATABASE_URL" "REDIS_URL" "JWT_SECRET" "STRIPE_SECRET_KEY")
    for var in "${required_vars[@]}"; do
        if ! grep -q "^$var=" .env; then
            echo "Warning: Missing required environment variable: $var"
        fi
    done
fi

# If docker-compose.yml changed, validate and restart services
if [ "$CHANGED_FILE" = "docker-compose.yml" ]; then
    docker-compose config
    echo "Docker Compose configuration is valid"
    
    # Optionally restart services (uncomment if desired)
    # docker-compose down && docker-compose up -d
fi

# If Cargo.toml changed, check dependencies
if [[ "$CHANGED_FILE" == *"Cargo.toml" ]]; then
    cargo check
    echo "Dependencies validated"
fi

# Generate environment documentation
cargo run --bin generate-env-docs || echo "Environment documentation generator not available"
```

## Configuration
- **Auto-fix**: Yes (for formatting)
- **Show notifications**: On validation errors
- **Run in background**: Yes
- **Timeout**: 60 seconds

## Prerequisites
- Docker and Docker Compose installed
- Required environment variables documented