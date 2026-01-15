# Insight Reader Windows Installation Script
# PowerShell script for installing Insight Reader on Windows

#Requires -Version 5.1

param(
    [string]$ReleaseTag = "",
    [switch]$SkipPython,
    [switch]$SkipPiper,
    [switch]$SkipShortcuts,
    [switch]$Force,
    [switch]$Yes  # Auto-accept all prompts (non-interactive mode)
)

$ErrorActionPreference = "Stop"

# Configuration
$AppName = "insight-reader"
$GithubRepo = "gabepsilva/insight-reader"
$GithubApi = "https://api.github.com/repos/$GithubRepo"
$DefaultModelName = "en_US-lessac-medium"

# Installation directories (Windows standard locations)
$InstallDir = Join-Path $env:LOCALAPPDATA $AppName
$BinDir = Join-Path $InstallDir "bin"
$VenvDir = Join-Path $InstallDir "venv"
$ModelsDir = Join-Path $InstallDir "models"
$InsightReaderBin = Join-Path $BinDir "insight-reader.exe"

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

function Test-CommandExists {
    param([string]$Command)
    $null -ne (Get-Command $Command -ErrorAction SilentlyContinue)
}

function Get-PythonCommand {
    # Try different Python commands
    # Note: On Windows, 'python' may exist as an app execution alias that shows an error
    # We need to actually test if Python runs successfully
    
    try {
        $result = & python --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $result -match "Python 3\.") {
            return "python"
        }
    } catch {
        # Python not available or app alias
    }
    
    try {
        $result = & python3 --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $result -match "Python 3\.") {
            return "python3"
        }
    } catch {
        # python3 not available
    }
    
    try {
        # Windows Python launcher
        $result = & py -3 --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $result -match "Python 3\.") {
            return "py -3"
        }
    } catch {
        # py launcher not available
    }
    
    return $null
}

function Test-PythonInstalled {
    $pythonCmd = Get-PythonCommand
    if ($null -eq $pythonCmd) {
        return $false
    }
    try {
        $cmdParts = $pythonCmd -split ' '
        if ($cmdParts.Count -eq 2) {
            $version = & $cmdParts[0] $cmdParts[1] --version 2>&1
        } else {
            $version = & $pythonCmd --version 2>&1
        }
        Write-ColorOutput "Python found: $version" "SUCCESS"
    } catch {
        Write-ColorOutput "Python found: $pythonCmd" "SUCCESS"
    }
    return $true
}

function Install-PythonViaWinget {
    Write-ColorOutput "Attempting to install Python via winget..." "INFO"
    
    # Check if winget is available
    if (-not (Test-CommandExists "winget")) {
        Write-ColorOutput "winget not found. Please install Python manually." "WARN"
        return $false
    }
    
    try {
        # Install Python 3.12 via winget (silent install)
        # Use --source winget to avoid Microsoft Store certificate issues
        Write-ColorOutput "Installing Python 3.12 (this may take a minute)..." "INFO"
        $result = & winget install Python.Python.3.12 --source winget --silent --accept-package-agreements --accept-source-agreements 2>&1
        
        # Exit code -1978335189 (0x8A15002B) means "already installed"
        if ($LASTEXITCODE -eq 0 -or $LASTEXITCODE -eq -1978335189) {
            Write-ColorOutput "Python installed successfully via winget" "SUCCESS"
            
            # Refresh PATH for current session
            $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
            
            # Verify installation
            Start-Sleep -Seconds 2  # Give Windows a moment to register the new install
            if (Test-PythonInstalled) {
                return $true
            } else {
                Write-ColorOutput "Python installed but not yet in PATH. Please restart your terminal after installation completes." "WARN"
                return $true
            }
        } else {
            Write-ColorOutput "winget installation returned non-zero exit code" "WARN"
            return $false
        }
    } catch {
        Write-ColorOutput "Failed to install Python via winget: $_" "WARN"
        return $false
    }
}

