# Unit Shopping Platform Setup Script for Windows

Write-Host "Setting up Unit Shopping Platform..." -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust is not installed. Please install Rust first:" -ForegroundColor Yellow
    Write-Host "1. Visit https://rustup.rs/" -ForegroundColor Yellow
    Write-Host "2. Download and run rustup-init.exe" -ForegroundColor Yellow
    Write-Host "3. Restart your terminal and run this script again" -ForegroundColor Yellow
    exit 1
}

# Check if Docker is installed
if (!(Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "Docker is not installed. Please install Docker Desktop:" -ForegroundColor Yellow
    Write-Host "1. Visit https://www.docker.com/products/docker-desktop/" -ForegroundColor Yellow
    Write-Host "2. Download and install Docker Desktop" -ForegroundColor Yellow
    Write-Host "3. Restart your terminal and run this script again" -ForegroundColor Yellow
    exit 1
}

# Install sqlx-cli if not present
Write-Host "Installing sqlx-cli..." -ForegroundColor Blue
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Start database services
Write-Host "Starting database services..." -ForegroundColor Blue
docker-compose up -d postgres redis

# Wait for services to be ready
Write-Host "Waiting for services to be ready..." -ForegroundColor Blue
Start-Sleep -Seconds 10

# Run database migrations
Write-Host "Running database migrations..." -ForegroundColor Blue
Set-Location backend
sqlx migrate run
Set-Location ..

# Build the project
Write-Host "Building the project..." -ForegroundColor Blue
cargo build

Write-Host "Setup complete! You can now run:" -ForegroundColor Green
Write-Host "  cargo run" -ForegroundColor Cyan
Write-Host "to start the development server." -ForegroundColor Green