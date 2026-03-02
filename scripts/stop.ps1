# Stop daemon and dev processes
$port = 27015
$maxAttempts = 5

function Stop-ProcessAndWait {
    param([int]$ProcId, [string]$Name)
    try {
        $proc = Get-Process -Id $ProcId -ErrorAction Stop
        Write-Host "Stopping $Name PID $ProcId..."
        Stop-Process -Id $ProcId -Force -ErrorAction SilentlyContinue

        # Wait for process to actually terminate
        for ($i = 0; $i -lt $maxAttempts; $i++) {
            Start-Sleep -Milliseconds 200
            $proc = Get-Process -Id $ProcId -ErrorAction SilentlyContinue
            if (-not $proc) {
                Write-Host "Stopped $Name PID $ProcId"
                return $true
            }
        }
        Write-Host "Warning: $Name PID $ProcId may not have terminated cleanly"
        return $false
    } catch {
        return $false
    }
}

# Kill by process name first (catches cargo and labalaba processes)
$anyStopped = $false
Get-Process | Where-Object {$_.ProcessName -like "*cargo*" -or $_.ProcessName -like "*labalaba*"} | ForEach-Object {
    if (Stop-ProcessAndWait -ProcId $_.Id -Name $_.ProcessName) {
        $anyStopped = $true
    }
}

# Check port for any remaining process
Start-Sleep -Milliseconds 300
$output = netstat -ano | Select-String ":$port\s+.*LISTENING"
if ($output) {
    foreach ($line in $output) {
        $parts = $line -split '\s+'
        $pidNum = $parts[-1]
        if ($pidNum -match '^\d+$' -and $pidNum -ne '0') {
            $procObj = Get-Process -Id $pidNum -ErrorAction SilentlyContinue
            if ($procObj) {
                Stop-ProcessAndWait -ProcId $pidNum -Name "daemon"
                $anyStopped = $true
            }
        }
    }
}

if (-not $anyStopped) {
    Write-Host "No daemon or cargo processes found"
}

Write-Host "Done"