function Install-Dependencies {
    Write-ColorOutput "Checking required dependencies..."
    
    $missingDeps = @()
    
    # Check Python
    if (-not (Test-PythonInstalled)) {
        $missingDeps += "Python 3"
    }
    
    if ($missingDeps.Count -gt 0) {
        Write-ColorOutput "Missing required dependencies:" "WARN"
        foreach ($dep in $missingDeps) {
            Write-Host "  - $dep"
        }
        Write-Host ""
        
        # Try to install Python automatically via winget
        $installPython = $Yes
        if (-not $Yes) {
            $response = Read-Host "Install Python 3 via winget? (Y/n)"
            $installPython = $response -notmatch "^[Nn]"
        }
        
        if ($installPython) {
            if (Install-PythonViaWinget) {
                # Re-check if Python is now available
                if (Test-PythonInstalled) {
                    Write-ColorOutput "All required dependencies are now installed" "SUCCESS"
                    return
                }
            }
        }
        
        # If we get here, Python still isn't available
        Write-ColorOutput "Python 3 is not installed" "WARN"
        Write-ColorOutput "You can install it manually from https://www.python.org/downloads/" "INFO"
        Write-ColorOutput "Or via Microsoft Store: 'python3' app" "INFO"
        Write-Host ""
        
        if ($Yes) {
            Write-ColorOutput "Continuing without Python (-Yes flag set)" "WARN"
        } else {
            $response = Read-Host "Continue anyway? (y/N)"
            if ($response -notmatch "^[Yy]") {
                Write-ColorOutput "Installation cancelled" "ERROR"
                exit 1
            }
        }
    } else {
        Write-ColorOutput "All required dependencies are installed" "SUCCESS"
    }
}

function Get-LatestRelease {
    Write-ColorOutput "Fetching latest release from GitHub..."
    
    try {
        $response = Invoke-RestMethod -Uri "$GithubApi/releases/latest" -Method Get
        return $response.tag_name
    } catch {
        Write-ColorOutput "Failed to fetch latest release, using 'latest'" "WARN"
        return "latest"
    }
}

function Install-Binary {
    Write-ColorOutput "Installing insight-reader binary..."
    
    # Create directories
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    
    # Check for local binary first (development)
    $localBinary = $null
    if (Test-Path "target\release\insight-reader.exe") {
        $localBinary = "target\release\insight-reader.exe"
    } elseif (Test-Path "insight-reader.exe") {
        $localBinary = "insight-reader.exe"
    }
    
    if ($localBinary) {
        Write-ColorOutput "Found local binary at $localBinary" "INFO"
        Copy-Item $localBinary $InsightReaderBin -Force
        Write-ColorOutput "Binary copied to $InsightReaderBin" "SUCCESS"
        return
    }
    
    # Download from GitHub
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
    $binaryName = "insight-reader-windows-$arch.exe"
    
    if ($ReleaseTag) {
        $downloadUrl = "https://github.com/$GithubRepo/releases/download/$ReleaseTag/$binaryName"
        Write-ColorOutput "Using release tag: $ReleaseTag" "INFO"
    } else {
        $downloadUrl = "https://github.com/$GithubRepo/releases/latest/download/$binaryName"
        Write-ColorOutput "Downloading latest release..." "INFO"
    }
    
    Write-ColorOutput "Downloading from $downloadUrl" "INFO"
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $InsightReaderBin
        Write-ColorOutput "Binary downloaded and installed to $InsightReaderBin" "SUCCESS"
    } catch {
        Write-ColorOutput "Failed to download binary: $_" "ERROR"
        Write-ColorOutput "Please build the binary first: cargo build --release" "INFO"
        exit 1
    }
}

function New-PythonVenv {
    Write-ColorOutput "Creating Python virtual environment at $VenvDir..."
    
    $pythonCmd = Get-PythonCommand
    if ($null -eq $pythonCmd) {
        Write-ColorOutput "Python not found, skipping venv creation" "WARN"
        return $false
    }
    
    # Remove existing venv if force flag is set
    if ($Force -and (Test-Path $VenvDir)) {
        Write-ColorOutput "Removing existing venv..." "INFO"
        Remove-Item -Recurse -Force $VenvDir
    }
    
    # Create parent directory
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    
    # Create venv
    $cmdParts = $pythonCmd -split ' '
    if ($cmdParts.Count -eq 2) {
        # Handle "py -3" case
        & $cmdParts[0] $cmdParts[1] -m venv $VenvDir
    } else {
        & $pythonCmd -m venv $VenvDir
    }
    
    if (-not (Test-Path (Join-Path $VenvDir "Scripts\activate.ps1"))) {
        Write-ColorOutput "Failed to create virtual environment" "ERROR"
        return $false
    }
    
    Write-ColorOutput "Virtual environment created" "SUCCESS"
    return $true
}

