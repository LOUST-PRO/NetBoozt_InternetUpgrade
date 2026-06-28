# uninstall-windows-service.ps1 — Remove the NetBoozt DNS failover Windows Service.
#
# Usage: .\uninstall-windows-service.ps1
# Requires: Administrator privileges.
#
# By LOUST (www.loust.pro)

param(
    [switch]$Force
)

$ErrorActionPreference = 'Stop'

Write-Host "==> NetBoozt DNS Failover — Windows Service uninstaller" -ForegroundColor Cyan

$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "This script requires Administrator privileges. Run as Admin."
    exit 1
}

$serviceName = "netboozt-dns"

# Check if service exists
$svc = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
if (-not $svc) {
    Write-Host "Service '$serviceName' not found. Nothing to uninstall." -ForegroundColor Yellow
    exit 0
}

# Stop
if ($svc.Status -eq 'Running') {
    Write-Host "Stopping service..."
    Stop-Service -Name $serviceName -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}

# Delete
Write-Host "Deleting service..."
$deleteResult = sc.exe delete $serviceName
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to delete service: $deleteResult"
    exit 1
}

Write-Host ""
Write-Host "✅ Service '$serviceName' uninstalled." -ForegroundColor Green
Write-Host ""
Write-Host "Note: The netboozt-service.exe binary was not modified."
