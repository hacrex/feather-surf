#!/usr/bin/env pwsh
# build-macos.ps1 - Build FeatherSurf for macOS
# Run this on macOS with Xcode installed.
# Prerequisites: rustup target add aarch64-apple-darwin x86_64-apple-darwin

param(
    [switch]$Release,
    [string]$Target = "aarch64-apple-darwin"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RustDir = Join-Path $ScriptDir "..\rust"
$OutputDir = Join-Path $ScriptDir "..\dist\macos"

Write-Host "=== FeatherSurf macOS Build ===" -ForegroundColor Cyan
Write-Host "Target: $Target"

rustup target add $Target 2>$null
$profile = if ($Release) { "--release" } else { "" }

Write-Host "`n[1/3] Building feathersurf-ffi..." -ForegroundColor Yellow
cargo build -p feathersurf-ffi --target $Target $profile
if ($LASTEXITCODE -ne 0) { Write-Error "FFI build failed"; exit 1 }

Write-Host "`n[2/3] Building feathersurf-pal..." -ForegroundColor Yellow
cargo build -p feathersurf-pal --target $Target $profile
if ($LASTEXITCODE -ne 0) { Write-Error "PAL build failed"; exit 1 }

Write-Host "`n[3/3] Copying artifacts..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$targetDir = if ($Release) { "release" } else { "debug" }
$srcPath = Join-Path $RustDir "target\$Target\$targetDir\libfeathersurf_ffi.dylib"
if (Test-Path $srcPath) {
    Copy-Item $srcPath -Destination $OutputDir -Force
    Write-Host "  Copied libfeathersurf_ffi.dylib" -ForegroundColor Green
}

Write-Host "`n=== Build Complete ===" -ForegroundColor Cyan