function Install-Piper {
    Write-ColorOutput "Installing piper-tts in virtual environment..."
    
    $activateScript = Join-Path $VenvDir "Scripts\Activate.ps1"
    if (-not (Test-Path $activateScript)) {
        Write-ColorOutput "Virtual environment not found, skipping piper installation" "WARN"
        return $false
    }
    
    # Use python -m pip instead of pip.exe directly for better compatibility
    $pythonExe = Join-Path $VenvDir "Scripts\python.exe"
    
    # Upgrade pip first
    Write-ColorOutput "Upgrading pip..." "INFO"
    & $pythonExe -m pip install --quiet --upgrade pip 2>$null
    
    # Clear pip cache
    Write-ColorOutput "Clearing pip cache..." "INFO"
    & $pythonExe -m pip cache purge 2>$null
    
    # Install onnxruntime first (required dependency for piper-tts)
    Write-ColorOutput "Installing onnxruntime (required dependency)..." "INFO"
    & $pythonExe -m pip install --quiet "onnxruntime<2,>=1" 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-ColorOutput "Standard onnxruntime installation failed, trying nightly build..." "WARN"
        & $pythonExe -m pip install --quiet --pre onnxruntime --extra-index-url=https://aiinfra.pkgs.visualstudio.com/PublicPackages/_packaging/ORT-Nightly/pypi/simple/ 2>$null
        if ($LASTEXITCODE -ne 0) {
            Write-ColorOutput "Failed to install onnxruntime" "ERROR"
            return $false
        }
        Write-ColorOutput "onnxruntime nightly build installed" "SUCCESS"
    } else {
        Write-ColorOutput "onnxruntime installed successfully" "SUCCESS"
    }
    
    # Install piper-tts
    Write-ColorOutput "Installing piper-tts package..." "INFO"
    & $pythonExe -m pip install --quiet --upgrade --force-reinstall piper-tts 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-ColorOutput "Standard installation failed, trying without dependency checks..." "WARN"
        & $pythonExe -m pip install --quiet --upgrade --force-reinstall --no-deps piper-tts 2>$null
        if ($LASTEXITCODE -ne 0) {
            Write-ColorOutput "Failed to install piper-tts" "ERROR"
            return $false
        }
        # Install other dependencies
        & $pythonExe -m pip install --quiet piper-phonemize 2>$null
    }
    
    # Verify installation
    $piperBin = Join-Path $VenvDir "Scripts\piper.exe"
    if (-not (Test-Path $piperBin)) {
        Write-ColorOutput "piper binary not found after installation" "ERROR"
        return $false
    }
    
    # Test piper
    try {
        & $piperBin --help 2>$null | Out-Null
        Write-ColorOutput "piper-tts installed successfully" "SUCCESS"
    } catch {
        Write-ColorOutput "piper binary found but doesn't respond to --help" "ERROR"
        return $false
    }
    
    # Note: Windows uses built-in Windows.Media.Ocr API for OCR functionality
    # No external dependencies (Python, EasyOCR, PyTorch) are required for OCR on Windows
    # The OCR engine automatically uses the user's profile languages
    
    return $true
}

# Install-OcrScript function removed - Windows uses built-in Windows.Media.Ocr API
# No Python script installation needed for Windows OCR functionality

