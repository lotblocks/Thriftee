# Documentation Update Hook

## Trigger
- **Event**: File Save
- **File Pattern**: `backend/src/handlers/**/*.rs`, `backend/src/models/**/*.rs`
- **Description**: Automatically update API documentation when handlers or models are modified

## Actions
1. **Generate API Docs**: Extract API documentation from code comments
2. **Update OpenAPI Spec**: Regenerate OpenAPI/Swagger specification
3. **Validate Documentation**: Check for missing or outdated documentation
4. **Update README**: Refresh API endpoint listings in README files

## Benefits
- Keeps API documentation in sync with code changes
- Ensures all endpoints are properly documented
- Reduces manual documentation maintenance
- Improves developer experience for API consumers

## Implementation
```bash
# Generate Rust documentation
cargo doc --no-deps --open

# Extract API documentation (if using utoipa or similar)
cargo run --bin generate-openapi-spec

# Validate documentation completeness
cargo run --bin validate-docs

# Update README with latest API endpoints
cargo run --bin update-readme-apis
```

## Configuration
- **Auto-fix**: Yes (formatting)
- **Show notifications**: On missing documentation
- **Run in background**: Yes
- **Timeout**: 45 seconds

## Prerequisites
- Documentation generation tools configured
- Proper doc comments in code