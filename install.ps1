#!/usr/bin/env pwsh
# Generic install script for WoW Emulation CLI tools with ephemeral key verification

param(
    [string]$Version = "",
    [string]$InstallDir = "",
    [switch]$Help
)

# ============================================================================
# PROJECT CONFIGURATION - Customize these for each project
# ============================================================================
$BinaryNames = "${env:BINARY_NAMES}"     # Required: Space-separated list, e.g., "binary1 binary2"
$BinaryDefault = "${env:BINARY_DEFAULT}"   # Required: Default binary, e.g., "binary1"
$Repo = "${env:REPO}"                     # Required: e.g., "wowemulation-dev/warcraft-rs"
$TagPrefix = "${env:TAG_PREFIX}"          # e.g., "warcraft-rs-v", "cascette-ribbit-v", "v"
# ============================================================================

# Verify required configuration
if ([string]::IsNullOrEmpty($BinaryNames) -or [string]::IsNullOrEmpty($BinaryDefault) -or [string]::IsNullOrEmpty($Repo)) {
    Write-Host "Error: BINARY_NAMES, BINARY_DEFAULT, and REPO must be configured"
    Write-Host "Usage: `$env:BINARY_NAMES=\"binary1 binary2\" `$env:BINARY_DEFAULT=binary1 `$env:REPO=org/repo .\$MyInvocation.MyCommand.Name [-Version <version>] [-InstallDir <path>] [-Help]"
    exit 1
}

# Validate that default is in list
$binaryList = $BinaryNames -split ' '
if ($BinaryDefault -notin $binaryList) {
    Write-Host "Error: BINARY_DEFAULT must be in BINARY_NAMES"
    Write-Host "BINARY_NAMES: $BinaryNames"
    Write-Host "BINARY_DEFAULT: $BinaryDefault"
    exit 1
}

# Set BinaryName to default (can be overridden by -Binary flag)
$BinaryName = $BinaryDefault

# Default values
if ([string]::IsNullOrEmpty($TagPrefix)) {
    $TagPrefix = "${BinaryName}-"
}

if ([string]::IsNullOrEmpty($InstallDir)) {
    $InstallDir = "$env:LOCALAPPDATA\Programs\$BinaryName"
}

$BaseUrl = "https://github.com/$Repo/releases"

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Write-Info {
    param($Message)
    Write-Host "[INFO] " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warn {
    param($Message)
    Write-Host "[WARN] " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-Error-And-Exit {
    param($Message)
    Write-Host "[ERROR] " -ForegroundColor Red -NoNewline
    Write-Host $Message
    exit 1
}

function Get-LatestVersion {
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        $version = $release.tag_name -replace [regex]::Escape($TagPrefix), ''
        return $version
    } catch {
        Write-Error-And-Exit "Failed to get latest version: $_"
    }
}

function Get-Platform {
    $arch = [System.Environment]::Is64BitOperatingSystem
    if ($arch) {
        $platform = "x86_64-pc-windows-msvc"
    } else {
        Write-Error-And-Exit "32-bit Windows is not supported"
    }

    # Check for ARM64
    $processorArch = $env:PROCESSOR_ARCHITECTURE
    if ($processorArch -eq "ARM64") {
        $platform = "aarch64-pc-windows-msvc"
    }

    return $platform
}

function Download-File {
    param(
        [string]$Url,
        [string]$OutFile
    )

    try {
        Write-Info "Downloading from $Url"
        Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing
    } catch {
        if ($_.Exception.Response.StatusCode -eq 404) {
            return $false
        }
        Write-Error-And-Exit "Failed to download: $_"
    }
    return $true
}

function Verify-Signature {
    param(
        [string]$File,
        [string]$PublicKeyFile
    )

    # Check for rsign or minisign
    $rsign = Get-Command rsign -ErrorAction SilentlyContinue
    $minisign = Get-Command minisign -ErrorAction SilentlyContinue

    if ($rsign) {
        Write-Info "Verifying signature with rsign..."
        $result = & rsign verify -p $PublicKeyFile $File 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error-And-Exit "Signature verification failed: $result"
        }
        Write-Info "Signature verification successful"
    } elseif ($minisign) {
        Write-Info "Verifying signature with minisign..."
        $result = & minisign -V -p $PublicKeyFile -m $File 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error-And-Exit "Signature verification failed: $result"
        }
        Write-Info "Signature verification successful"
    } else {
        Write-Warn "Neither rsign nor minisign found, skipping verification"
        Write-Warn "Install from: https://github.com/jedisct1/minisign/releases"
        Write-Warn "Or install rsign with: cargo install rsign2"
    }
}