function Install-Model {
    Write-ColorOutput "Checking for default voice model: $DefaultModelName..."
    
    $modelOnnx = Join-Path $ModelsDir "$DefaultModelName.onnx"
    $modelJson = Join-Path $ModelsDir "$DefaultModelName.onnx.json"
    
    # Check if model already exists
    if ((Test-Path $modelOnnx) -and (Test-Path $modelJson)) {
        Write-ColorOutput "Model already exists at $ModelsDir" "SUCCESS"
        return $true
    }
    
    Write-ColorOutput "Model not found. Downloading from HuggingFace..." "INFO"
    
    # Create models directory
    New-Item -ItemType Directory -Path $ModelsDir -Force | Out-Null
    
    $modelBaseUrl = "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium"
    
    try {
        Write-ColorOutput "Downloading $DefaultModelName.onnx..." "INFO"
        Invoke-WebRequest -Uri "$modelBaseUrl/$DefaultModelName.onnx" -OutFile $modelOnnx
        
        Write-ColorOutput "Downloading $DefaultModelName.onnx.json..." "INFO"
        Invoke-WebRequest -Uri "$modelBaseUrl/$DefaultModelName.onnx.json" -OutFile $modelJson
        
        Write-ColorOutput "Model downloaded successfully to $ModelsDir" "SUCCESS"
        return $true
    } catch {
        Write-ColorOutput "Failed to download model: $_" "WARN"
        Write-ColorOutput "You can download the model manually from:" "INFO"
        Write-ColorOutput "  $modelBaseUrl/$DefaultModelName.onnx" "INFO"
        Write-ColorOutput "  $modelBaseUrl/$DefaultModelName.onnx.json" "INFO"
        Write-ColorOutput "Place the files in: $ModelsDir" "INFO"
        return $false
    }
}

function Add-ToPath {
    Write-ColorOutput "Checking PATH environment variable..."
    
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    
    if ($userPath -notlike "*$BinDir*") {
        Write-ColorOutput "Adding $BinDir to user PATH..." "INFO"
        $newPath = "$userPath;$BinDir"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-ColorOutput "Added to PATH. Please restart your terminal for changes to take effect." "SUCCESS"
    } else {
        Write-ColorOutput "insight-reader is already in PATH" "SUCCESS"
    }
}

function Create-Shortcuts {
    Write-ColorOutput "Creating Start Menu and Desktop shortcuts..." "INFO"
    
    # Check if binary exists
    if (-not (Test-Path $InsightReaderBin)) {
        Write-ColorOutput "Binary not found, skipping shortcut creation" "WARN"
        return $false
    }
    
    # Paths for shortcuts
    $StartMenuDir = Join-Path ([Environment]::GetFolderPath("StartMenu")) "Programs"
    $DesktopDir = [Environment]::GetFolderPath("Desktop")
    $StartMenuShortcut = Join-Path $StartMenuDir "Insight Reader.lnk"
    $DesktopShortcut = Join-Path $DesktopDir "Insight Reader.lnk"
    
    # Create Start Menu Programs directory if it doesn't exist
    if (-not (Test-Path $StartMenuDir)) {
        New-Item -ItemType Directory -Path $StartMenuDir -Force | Out-Null
    }
    
    # Get ICO logo (check local first, then download from GitHub)
    $IconPath = "$InsightReaderBin,0"  # Default to executable icon
    
    # Check for local ICO file first (for development)
    # Try multiple methods to get script directory
    $ScriptDir = $PSScriptRoot
    if (-not $ScriptDir) {
        $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    }
    if (-not $ScriptDir) {
        $ScriptDir = Split-Path -Parent $PSCommandPath
    }
    if (-not $ScriptDir) {
        # Fallback: use current directory
        $ScriptDir = Get-Location
    }
    
    $ProjectRoot = Split-Path -Parent $ScriptDir
    $LocalIco = Join-Path $ProjectRoot "assets\logo.ico"
    $IconIco = Join-Path $InstallDir "insight-reader.ico"
    
    # Try to use local ICO file if it exists
    if (Test-Path $LocalIco) {
        Write-ColorOutput "Using local logo icon from: $LocalIco" "INFO"
        try {
            Copy-Item -Path $LocalIco -Destination $IconIco -Force -ErrorAction Stop
            $IconPath = $IconIco
            Write-ColorOutput "Logo icon copied to: $IconIco" "SUCCESS"
        } catch {
            Write-ColorOutput "Could not copy local icon, trying download..." "WARN"
            # Fall through to download attempt
        }
    }
    
    # If local copy failed or doesn't exist, try downloading from GitHub
    if ($IconPath -eq "$InsightReaderBin,0") {
        $IconUrl = "https://raw.githubusercontent.com/$GithubRepo/master/assets/logo.ico"
        try {
            Write-ColorOutput "Downloading logo icon from GitHub..." "INFO"
            Invoke-WebRequest -Uri $IconUrl -OutFile $IconIco -ErrorAction Stop
            
            # Verify the ICO file was downloaded and is valid
            if (Test-Path $IconIco) {
                $IconPath = $IconIco
                Write-ColorOutput "Logo icon downloaded to: $IconIco" "SUCCESS"
            } else {
                Write-ColorOutput "ICO file not found after download, using executable icon" "WARN"
                $IconPath = "$InsightReaderBin,0"
            }
        } catch {
            Write-ColorOutput "Could not download logo icon, using executable icon" "WARN"
            $IconPath = "$InsightReaderBin,0"
        }
    }
    
    # Create shortcut function
    function New-Shortcut {
        param(
            [string]$TargetPath,
            [string]$ShortcutPath,
            [string]$Description,
            [string]$IconLocation
        )
        
        try {
            $WshShell = New-Object -ComObject WScript.Shell
            $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
            $Shortcut.TargetPath = $TargetPath
            $Shortcut.WorkingDirectory = $BinDir
            $Shortcut.Description = $Description
            
            # Set icon location (use absolute path for ICO files)
            if ($IconLocation -like "*.ico") {
                # For ICO files, use the full path
                $IconLocation = (Resolve-Path $IconLocation -ErrorAction SilentlyContinue).Path
                if (-not $IconLocation) {
                    $IconLocation = (Get-Item $IconLocation -ErrorAction SilentlyContinue).FullName
                }
            }
            $Shortcut.IconLocation = $IconLocation
            
            $Shortcut.Save()
            Write-ColorOutput "Shortcut created with icon: $IconLocation" "INFO"
            return $true
        } catch {
            Write-ColorOutput "Failed to create shortcut: $_" "ERROR"
            return $false
        }
    }
    
    # Create Start Menu shortcut (auto-create, no prompt)
    if (New-Shortcut -TargetPath $InsightReaderBin -ShortcutPath $StartMenuShortcut -Description "Insight Reader - Text-to-Speech application" -IconLocation $IconPath) {
        Write-ColorOutput "Start Menu shortcut created" "SUCCESS"
    }
    
    # Create Desktop shortcut (auto-create, no prompt)
    if (New-Shortcut -TargetPath $InsightReaderBin -ShortcutPath $DesktopShortcut -Description "Insight Reader - Text-to-Speech application" -IconLocation $IconPath) {
        Write-ColorOutput "Desktop shortcut created" "SUCCESS"
    }
    
    return $true
}

