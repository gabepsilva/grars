# Insight Reader Windows Uninstall Script
# PowerShell script for uninstalling Insight Reader on Windows

#Requires -Version 5.1

param(
    [switch]$Yes,  # Auto-accept all prompts (non-interactive mode)
    [switch]$Help  # Show help message
)

$ErrorActionPreference = "Stop"

# Configuration
$AppName = "insight-reader"

# Installation directories (Windows standard locations)
$InstallDir = Join-Path $env:LOCALAPPDATA $AppName
$BinDir = Join-Path $InstallDir "bin"
$VenvDir = Join-Path $InstallDir "venv"
$ModelsDir = Join-Path $InstallDir "models"
$LogDir = Join-Path $InstallDir "logs"
$InsightReaderBin = Join-Path $BinDir "insight-reader.exe"

# Config directory (APPDATA, not LOCALAPPDATA)
$ConfigDir = Join-Path $env:APPDATA $AppName
$ConfigFile = Join-Path $ConfigDir "config.json"

# Shortcut paths
$StartMenuDir = Join-Path ([Environment]::GetFolderPath("StartMenu")) "Programs"
$DesktopDir = [Environment]::GetFolderPath("Desktop")
$StartMenuShortcut = Join-Path $StartMenuDir "Insight Reader.lnk"
$DesktopShortcut = Join-Path $DesktopDir "Insight Reader.lnk"

