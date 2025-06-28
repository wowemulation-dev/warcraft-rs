#!/usr/bin/env bash
# Install script for warcraft-rs
# Based on cargo-binstall's installation approach

set -euo pipefail

# Configuration
BINARY_NAME="warcraft-rs"
REPO="wowemulation-dev/warcraft-rs"
BASE_URL="https://github.com/${REPO}/releases"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os arch

    # Detect OS
    case "$(uname -s)" in
        Linux*)     os="unknown-linux";;
        Darwin*)    os="apple-darwin";;
        MINGW*|MSYS*|CYGWIN*)  os="pc-windows";;
        *)          error "Unsupported operating system: $(uname -s)";;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64";;
        aarch64|arm64)  arch="aarch64";;
        armv7l)         arch="armv7";;
        *)              error "Unsupported architecture: $(uname -m)";;
    esac

    # Detect libc for Linux
    if [[ "$os" == "unknown-linux" ]]; then
        if ldd --version 2>&1 | grep -q musl; then
            os="${os}-musl"
        else
            os="${os}-gnu"
        fi

        # Special case for armv7
        if [[ "$arch" == "armv7" ]]; then
            if [[ "$os" == "unknown-linux-musl" ]]; then
                os="unknown-linux-musleabihf"
            else
                os="unknown-linux-gnueabihf"
            fi
        fi
    fi

    # Windows uses different extension
    if [[ "$os" == "pc-windows" ]]; then
        os="${os}-msvc"
    fi

    echo "${arch}-${os}"
}

# Download and verify file
download_file() {
    local url="$1"
    local output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Get latest release version
get_latest_version() {
    local version
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')

    if [[ -z "$version" ]]; then
        error "Failed to get latest version"
    fi

    echo "$version"
}

# Main installation function
install_warcraft_rs() {
    local version="${1:-}"
    local install_dir="${CARGO_HOME:-$HOME/.cargo}/bin"

    # Get version if not specified
    if [[ -z "$version" ]]; then
        info "Getting latest version..."
        version=$(get_latest_version)
    fi

    info "Installing warcraft-rs v${version}"

    # Detect platform
    local platform
    platform=$(detect_platform)
    info "Detected platform: ${platform}"

    # Determine file extension
    local ext="tar.gz"
    if [[ "$platform" == *"windows"* ]]; then
        ext="zip"
    fi

    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap 'rm -rf $temp_dir' EXIT

    # Download archive and signature
    local archive_url="${BASE_URL}/download/v${version}/${BINARY_NAME}-${platform}.${ext}"
    local sig_url="${archive_url}.minisig"
    local archive_path="${temp_dir}/${BINARY_NAME}-${platform}.${ext}"
    local sig_path="${archive_path}.minisig"

    info "Downloading ${BINARY_NAME}..."
    download_file "$archive_url" "$archive_path"

    # Verify signature if minisign is available
    if command -v minisign >/dev/null 2>&1; then
        info "Downloading signature..."
        download_file "$sig_url" "$sig_path"

        # Download public key
        local pubkey_url="${BASE_URL}/download/v${version}/minisign.pub"
        local pubkey_path="${temp_dir}/minisign.pub"
        download_file "$pubkey_url" "$pubkey_path"

        info "Verifying signature..."
        if minisign -V -p "$pubkey_path" -m "$archive_path"; then
            info "Signature verified successfully"
        else
            warn "Signature verification failed, proceeding anyway"
        fi
    else
        warn "minisign not found, skipping signature verification"
    fi

    # Extract archive
    info "Extracting archive..."
    cd "$temp_dir"
    if [[ "$ext" == "zip" ]]; then
        unzip -q "$archive_path"
    else
        tar -xzf "$archive_path"
    fi

    # Install binary
    info "Installing to ${install_dir}..."
    mkdir -p "$install_dir"

    local binary_name="${BINARY_NAME}"
    if [[ "$platform" == *"windows"* ]]; then
        binary_name="${BINARY_NAME}.exe"
    fi

    if [[ -f "$binary_name" ]]; then
        chmod +x "$binary_name"
        mv "$binary_name" "${install_dir}/"
        info "Successfully installed ${BINARY_NAME} v${version}"
        info "Run '${BINARY_NAME} --version' to verify installation"
    else
        error "Binary not found in archive"
    fi
}

# Parse command line arguments
main() {
    local version=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                version="$2"
                shift 2
                ;;
            -h|--help)
                echo "Usage: $0 [-v|--version VERSION]"
                echo "Install warcraft-rs binary"
                echo ""
                echo "Options:"
                echo "  -v, --version VERSION    Install specific version (default: latest)"
                echo "  -h, --help              Show this help message"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done

    install_warcraft_rs "$version"
}

main "$@"
