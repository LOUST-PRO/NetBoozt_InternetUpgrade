# install-windows-service.ps1 — Install NetBoozt DNS failover as a Windows Service.
#
# Usage: .\install-windows-service.ps1
# Requires: Administrator privileges, netboozt-service.exe built.
#
# By LOUST (www.loust.pro)

param(
    [string]$BinaryPath = "$PWD\target\release\netboozt-service.exe"
)

$ErrorActionPreference = 'Stop'

Write-Host "==> NetBoozt DNS Failover — Windows Service installer" -ForegroundColor Cyan

# Check for admin
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "This script requires Administrator privileges. Run as Admin."
    exit 1
}

# Find binary
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at: $BinaryPath"
    Write-Host "Build it first: cargo build --release --bin netboozt-service" -ForegroundColor Yellow
    exit 1
}

$serviceName = "netboozt-dns"

# Check if already installed
$existing = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "Service '$serviceName' already exists. Stopping and removing..." -ForegroundColor Yellow
    Stop-Service -Name $serviceName -Force -ErrorAction SilentlyContinue
    sc.exe delete $serviceName | Out-Null
    Start-Sleep -Seconds 2
}

# Create the service
Write-Host "Creating service '$serviceName'..."
$createResult = sc.exe create $serviceName binPath= "$BinaryPath" DisplayName= "NetBoozt DNS Failover" start= auto type= own
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to create service: $createResult"
    exit 1
}

# Set recovery options
Write-Host "Configuring recovery options..."
sc.exe failure $serviceName reset= 86400 actions= restart/10000/restart/10000/restart/10000 | Out-Null

# Start the service
Write-Host "Starting service..."
Start-Service -Name $serviceName -ErrorAction SilentlyContinue

# Verify
$svc = Get-Service -Name $serviceName
if ($svc.Status -eq 'Running') {
    Write-Host ""
    Write-Host "✅ Service '$serviceName' is running." -ForegroundColor Green
    Write-Host ""
    Write-Host "Useful commands:"
    Write-Host "  Get-Service $serviceName          # view status"
    Write-Host "  sc.exe query $serviceName         # detailed status"
    Write-Host "  Stop-Service $serviceName        # stop"
    Write-Host "  sc.exe delete $serviceName       # remove (use uninstall script instead)"
} else {
    Write-Host ""
    Write-Warning "Service started but may not be fully running yet. Status: $($svc.Status)"
    Write-Host "Check: Get-Service $serviceName"
}
