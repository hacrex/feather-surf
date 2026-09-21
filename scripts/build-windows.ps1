#!/usr/bin/env pwsh
# build-windows.ps1 - Build FeatherSurf for Windows x64
#
# Prerequisites:
#   - Rust toolchain: rustup target add x86_64-pc-windows-msvc
#   - Visual Studio Build Tools (MSVC)
#   - CEF SDK (downloaded separately)
#
# Usage:
#   ./scripts/build-windows.ps1 [-Release] [-Target x86_64-pc-windows-msvc]

param(
    [switch]$Release,
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RustDir = Join-Path $ScriptDir "..\rust"
$OutputDir = Join-Path $ScriptDir "..\dist\windows"

Write-Host "=== FeatherSurf Windows Build ===" -ForegroundColor Cyan
Write-Host "Target: $Target"
Write-Host "Mode: $(if ($Release) { 'Release' } else { 'Debug' })"

# Ensure target is installed
Write-Host "`n[1/4] Checking Rust target..." -ForegroundColor Yellow
rustup target list --installed | Select-String -Pattern $Target
if ($LASTEXITCODE -ne 0) {
    Write-Host "Installing target $Target..."
    rustup target add $Target
}

# Build the FFI library
Write-Host "`n[2/4] Building feathersurf-ffi..." -ForegroundColor Yellow
$profile = if ($Release) { "--release" } else { "" }
cargo build -p feathersurf-ffi --target $Target $profile
if ($LASTEXITCODE -ne 0) {
    Write-Error "FFI build failed"
    exit 1
}

# Build the PAL
Write-Host "`n[3/4] Building feathersurf-pal..." -ForegroundColor Yellow
cargo build -p feathersurf-pal --target $Target $profile
if ($LASTEXITCODE -ne 0) {
    Write-Error "PAL build failed"
    exit 1
}

# Copy artifacts to dist
Write-Host "`n[4/4] Copying artifacts..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$targetDir = if ($Release) { "release" } else { "debug" }
$artifacts = @(
    @{ Src = "feathersurf_ffi.dll"; Dst = "feathersurf_ffi.dll" },
    @{ Src = "feathersurf_ffi.lib"; Dst = "feathersurf_ffi.lib" },
    @{ Src = "feathersurf_ffi.pdb"; Dst = "feathersurf_ffi.pdb" }
)

foreach ($artifact in $artifacts) {
    $srcPath = Join-Path $RustDir "target\$Target\$targetDir\$($artifact.Src)"
    if (Test-Path $srcPath) {
        Copy-Item $srcPath -Destination $OutputDir -Force
        Write-Host "  Copied $($artifact.Dst)" -ForegroundColor Green
    }
}

# Also copy headers
$ffiHeaders = Join-Path $RustDir "ffi\headers"
if (Test-Path $ffiHeaders) {
    Copy-Item $ffiHeaders -Destination $OutputDir -Recurse -Force
    Write-Host "  Copied headers/" -ForegroundColor Green
}

Write-Host "`n=== Build Complete ===" -ForegroundColor Cyan
Write-Host "Output: $OutputDir"
Get-ChildItem $OutputDir | Format-Table Name, Length
