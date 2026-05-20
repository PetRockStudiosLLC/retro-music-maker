# Navigate to project root
Set-Location $PSScriptRoot

# Start frontend dev server first
Write-Host "Starting frontend dev server..." -ForegroundColor Green
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd frontend; npm run dev" -WorkingDirectory (Get-Location)

# Wait for frontend to be ready
Write-Host "Waiting for frontend on port 6004..." -ForegroundColor Yellow
do {
    Start-Sleep -Milliseconds 500
    $conn = New-Object System.Net.Sockets.TcpClient
    try {
        $conn.Connect("localhost", 6004)
        $frontendReady = $true
    } catch {
        $frontendReady = $false
    }
    $conn.Close()
} while (-not $frontendReady)
Write-Host "Frontend ready!" -ForegroundColor Green

# Start backend API server
Write-Host "Starting backend API server..." -ForegroundColor Green
Start-Process powershell -ArgumentList "-NoExit", "-Command", "python main.py api" -WorkingDirectory (Get-Location)

# Wait for backend to be ready
Write-Host "Waiting for backend on port 8000..." -ForegroundColor Yellow
do {
    Start-Sleep -Milliseconds 500
    $conn = New-Object System.Net.Sockets.TcpClient
    try {
        $conn.Connect("localhost", 8000)
        $backendReady = $true
    } catch {
        $backendReady = $false
    }
    $conn.Close()
} while (-not $backendReady)
Write-Host "Backend ready!" -ForegroundColor Green

Write-Host "All services started. Launching Tauri..." -ForegroundColor Green
