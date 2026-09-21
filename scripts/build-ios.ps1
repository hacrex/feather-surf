#!/usr/bin/env pwsh
# build-ios.ps1 - Build FeatherSurf for iOS (aarch64)
#
# Prerequisites:
#   - macOS with Xcode installed
#   - Rust targets: rustup target add aarch64-apple-ios
#   - Must be run on macOS (iOS builds cannot cross-compile from Windows)
#
# Usage (on macOS):
#   ./scripts/build-ios.ps1 [-Release] [-Simulator]

param(
    [switch]$Release,
    [switch]$Simulator
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RustDir = Join-Path $ScriptDir "..\rust"
$OutputDir = Join-Path $ScriptDir "..\dist\ios"

$target = if ($Simulator) { "aarch64-apple-ios-sim" } else { "aarch64-apple-ios" }

Write-Host "=== FeatherSurf iOS Build ===" -ForegroundColor Cyan
Write-Host "Target: $target"
Write-Host "Simulator: $Simulator"

# Check if running on macOS
$isMacOS = $PSVersionTable.OS -match "Darwin"
if (-not $isMacOS) {
    Write-Error "iOS builds must be run on macOS with Xcode installed."
    exit 1
}

# Install target
Write-Host "`n[1/3] Installing Rust target..." -ForegroundColor Yellow
rustup target add $target

$profile = if ($Release) { "--release" } else { "" }

# Build the FFI library as static
Write-Host "`n[2/3] Building feathersurf-ffi (static)..." -ForegroundColor Yellow
cargo build -p feathersurf-ffi --target $target $profile
if ($LASTEXITCODE -ne 0) {
    Write-Error "FFI build failed"
    exit 1
}

# Copy artifacts
Write-Host "`n[3/3] Copying artifacts..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$targetDir = if ($Release) { "release" } else { "debug" }

$srcPath = Join-Path $RustDir "target\$target\$targetDir\libfeathersurf_ffi.a"
if (Test-Path $srcPath) {
    Copy-Item $srcPath -Destination $OutputDir -Force
    Write-Host "  Copied libfeathersurf_ffi.a" -ForegroundColor Green
}

Write-Host "`n=== Build Complete ===" -ForegroundColor Cyan
Write-Host "Output: $OutputDir"
