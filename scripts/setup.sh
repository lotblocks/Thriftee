#!/bin/bash

# Unit Shopping Platform Setup Script for Unix/Linux/macOS

echo "Setting up Unit Shopping Platform..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Please install Rust first:"
    echo "1. Visit https://rustup.rs/"
    echo "2. Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "3. Restart your terminal and run this script again"
    exit 1
fi

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Docker is not installed. Please install Docker:"
    echo "1. Visit https://docs.docker.com/get-docker/"
    echo "2. Follow the installation instructions for your OS"
    echo "3. Restart your terminal and run this script again"
    exit 1
fi

# Install sqlx-cli if not present
echo "Installing sqlx-cli..."
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Start database services
echo "Starting database services..."
docker-compose up -d postgres redis

# Wait for services to be ready
echo "Waiting for services to be ready..."
sleep 10

# Run database migrations
echo "Running database migrations..."
cd backend
sqlx migrate run
cd ..

# Build the project
echo "Building the project..."
cargo build

echo "Setup complete! You can now run:"
echo "  cargo run"
echo "to start the development server."