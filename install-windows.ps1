#!/usr/bin/env pwsh
#Requires -Version 5.1
<#
.SYNOPSIS
    Install fy CLI for Windows.
.DESCRIPTION
    Downloads the latest (or a specific version) fy Windows binary from GitHub
    Releases and installs it to %USERPROFILE%\.local\bin.
.PARAMETER Version
    Version tag to install (e.g. "0.1.3"). Defaults to "latest".
.PARAMETER InstallDir
    Directory to install fy.exe. Defaults to "%USERPROFILE%\.local\bin".
.EXAMPLE
    .\install-windows.ps1
    .\install-windows.ps1 -Version "0.1.3"
    .\install-windows.ps1 -InstallDir "C:\Tools"
#>
[CmdletBinding()]
param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:USERPROFILE\.local\bin"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$repo = "mars-base/fy"
$arch = "x86_64-pc-windows-msvc"

function Write-Info($msg) { Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "[OK]   $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "[WARN] $msg" -ForegroundColor Yellow }

# Determine version
if ($Version -eq "latest") {
    Write-Info "Resolving latest release..."
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" -UseBasicParsing
    $tag = $release.tag_name
    if (-not $tag) { throw "Could not determine latest release tag" }
    $Version = $tag.TrimStart('v')
}

$assetName = "fy-$Version-$arch.exe"
$downloadUrl = "https://github.com/$repo/releases/download/v$Version/$assetName"

Write-Info "Installing fy v$Version for Windows ($arch)..."
Write-Info "Download URL: $downloadUrl"
Write-Info "Install directory: $InstallDir"

# Create install directory
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$tmpFile = Join-Path $env:TEMP $assetName
$outFile = Join-Path $InstallDir "fy.exe"

# Download
Invoke-WebRequest -Uri $downloadUrl -OutFile $tmpFile -UseBasicParsing
Write-Success "Downloaded $assetName"

# Install
if (Test-Path $outFile) {
    Write-Warn "Existing fy.exe found, overwriting..."
}
Move-Item -Path $tmpFile -Destination $outFile -Force
Write-Success "Installed to $outFile"

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    Write-Success "Added $InstallDir to user PATH"
    # Refresh PATH in the current session so the user can run fy immediately
    $env:Path = "$env:Path;$InstallDir"
} else {
    Write-Info "$InstallDir already in user PATH"
}

# Verify
$installedVersion = & $outFile "--help" 2>&1 | Select-Object -First 1
Write-Success "fy installed: $installedVersion"
Write-Host ""
Write-Host "Try it now: fy en 你好世界" -ForegroundColor Cyan
