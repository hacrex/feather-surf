#!/usr/bin/env pwsh
# build-android.ps1 - Build FeatherSurf for Android (aarch64, armv7, x86_64)
#
# Prerequisites:
#   - Android NDK installed (via Android Studio or standalone)
#   - Set ANDROID_NDK_HOME environment variable
#   - Rust targets: rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
#
# Usage:
#   ./scripts/build-android.ps1 [-Release]

param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RustDir = Join-Path $ScriptDir "..\rust"
$OutputDir = Join-Path $ScriptDir "..\dist\android"

$targets = @(
    "aarch64-linux-android",
    "armv7-linux-androideabi",
    "x86_64-linux-android"
)

Write-Host "=== FeatherSurf Android Build ===" -ForegroundColor Cyan
Write-Host "Targets: $($targets -join ', ')"

# Check NDK
$ndkHome = $env:ANDROID_NDK_HOME
if (-not $ndkHome) {
    Write-Host "ANDROID_NDK_HOME not set. Attempting auto-detect..." -ForegroundColor Yellow
    $defaultPaths = @(
        "$env:LOCALAPPDATA\Android\Sdk\ndk\latest",
        "$env:USERPROFILE\AppData\Local\Android\Sdk\ndk\latest"
    )
    foreach ($path in $defaultPaths) {
        if (Test-Path $path) {
            $ndkHome = $path
            break
        }
    }
}

if ($ndkHome -and (Test-Path $ndkHome)) {
    Write-Host "NDK: $ndkHome" -ForegroundColor Green
} else {
    Write-Host "WARNING: Android NDK not found. Builds may fail." -ForegroundColor Yellow
}

# Install targets
Write-Host "`n[1/3] Installing Rust targets..." -ForegroundColor Yellow
foreach ($target in $targets) {
    rustup target add $target 2>$null
}

$profile = if ($Release) { "--release" } else { "" }

# Build for each target
Write-Host "`n[2/3] Building FFI library for each target..." -ForegroundColor Yellow
foreach ($target in $targets) {
    Write-Host "  Building for $target..." -ForegroundColor Cyan
    cargo build -p feathersurf-ffi --target $target $profile
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed for $target"
        exit 1
    }
}

# Copy artifacts
Write-Host "`n[3/3] Copying artifacts..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$targetDir = if ($Release) { "release" } else { "debug" }

foreach ($target in $targets) {
    $archDir = $target -replace '-', '_'
    $destDir = Join-Path $OutputDir $archDir
    New-Item -ItemType Directory -Path $destDir -Force | Out-Null

    $srcPath = Join-Path $RustDir "target\$target\$targetDir\libfeathersurf_ffi.so"
    if (Test-Path $srcPath) {
        Copy-Item $srcPath -Destination $destDir -Force
        Write-Host "  Copied to $archDir/" -ForegroundColor Green
    }
}

Write-Host "`n=== Build Complete ===" -ForegroundColor Cyan
Write-Host "Output: $OutputDir"