function Show-Summary {
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  Installation Complete!" -ForegroundColor Cyan
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Installation directory: $InstallDir"
    Write-Host "Binary: $InsightReaderBin"
    Write-Host "Piper venv: $VenvDir\Scripts\piper.exe"
    Write-Host "Models directory: $ModelsDir"
    Write-Host ""
    Write-Host "Run insight-reader with: insight-reader" -ForegroundColor Green
    Write-Host ""
}

# Main installation
function Main {
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  Insight Reader Installation Script" -ForegroundColor Cyan
    Write-Host "  Windows Edition" -ForegroundColor Cyan
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host ""
    
    Write-ColorOutput "Installing to: $InstallDir" "INFO"
    Write-ColorOutput "Binary will be installed to: $BinDir" "INFO"
    Write-Host ""
    
    # Check dependencies
    Install-Dependencies
    
    # Install binary
    Write-Host ""
    Install-Binary
    
    # Create venv and install piper (unless skipped)
    if (-not $SkipPython) {
        Write-Host ""
        if (New-PythonVenv) {
            if (-not $SkipPiper) {
                Write-Host ""
                Install-Piper | Out-Null
            }
        }
    }
    
    # OCR functionality uses built-in Windows.Media.Ocr API - no installation needed
    Write-Host ""
    Write-ColorOutput "OCR functionality uses Windows built-in OCR (no installation required)" "INFO"
    
    # Download model
    Write-Host ""
    Install-Model | Out-Null
    
    # Add to PATH
    Write-Host ""
    Add-ToPath
    
    # Create shortcuts (unless skipped)
    if (-not $SkipShortcuts) {
        Write-Host ""
        Create-Shortcuts | Out-Null
    }
    
    # Show summary
    Show-Summary
}

# Run main function
Main
