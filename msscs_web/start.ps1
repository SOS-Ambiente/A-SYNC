# MSSCS Web Startup Script

Write-Host "Starting MSSCS Web..." -ForegroundColor Cyan

# Check if Node.js is installed
if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Node.js is not installed!" -ForegroundColor Red
    Write-Host "Please install Node.js from https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}

# Start the server
Write-Host "Starting web server on http://localhost:8000" -ForegroundColor Green
node server.js
