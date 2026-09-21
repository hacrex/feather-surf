#!/usr/bin/env pwsh
# build-linux.ps1 - Build FeatherSurf for Linux x64
#
# Prerequisites:
#   - Rust toolchain: rustup target add x86_64-unknown-linux-gnu
#   - Cross-compilation toolchain (gcc-multilib or cross)
#   - CEF SDK for Linux (downloaded separately)
#
# Usage:
#   ./scripts/build-linux.ps1 [-Release] [-Target x86_64-unknown-linux-gnu]

param(
    [switch]$Release,
    [string]$Target = "x86_64-unknown-linux-gnu"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RustDir = Join-Path $ScriptDir "..\rust"
$OutputDir = Join-Path $ScriptDir "..\dist\linux"

Write-Host "=== FeatherSurf Linux Build ===" -ForegroundColor Cyan
Write-Host "Target: $Target"
Write-Host "Mode: $(if ($Release) { 'Release' } else { 'Debug' })"

# Check if cross is available
Write-Host "`n[1/4] Checking cross-compilation tools..." -ForegroundColor Yellow
$hasCross = Get-Command cross -ErrorAction SilentlyContinue
$hasCargoBuild = Get-Command cargo -ErrorAction SilentlyContinue

if ($hasCross) {
    Write-Host "Using 'cross' for cross-compilation" -ForegroundColor Green
} else {
    Write-Host "Using cargo (ensure target $Target is installed)" -ForegroundColor Yellow
    rustup target add $Target 2>$null
}

# Build the FFI library
Write-Host "`n[2/4] Building feathersurf-ffi..." -ForegroundColor Yellow
$profile = if ($Release) { "--release" } else { "" }

if ($hasCross) {
    cross build -p feathersurf-ffi --target $Target $profile
} else {
    cargo build -p feathersurf-ffi --target $Target $profile
}
if ($LASTEXITCODE -ne 0) {
    Write-Error "FFI build failed"
    exit 1
}

# Build the PAL
Write-Host "`n[3/4] Building feathersurf-pal..." -ForegroundColor Yellow
if ($hasCross) {
    cross build -p feathersurf-pal --target $Target $profile
} else {
    cargo build -p feathersurf-pal --target $Target $profile
}
if ($LASTEXITCODE -ne 0) {
    Write-Error "PAL build failed"
    exit 1
}

# Copy artifacts to dist
Write-Host "`n[4/4] Copying artifacts..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$targetDir = if ($Release) { "release" } else { "debug" }
$artifacts = @(
    @{ Src = "libfeathersurf_ffi.so"; Dst = "libfeathersurf_ffi.so" },
    @{ Src = "libfeathersurf_ffi.a"; Dst = "libfeathersurf_ffi.a" }
)

foreach ($artifact in $artifacts) {
    $srcPath = Join-Path $RustDir "target\$Target\$targetDir\$($artifact.Src)"
    if (Test-Path $srcPath) {
        Copy-Item $srcPath -Destination $OutputDir -Force
        Write-Host "  Copied $($artifact.Dst)" -ForegroundColor Green
    }
}

Write-Host "`n=== Build Complete ===" -ForegroundColor Cyan
Write-Host "Output: $OutputDir"
Get-ChildItem $OutputDir | Format-Table Name, Length