# Colors for output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Type = "INFO"
    )
    switch ($Type) {
        "INFO"    { Write-Host "[INFO] $Message" -ForegroundColor Blue }
        "SUCCESS" { Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
        "WARN"    { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
        "ERROR"   { Write-Host "[ERROR] $Message" -ForegroundColor Red }
    }
}

# Show help
if ($Help) {
    Write-Host ""
    Write-Host "Usage: .\install\uninstall-windows.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Yes, -y     Skip confirmation prompt (for non-interactive use)"
    Write-Host "  -Help, -h    Show this help message"
    Write-Host ""
    Write-Host "This script will remove:"
    Write-Host "  - insight-reader binary from $BinDir"
    Write-Host "  - Python virtual environment from $VenvDir"
    Write-Host "  - Voice models from $ModelsDir"
    Write-Host "  - Log files from $LogDir"
    Write-Host "  - Config file from $ConfigFile"
    Write-Host "  - Start Menu shortcut (if created)"
    Write-Host "  - Desktop shortcut (if created)"
    Write-Host "  - insight-reader from PATH (if added)"
    Write-Host "  - Installation directory: $InstallDir"
    Write-Host ""
    exit 0
}

# Main uninstall function
function Main {
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  Insight Reader Uninstall Script" -ForegroundColor Cyan
    Write-Host "  Windows Edition" -ForegroundColor Cyan
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host ""
    
    # Collect items to remove
    $itemsToRemove = @()
    
    if (Test-Path $InsightReaderBin) {
        $itemsToRemove += "Binary: $InsightReaderBin"
    }
    
    if (Test-Path $VenvDir) {
        $itemsToRemove += "Python venv: $VenvDir"
    }
    
    if (Test-Path $ModelsDir) {
        $itemsToRemove += "Models directory: $ModelsDir"
    }
    
    if (Test-Path $LogDir) {
        $itemsToRemove += "Log directory: $LogDir"
    }
    
    if (Test-Path $ConfigFile) {
        $itemsToRemove += "Config file: $ConfigFile"
    }
    
    if (Test-Path $StartMenuShortcut) {
        $itemsToRemove += "Start Menu shortcut: $StartMenuShortcut"
    }
    
    if (Test-Path $DesktopShortcut) {
        $itemsToRemove += "Desktop shortcut: $DesktopShortcut"
    }
    
    # Check if bin directory is in PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -like "*$BinDir*") {
        $itemsToRemove += "PATH entry: $BinDir"
    }
    
    # Add installation directory to removal list (will be fully removed)
    if (Test-Path $InstallDir) {
        $itemsToRemove += "Installation directory: $InstallDir"
    }
    
    # If nothing to remove, exit
    if ($itemsToRemove.Count -eq 0) {
        Write-ColorOutput "No insight-reader installation found to remove." "WARN"
        Write-ColorOutput "Checked locations:" "INFO"
        Write-Host "  - $InstallDir"
        Write-Host "  - $ConfigDir"
        exit 0
    }
    
    # Show what will be removed
    Write-ColorOutput "The following will be removed:" "INFO"
    foreach ($item in $itemsToRemove) {
        Write-Host "  - $item"
    }
    
    Write-Host ""
    
    # Confirm removal
    if (-not $Yes) {
        $response = Read-Host "Continue with removal? (y/N)"
        if ($response -notmatch "^[Yy]$") {
            Write-ColorOutput "Cancelled" "INFO"
            exit 0
        }
    } else {
        Write-ColorOutput "Auto-confirming removal (-Yes flag)" "INFO"
    }
    
    Write-Host ""
    
    # Remove binary
    if (Test-Path $InsightReaderBin) {
        Write-ColorOutput "Removing binary: $InsightReaderBin" "INFO"
        try {
            Remove-Item -Path $InsightReaderBin -Force -ErrorAction Stop
            Write-ColorOutput "Removed binary" "SUCCESS"
        } catch {
            Write-ColorOutput "Failed to remove binary: $_" "ERROR"
        }
    }
    
    # Remove bin directory if empty
    if (Test-Path $BinDir) {
        try {
            $binContents = Get-ChildItem -Path $BinDir -ErrorAction SilentlyContinue
            if ($null -eq $binContents -or $binContents.Count -eq 0) {
                Write-ColorOutput "Removing empty bin directory: $BinDir" "INFO"
                Remove-Item -Path $BinDir -Force -ErrorAction Stop
                Write-ColorOutput "Removed bin directory" "SUCCESS"
            }
        } catch {
            Write-ColorOutput "Failed to remove bin directory: $_" "WARN"
        }
    }
    
    # Remove Python venv
    if (Test-Path $VenvDir) {
        Write-ColorOutput "Removing Python venv: $VenvDir" "INFO"
        try {
            Remove-Item -Path $VenvDir -Recurse -Force -ErrorAction Stop
            Write-ColorOutput "Removed Python venv" "SUCCESS"
        } catch {
            Write-ColorOutput "Failed to remove venv: $_" "ERROR"
        }
    }
    
    # Remove models directory
    if (Test-Path $ModelsDir) {
        Write-ColorOutput "Removing models directory: $ModelsDir" "INFO"
        try {
            Remove-Item -Path $ModelsDir -Recurse -Force -ErrorAction Stop
            Write-ColorOutput "Removed models directory" "SUCCESS"
        } catch {
            Write-ColorOutput "Failed to remove models directory: $_" "ERROR"
        }
    }
    
    # Remove log directory
    if (Test-Path $LogDir) {
        Write-ColorOutput "Removing log directory: $LogDir" "INFO"
        try {
            Remove-Item -Path $LogDir -Recurse -Force -ErrorAction Stop
            Write-ColorOutput "Removed log directory" "SUCCESS"
        } catch {
            Write-ColorOutput "Failed to remove log directory: $_" "ERROR"
        }
    }
    
    # Remove config file
    if (Test-Path $ConfigFile) {
        Write-ColorOutput "Removing config file: $ConfigFile" "INFO"
        try {
            Remove-Item -Path $ConfigFile -Force -ErrorAction Stop
            Write-ColorOutput "Removed config file" "SUCCESS"
        } catch {
            Write-ColorOutput "Failed to remove config file: $_" "ERROR"
        }
    }
    
    # Remove config directory if empty
    if (Test-Path $ConfigDir) {
        try {
            $configContents = Get-ChildItem -Path $ConfigDir -ErrorAction SilentlyContinue
            if ($null -eq $configContents -or $configContents.Count -eq 0) {
                Write-ColorOutput "Removing empty config directory: $ConfigDir" "INFO"
                Remove-Item -Path $ConfigDir -Force -ErrorAction Stop
                Write-ColorOutput "Removed config directory" "SUCCESS"
            }
        } catch {
            Write-ColorOutput "Failed to remove config directory: $_" "WARN"
        }
    }
    
    # Remove Start Menu shortcut
    if (Test-Path $StartMenuShortcut) {
        Write-ColorOutput "Removing Start Menu shortcut: $StartMenuShortcut" "INFO"
        try {
            Remove-Item -Path $StartMenuShortcut -Force -ErrorAction Stop
            Write-ColorOutput "Removed Start Menu shortcut" "SUCCESS"
        } catch {
            Write-ColorOutput "Failed to remove Start Menu shortcut: $_" "ERROR"
        }
    }
    
    # Remove Desktop shortcut
    if (Test-Path $DesktopShortcut) {
        Write-ColorOutput "Removing Desktop shortcut: $DesktopShortcut" "INFO"
        try {
            Remove-Item -Path $DesktopShortcut -Force -ErrorAction Stop
            Write-ColorOutput "Removed Desktop shortcut" "SUCCESS"
        } catch {
            Write-ColorOutput "Failed to remove Desktop shortcut: $_" "ERROR"
        }
    }
    
    # Remove from PATH (auto-remove, no prompt)
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -like "*$BinDir*") {
        Write-ColorOutput "Removing insight-reader from PATH..." "INFO"
        $pathEntries = $userPath -split ';' | Where-Object { $_ -ne $BinDir -and $_ -ne "" }
        $newPath = $pathEntries -join ';'
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-ColorOutput "Removed from PATH. Please restart your terminal for changes to take effect." "SUCCESS"
    }
    
    # Remove main installation directory (fully remove, not just when empty)
    if (Test-Path $InstallDir) {
        Write-ColorOutput "Removing installation directory: $InstallDir" "INFO"
        try {
            Remove-Item -Path $InstallDir -Recurse -Force -ErrorAction Stop
            Write-ColorOutput "Removed installation directory" "SUCCESS"
        } catch {
            Write-ColorOutput "Failed to remove installation directory: $_" "WARN"
        }
    }
    
    Write-Host ""
    Write-ColorOutput "Uninstall complete!" "SUCCESS"
    Write-Host ""
    Write-ColorOutput "You can now run .\install\install-windows.ps1 to reinstall." "INFO"
    Write-Host ""
}

# Run main function
Main
