# Rust Code Quality Check Hook

## Trigger
- **Event**: File Save
- **File Pattern**: `**/*.rs`
- **Description**: Automatically run code quality checks when Rust files are saved

## Actions
1. **Format Code**: Run `cargo fmt` to ensure consistent formatting
2. **Lint Code**: Run `cargo clippy` to catch common mistakes and improve code quality
3. **Run Tests**: Execute `cargo test` for the modified module
4. **Check Compilation**: Verify code compiles without errors

## Benefits
- Maintains consistent code quality across the project
- Catches errors early in development
- Ensures adherence to Rust best practices
- Reduces time spent in code review on formatting issues

## Implementation
```bash
# Format code
cargo fmt --check

# Run clippy with all lints
cargo clippy --all-targets --all-features -- -D warnings

# Run tests for the current workspace
cargo test

# Check compilation
cargo check
```

## Configuration
- **Auto-fix**: Yes (formatting)
- **Show notifications**: On errors only
- **Run in background**: Yes
- **Timeout**: 30 seconds