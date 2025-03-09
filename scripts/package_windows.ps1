#!/usr/bin/env pwsh
# Script to package the Desmos MIDI application for Windows
# Creates a zip file with the binary, soundfonts folder, and config.toml

# Stop on first error
$ErrorActionPreference = "Stop"

# Check PowerShell version (PowerShell 5.1 and higher has reliable ZIP support)
$PSVersion = $PSVersionTable.PSVersion
if ($PSVersion.Major -lt 5 -or ($PSVersion.Major -eq 5 -and $PSVersion.Minor -lt 1)) {
    Write-Host "Warning: PowerShell version $($PSVersion.Major).$($PSVersion.Minor) detected."
    Write-Host "For best results, please update to PowerShell 5.1 or higher."
    Write-Host "Press any key to continue anyway, or CTRL+C to exit..."
    $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") | Out-Null
}

# Get system architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }

# Version from Cargo.toml (can be extracted programmatically for more automation)
$version = (Get-Content Cargo.toml | Select-String -Pattern 'version = "(.*)"').Matches.Groups[1].Value
$packageName = "desmos_midi_${version}_windows_${arch}"
$outputDir = "package"
$zipFileName = "${packageName}.zip"

Write-Host "Building release binaries..."
cargo build --release --features "cli,webui"

# Create output directory if it doesn't exist
if (!(Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir
}
$packageDir = Join-Path $outputDir $packageName
if (Test-Path $packageDir) {
    Remove-Item -Recurse -Force $packageDir
}
New-Item -ItemType Directory -Path $packageDir

# Copy binary files
Write-Host "Copying binary files..."
Copy-Item "target\release\desmos_midi.exe" -Destination $packageDir
Copy-Item "target\release\desmos_midi_web.exe" -Destination $packageDir

# Copy config and soundfonts
Write-Host "Copying configuration and soundfonts..."
Copy-Item "config.toml" -Destination $packageDir
Copy-Item -Recurse "soundfonts" -Destination $packageDir

# Copy README and LICENSE
Copy-Item "README.md" -Destination $packageDir
Copy-Item "LICENSE.txt" -Destination $packageDir

# Create zip file
Write-Host "Creating zip file..."
$zipPath = Join-Path $outputDir $zipFileName
if (Test-Path $zipPath) {
    Remove-Item -Force $zipPath
}

# Use Compress-Archive instead of ZipFile::CreateFromDirectory
# This preserves the directory structure better
Compress-Archive -Path $packageDir -DestinationPath $zipPath -Force

Write-Host "Package created at: $zipPath"