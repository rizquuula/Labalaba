# Start backend daemon only
$ErrorActionPreference = "Stop"
$port = 27015

Write-Host "Starting backend (daemon)..." -ForegroundColor Cyan

# Check if port is already in use
$portCheck = netstat -ano | findstr ":$port.*LISTENING"
if ($portCheck) {
    Write-Host "Port $port is already in use. Running stop script first..." -ForegroundColor Yellow
    & powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\stop.ps1"
    Start-Sleep -Seconds 2
}

# Start daemon in a new terminal window
Start-Process -FilePath "cmd" -ArgumentList "/c cargo run -p labalaba-daemon" -WindowStyle Normal

# Wait for daemon to be ready
$maxWait = 30
$ready = $false
for ($i = 0; $i -lt $maxWait; $i++) {
    Start-Sleep -Milliseconds 500
    $portCheck = netstat -ano | findstr ":$port.*LISTENING"
    if ($portCheck) {
        Write-Host "Daemon is ready on port $port" -ForegroundColor Green
        $ready = $true
        break
    }
    Write-Host "Waiting for daemon... ($i/$maxWait)"
}

if (-not $ready) {
    Write-Host "Warning: Daemon may not have started correctly" -ForegroundColor Yellow
}

Write-Host "Backend started. Press Ctrl+C to stop." -ForegroundColor Green
while ($true) { Start-Sleep -Seconds 1 }
