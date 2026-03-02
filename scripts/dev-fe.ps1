# Start frontend only (assumes daemon is already running)
$ErrorActionPreference = "Stop"
$port = 27015

Write-Host "Checking if daemon is running..." -ForegroundColor Cyan

# Check if daemon port is available
$portCheck = netstat -ano | findstr ":$port.*LISTENING"
if (-not $portCheck) {
    Write-Host "Error: Daemon is not running on port $port" -ForegroundColor Red
    Write-Host "Please start the daemon first with: make dev-be" -ForegroundColor Yellow
    exit 1
}

Write-Host "Daemon is running on port $port" -ForegroundColor Green

# Start Vite dev server (frontend only)
Write-Host "Starting Vite dev server (frontend only)..." -ForegroundColor Cyan
$guiDir = Join-Path $PSScriptRoot "..\gui"
Set-Location $guiDir
npm run dev
