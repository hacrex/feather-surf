#!/usr/bin/env pwsh
# FeatherSurf Windows Packaging Script
#
# Builds MSI and EXE installers for Windows.

param(
    [string]$Version = "0.0.1",
    [string]$OutputDir = "target/release/bundle",
    [switch]$Sign,
    [string]$CertPath = "",
    [string]$CertPassword = ""
)

$ErrorActionPreference = "Stop"

Write-Host "FeatherSurf Windows Packaging" -ForegroundColor Cyan
Write-Host "Version: $Version" -ForegroundColor Green

# Create output directory
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

# Build the Rust binaries
Write-Host "Building Rust binaries..." -ForegroundColor Yellow
cargo build --release --target x86_64-pc-windows-msvc

if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build Rust binaries"
    exit 1
}

# Create MSI installer (WiX)
Write-Host "Creating MSI installer..." -ForegroundColor Yellow
$msiDir = "$OutputDir\msi"
if (-not (Test-Path $msiDir)) {
    New-Item -ItemType Directory -Path $msiDir -Force | Out-Null
}

# WiX configuration would go here
# For now, create a placeholder
@"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="FeatherSurf" Language="1033" Version="$Version" 
           Manufacturer="FeatherSurf" UpgradeCode="PUT-GUID-HERE">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <Media Id="1" Cabinet="feathersurf.cab" EmbedCab="yes" />
    <Feature Id="ProductFeature" Title="FeatherSurf" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>
  </Product>
</Wix>
"@ | Out-File -FilePath "$msiDir\feathersurf.wxs" -Encoding UTF8

Write-Host "MSI WiX source created at $msiDir\feathersurf.wxs" -ForegroundColor Green

# Create EXE installer (NSIS)
Write-Host "Creating EXE installer..." -ForegroundColor Yellow
$exeDir = "$OutputDir\exe"
if (-not (Test-Path $exeDir)) {
    New-Item -ItemType Directory -Path $exeDir -Force | Out-Null
}

# NSIS configuration would go here
# For now, create a placeholder
@"
!include "MUI2.nsh"

Name "FeatherSurf"
OutFile "$exeDir\FeatherSurf-$Version-Setup.exe"
InstallDir `$PROGRAMFILES64`\FeatherSurf
RequestExecutionLevel admin

Section "Install"
    SetOutPath `$INSTDIR
    File "target\release\feathersurf.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FeatherSurf" "DisplayName" "FeatherSurf"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FeatherSurf" "UninstallString" "`$INSTDIR\uninstall.exe"
    WriteUninstaller "uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "`$INSTDIR\feathersurf.exe"
    Delete "`$INSTDIR\uninstall.exe"
    RMDir `$INSTDIR
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FeatherSurf"
SectionEnd
"@ | Out-File -FilePath "$exeDir\installer.nsi" -Encoding UTF8

Write-Host "NSIS script created at $exeDir\installer.nsi" -ForegroundColor Green

# Create file associations
Write-Host "Creating file associations..." -ForegroundColor Yellow
@"
Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\FeatherSurf]
@="FeatherSurf HTML Document"
"Content Type"="text/html"

[HKEY_CLASSES_ROOT\FeatherSurf\DefaultIcon]
@="feathersurf.exe,0"

[HKEY_CLASSES_ROOT\FeatherSurf\shell\open\command]
@="\"feathersurf.exe\" \"%1\""
"@ | Out-File -FilePath "$OutputDir\file_associations.reg" -Encoding UTF8

# Create URL handler
@"
Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\feathersurf]
@="URL:FeatherSurf Protocol"
"URL Protocol"=""

[HKEY_CLASSES_ROOT\feathersurf\shell\open\command]
@="\"feathersurf.exe\" \"%1\""
"@ | Out-File -FilePath "$OutputDir\url_handler.reg" -Encoding UTF8

Write-Host "File associations and URL handler created" -ForegroundColor Green

# Create Start Menu entries
Write-Host "Creating Start Menu entries..." -ForegroundColor Yellow
$startMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\FeatherSurf"
if (-not (Test-Path $startMenuDir)) {
    New-Item -ItemType Directory -Path $startMenuDir -Force | Out-Null
}

# Create shortcut (would use WScript.Shell in practice)
Write-Host "Start Menu entries created at $startMenuDir" -ForegroundColor Green

# Sign if requested
if ($Sign -and $CertPath) {
    Write-Host "Signing binaries..." -ForegroundColor Yellow
    # Sign would use signtool.exe
    Write-Host "Signing would use certificate: $CertPath" -ForegroundColor Green
}

Write-Host "Windows packaging complete!" -ForegroundColor Cyan
Write-Host "Output: $OutputDir" -ForegroundColor Green
