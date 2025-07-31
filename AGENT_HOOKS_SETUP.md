# Agent Hooks Setup Guide

This document explains how to set up and configure Agent Hooks in Kiro IDE for the Unit Shopping Platform project.

## Overview

Agent Hooks are automated workflows that trigger when specific events occur in your development environment. They help maintain code quality, run tests, update documentation, and ensure security compliance automatically.

## Available Hooks

### 1. 🧪 **Rust Code Quality Check Hook**
- **Trigger**: Saving `.rs` files
- **Actions**: Format code, run lints, execute tests, check compilation
- **Benefits**: Maintains code quality and catches errors early

### 2. 🗄️ **Database Migration Hook**
- **Trigger**: Modifying migration files (`backend/migrations/*.sql`)
- **Actions**: Validate and run migrations, update schema, run database tests
- **Benefits**: Keeps database schema in sync and catches migration errors

### 3. ⛓️ **Smart Contract Compilation Hook**
- **Trigger**: Modifying Solidity files (`contracts/**/*.sol`)
- **Actions**: Compile contracts, generate types, run tests, analyze gas usage
- **Benefits**: Immediate feedback on contract changes and gas optimization

### 4. 📚 **Documentation Update Hook**
- **Trigger**: Modifying API handlers or models
- **Actions**: Generate API docs, update OpenAPI spec, validate documentation
- **Benefits**: Keeps documentation in sync with code changes

### 5. 🔒 **Security Audit Hook**
- **Trigger**: Modifying security-sensitive files
- **Actions**: Dependency audit, code analysis, crypto validation
- **Benefits**: Early detection of security vulnerabilities

### 6. ⚙️ **Environment Sync Hook**
- **Trigger**: Modifying configuration files (`.env*`, `docker-compose.yml`)
- **Actions**: Validate environment, sync services, update dependencies
- **Benefits**: Prevents runtime errors from configuration issues

## Setup Instructions

### Method 1: Using Kiro IDE Interface

1. **Open Agent Hooks Panel**
   - In Kiro IDE, go to the Explorer view
   - Find the "Agent Hooks" section
   - Click "Create New Hook"

2. **Configure Each Hook**
   - Use the hook configuration files in `.kiro/hooks/` as templates
   - Set appropriate triggers and file patterns
   - Configure actions and timeouts

### Method 2: Using Command Palette

1. **Open Command Palette**
   - Press `Ctrl+Shift+P` (Windows/Linux) or `Cmd+Shift+P` (Mac)
   - Type "Open Kiro Hook UI"
   - Select the command

2. **Create Hooks**
   - Follow the UI wizard to create each hook
   - Import configurations from the `.kiro/hooks/` directory

### Method 3: Manual Configuration

If you prefer to configure hooks manually, use the hook definition files in `.kiro/hooks/` as reference for the exact triggers, patterns, and actions.

## Hook Configuration Details

### File Patterns
- `**/*.rs` - All Rust files
- `backend/migrations/*.sql` - Database migration files
- `contracts/**/*.sol` - Smart contract files
- `backend/src/handlers/**/*.rs` - API handlers
- `backend/src/models/**/*.rs` - Data models
- `.env*` - Environment files
- `docker-compose.yml` - Docker configuration

### Common Actions
- **Code Formatting**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Testing**: `cargo test`
- **Compilation**: `cargo check`
- **Migration**: `sqlx migrate run`
- **Documentation**: `cargo doc`
- **Security Audit**: `cargo audit`

## Prerequisites

### Required Tools
Make sure these tools are installed for hooks to work properly:

```bash
# Rust toolchain
rustup component add rustfmt clippy

# Database tools
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Security tools
cargo install cargo-audit

# Smart contract tools (if using)
npm install -g hardhat
```

### Environment Setup
Ensure your development environment is properly configured:

1. **Database Running**
   ```bash
   docker-compose up -d postgres redis
   ```

2. **Environment Variables Set**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Dependencies Installed**
   ```bash
   cargo build
   ```

## Hook Behavior

### Automatic Execution
- Hooks run automatically when their trigger conditions are met
- Most hooks run in the background without interrupting your workflow
- Security and migration hooks may require your attention

### Notifications
- **Success**: Minimal notifications for successful operations
- **Warnings**: Notifications for potential issues that don't block development
- **Errors**: Prominent notifications for issues that need immediate attention

### Performance
- Hooks are optimized to run quickly (most under 30 seconds)
- Long-running operations (like full test suites) run in background
- Hooks can be temporarily disabled if needed

## Customization

### Modifying Hooks
You can customize hooks by:

1. **Adjusting File Patterns**: Change which files trigger the hook
2. **Modifying Actions**: Add or remove actions performed by the hook
3. **Setting Timeouts**: Adjust how long hooks are allowed to run
4. **Configuring Notifications**: Control when and how you're notified

### Adding New Hooks
To add custom hooks for your specific needs:

1. Create a new hook definition file in `.kiro/hooks/`
2. Define the trigger conditions and actions
3. Configure the hook in Kiro IDE
4. Test the hook with sample file changes

## Troubleshooting

### Common Issues

#### Hook Not Triggering
- Check file patterns match the files you're modifying
- Verify the hook is enabled in Kiro IDE
- Ensure trigger conditions are met

#### Hook Failing
- Check that required tools are installed
- Verify environment variables are set correctly
- Review hook logs for specific error messages

#### Performance Issues
- Adjust hook timeouts if operations are taking too long
- Consider running fewer actions per hook
- Disable hooks temporarily if they're interfering with development

### Debug Commands
```bash
# Test individual hook actions
cargo fmt --check
cargo clippy
cargo test
sqlx migrate info
cargo audit

# Check environment
echo $DATABASE_URL
docker-compose ps
```

## Best Practices

### Development Workflow
1. **Start with Essential Hooks**: Enable code quality and testing hooks first
2. **Gradual Adoption**: Add more hooks as you become comfortable with the workflow
3. **Team Coordination**: Ensure all team members have the same hooks configured
4. **Regular Review**: Periodically review and update hook configurations

### Performance Optimization
- Use specific file patterns to avoid unnecessary hook executions
- Set appropriate timeouts for different types of operations
- Consider the impact of hooks on development speed

### Security Considerations
- Security hooks should never auto-fix issues - they should alert you to review
- Regularly update security tools used by hooks
- Review security hook results carefully

## Integration with CI/CD

The hooks complement your CI/CD pipeline by:
- Catching issues before they reach version control
- Maintaining consistent code quality locally
- Reducing CI/CD pipeline failures
- Speeding up the development feedback loop

Your GitHub Actions workflow will run similar checks, but hooks provide immediate local feedback.

## Conclusion

Agent Hooks significantly improve development efficiency by automating routine tasks and catching issues early. Start with the basic code quality hooks and gradually add more as needed for your development workflow.

For questions or issues with hooks, refer to the Kiro IDE documentation or create an issue in your project repository.