function Install-Binary {
    param(
        [string]$Version,
        [string]$InstallDir
    )

    # Get version if not specified
    if ([string]::IsNullOrEmpty($Version)) {
        Write-Info "Fetching latest version..."
        $Version = Get-LatestVersion
    }

    Write-Info "Installing $BinaryName v$Version"

    # Detect platform
    $platform = Get-Platform
    Write-Info "Detected platform: $platform"

    # Create temp directory
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }

    try {
        # Download files
        $fileName = "$BinaryName-$Version-$platform.zip"
        $downloadUrl = "$BaseUrl/download/$TagPrefix$Version/$fileName"
        $zipFile = Join-Path $tempDir $fileName
        $sigFile = "$zipFile.sig"
        $pubKeyFile = Join-Path $tempDir "minisign.pub"

        if (-not (Download-File -Url $downloadUrl -OutFile $zipFile)) {
            Write-Error-And-Exit "Failed to download binary"
        }

        # Download signature (ephemeral signing uses .sig extension)
        $sigDownloaded = Download-File -Url "$downloadUrl.sig" -OutFile $sigFile

        # Download public key (per-release ephemeral key)
        $pubKeyDownloaded = Download-File -Url "$BaseUrl/download/$TagPrefix$Version/minisign.pub" -OutFile $pubKeyFile

        # Verify signature if available
        if ($sigDownloaded -and $pubKeyDownloaded -and (Test-Path $sigFile) -and (Test-Path $pubKeyFile)) {
            Verify-Signature -File $zipFile -PublicKeyFile $pubKeyFile
        } else {
            Write-Warn "Signature or public key file not found, proceeding without verification"
        }

        # Extract binary
        Write-Info "Extracting binary..."
        Expand-Archive -Path $zipFile -DestinationPath $tempDir -Force

        # Create install directory
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        # Install binary
        $binaryName = "$BinaryName.exe"
        $sourcePath = Join-Path $tempDir $binaryName
        $destPath = Join-Path $InstallDir $binaryName

        if (Test-Path $sourcePath) {
            Write-Info "Installing to $destPath"
            Copy-Item -Path $sourcePath -Destination $destPath -Force
        } else {
            Write-Error-And-Exit "Binary not found in archive"
        }

        # Verify installation
        try {
            $testOutput = & $destPath --version 2>&1
            Write-Info "Installation successful!"
            Write-Info "Binary installed to: $destPath"
            Write-Info "Version: $testOutput"
        } catch {
            Write-Error-And-Exit "Installation verification failed"
        }

        # Check PATH
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$InstallDir*") {
            Write-Warn "$InstallDir is not in your PATH"
            Write-Info "To add it to PATH, run:"
            Write-Host "  [Environment]::SetEnvironmentVariable('Path', $env:Path + ';$InstallDir', 'User')"
            Write-Info "Or restart your terminal after running:"
            Write-Host "  setx PATH `"%PATH%;$InstallDir`""
        } else {
            Write-Info "$BinaryName is available in your PATH"
        }

    } finally {
        # Cleanup
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Main
if ($Help) {
    Write-Host @"
Generic install script for $Repo

USAGE:
    $($MyInvocation.MyCommand.Name) [-Version <version>] [-InstallDir <path>] [-Help]

ENVIRONMENT VARIABLES:
    BINARY_NAMES        Space-separated list of available binaries (required)
    BINARY_DEFAULT      Default binary to install (required)
    REPO                 GitHub repo (required)
    TAG_PREFIX           Release tag prefix (default: `${BINARY_DEFAULT}-)
    INSTALL_DIR          Override default install directory

OPTIONS:
    -Version     Install specific version (default: latest)
    -InstallDir  Installation directory (default: $env:LOCALAPPDATA\Programs\$BinaryName)
    -Help        Show this help message

EXAMPLES:
    # Install latest version (uses default binary)
    `$env:BINARY_NAMES="`"$BinaryNames`"" `$env:BINARY_DEFAULT=$BinaryDefault `$env:REPO=$Repo .\$MyInvocation.MyCommand.Name

    # Install specific version
    `$env:BINARY_NAMES="`"$BinaryNames`"" `$env:BINARY_DEFAULT=$BinaryDefault `$env:REPO=$Repo .\$MyInvocation.MyCommand.Name -Version 0.1.0

    # Install to custom directory
    `$env:BINARY_NAMES="`"$BinaryNames`"" `$env:BINARY_DEFAULT=$BinaryDefault `$env:REPO=$Repo .\$MyInvocation.MyCommand.Name -InstallDir "C:\Tools\$BinaryName"
"@

    exit 0
}

Install-Binary -Version $Version -InstallDir $InstallDir
