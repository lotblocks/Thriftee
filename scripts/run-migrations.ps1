# Database migration script for Raffle Platform (PowerShell version)
# This script provides a convenient way to run database migrations on Windows

param(
    [Parameter(Position=0)]
    [string]$Command = "up",
    
    [Parameter(Position=1)]
    [int]$Steps = 1
)

# Colors for output
$Red = [System.ConsoleColor]::Red
$Green = [System.ConsoleColor]::Green
$Yellow = [System.ConsoleColor]::Yellow
$Blue = [System.ConsoleColor]::Blue
$White = [System.ConsoleColor]::White

function Write-ColorOutput {
    param(
        [string]$Message,
        [System.ConsoleColor]$Color = $White
    )
    $originalColor = $Host.UI.RawUI.ForegroundColor
    $Host.UI.RawUI.ForegroundColor = $Color
    Write-Output $Message
    $Host.UI.RawUI.ForegroundColor = $originalColor
}

function Log-Info {
    param([string]$Message)
    Write-ColorOutput "[INFO] $Message" $Blue
}

function Log-Success {
    param([string]$Message)
    Write-ColorOutput "[SUCCESS] $Message" $Green
}

function Log-Warning {
    param([string]$Message)
    Write-ColorOutput "[WARNING] $Message" $Yellow
}

function Log-Error {
    param([string]$Message)
    Write-ColorOutput "[ERROR] $Message" $Red
}

# Check if we're in the correct directory
if (-not (Test-Path "backend/Cargo.toml")) {
    Log-Error "This script must be run from the project root directory"
    exit 1
}

# Check if .env file exists
if (-not (Test-Path ".env")) {
    Log-Error ".env file not found. Please create one with DATABASE_URL"
    exit 1
}

# Load environment variables from .env file
Get-Content ".env" | ForEach-Object {
    if ($_ -match "^([^#][^=]+)=(.*)$") {
        [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
}

# Check if DATABASE_URL is set
$databaseUrl = [Environment]::GetEnvironmentVariable("DATABASE_URL")
if (-not $databaseUrl) {
    Log-Error "DATABASE_URL environment variable is not set"
    exit 1
}

# Change to backend directory
Push-Location "backend"

try {
    switch ($Command.ToLower()) {
        "up" {
            Log-Info "Running database migrations..."
            cargo run --bin migrate up
            if ($LASTEXITCODE -eq 0) {
                Log-Success "Database migrations completed successfully"
            } else {
                Log-Error "Migration failed"
                exit 1
            }
        }
        "down" {
            Log-Warning "Rolling back $Steps migration(s)..."
            cargo run --bin migrate down $Steps
            if ($LASTEXITCODE -eq 0) {
                Log-Success "Database rollback completed successfully"
            } else {
                Log-Error "Rollback failed"
                exit 1
            }
        }
        "status" {
            Log-Info "Checking migration status..."
            cargo run --bin migrate status
        }
        "reset" {
            Log-Warning "This will reset the entire database and delete all data!"
            $confirmation = Read-Host "Are you sure you want to continue? (y/N)"
            if ($confirmation -eq "y" -or $confirmation -eq "Y") {
                Log-Info "Resetting database..."
                cargo run --bin migrate reset
                if ($LASTEXITCODE -eq 0) {
                    Log-Success "Database reset completed"
                } else {
                    Log-Error "Database reset failed"
                    exit 1
                }
            } else {
                Log-Info "Database reset cancelled"
            }
        }
        "fresh" {
            Log-Info "Running fresh migration (reset + up)..."
            Log-Warning "This will delete all existing data!"
            $confirmation = Read-Host "Are you sure you want to continue? (y/N)"
            if ($confirmation -eq "y" -or $confirmation -eq "Y") {
                cargo run --bin migrate reset
                if ($LASTEXITCODE -eq 0) {
                    cargo run --bin migrate up
                    if ($LASTEXITCODE -eq 0) {
                        Log-Success "Fresh migration completed successfully"
                    } else {
                        Log-Error "Fresh migration failed during up phase"
                        exit 1
                    }
                } else {
                    Log-Error "Fresh migration failed during reset phase"
                    exit 1
                }
            } else {
                Log-Info "Fresh migration cancelled"
            }
        }
        { $_ -in @("help", "-h", "--help") } {
            Write-Output "Database Migration Tool"
            Write-Output "======================="
            Write-Output ""
            Write-Output "Usage: .\scripts\run-migrations.ps1 [command] [options]"
            Write-Output ""
            Write-Output "Commands:"
            Write-Output "  up              Run all pending migrations (default)"
            Write-Output "  down [steps]    Rollback migrations (default: 1 step)"
            Write-Output "  status          Show migration status"
            Write-Output "  reset           Drop all tables and reset database"
            Write-Output "  fresh           Reset database and run all migrations"
            Write-Output "  help            Show this help message"
            Write-Output ""
            Write-Output "Examples:"
            Write-Output "  .\scripts\run-migrations.ps1                    # Run all pending migrations"
            Write-Output "  .\scripts\run-migrations.ps1 up                 # Run all pending migrations"
            Write-Output "  .\scripts\run-migrations.ps1 down               # Rollback 1 migration"
            Write-Output "  .\scripts\run-migrations.ps1 down 3             # Rollback 3 migrations"
            Write-Output "  .\scripts\run-migrations.ps1 status             # Show migration status"
            Write-Output "  .\scripts\run-migrations.ps1 reset              # Reset entire database"
            Write-Output "  .\scripts\run-migrations.ps1 fresh              # Reset and run all migrations"
        }
        default {
            Log-Error "Unknown command: $Command"
            Write-Output "Use '.\scripts\run-migrations.ps1 help' to see available commands"
            exit 1
        }
    }
} finally {
    Pop-Location
}