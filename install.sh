#!/usr/bin/env bash
# Generic install script for WoW Emulation CLI tools
# Based on cargo-binstall's installation approach

set -euo pipefail

# ============================================================================
# PROJECT CONFIGURATION - Customize these for each project
# ============================================================================
BINARY_NAMES="${BINARY_NAMES:-}"         # Required: Space-separated list, e.g., "ribbit-server cli bnet-agent"
BINARY_DEFAULT="${BINARY_DEFAULT:-}"     # Required: Default binary to install, must be in BINARY_NAMES
REPO="${REPO:-}"                         # Required: e.g., "wowemulation-dev/cascette-rs"
TAG_PREFIX="${TAG_PREFIX:-${BINARY_DEFAULT}-}" # e.g., "cascette-ribbit-v", "warcraft-rs-v", "v"
# ============================================================================

# Verify required configuration
if [[ -z "$BINARY_NAMES" ]] || [[ -z "$BINARY_DEFAULT" ]] || [[ -z "$REPO" ]]; then
    echo "Error: BINARY_NAMES, BINARY_DEFAULT, and REPO must be configured"
    echo "Usage: BINARY_NAMES=\"binary1 binary2\" BINARY_DEFAULT=binary1 REPO=org/repo $0 [OPTIONS] [VERSION]"
    exit 1
fi

# Validate that default is in the list
if [[ ! " $BINARY_NAMES " =~ " $BINARY_DEFAULT " ]]; then
    echo "Error: BINARY_DEFAULT must be in BINARY_NAMES"
    echo "BINARY_NAMES: $BINARY_NAMES"
    echo "BINARY_DEFAULT: $BINARY_DEFAULT"
    exit 1
fi

# Set BINARY_NAME to default (can be overridden by --binary flag)
BINARY_NAME="$BINARY_DEFAULT"

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
        *)              error "Unsupported architecture: $(uname -m)";;
    esac

    # Detect libc for Linux
    if [[ "$os" == "unknown-linux" ]]; then
        if ldd --version 2>&1 | grep -q musl; then
            os="${os}-musl"
        else
            os="${os}-gnu"
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
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E "s/.*\"${TAG_PREFIX}([^\"]+)\".*/\1/")

    if [[ -z "$version" ]]; then
        error "Failed to get latest version"
    fi

    echo "$version"
}

# Verify binary with ephemeral minisign key
verify_binary() {
    local file="$1"
    local pubkey_file="$2"

    # Check if signature verification tools are available
    if command -v rsign >/dev/null 2>&1; then
        info "Verifying signature with rsign..."
        if rsign verify -p "$pubkey_file" "$file"; then
            info "Signature verification successful"
        else
            error "Signature verification failed"
        fi
    elif command -v minisign >/dev/null 2>&1; then
        info "Verifying signature with minisign..."
        if minisign -V -p "$pubkey_file" -m "$file"; then
            info "Signature verification successful"
        else
            error "Signature verification failed"
        fi
    else
        warn "Neither rsign nor minisign found, skipping signature verification"
        warn "Install minisign or rsign to verify binary signatures"
    fi
}

# Main installation function
install() {
    local version="${1:-}"
    local install_dir="${INSTALL_DIR:-$HOME/.local/bin}"
    local temp_dir
    temp_dir=$(mktemp -d)

    trap 'rm -rf "$temp_dir"' EXIT

    # Get version if not specified
    if [[ -z "$version" ]]; then
        info "Fetching latest version..."
        version=$(get_latest_version)
    fi

    info "Installing ${BINARY_NAME} v${version}"

    # Detect platform
    local platform
    platform=$(detect_platform)
    info "Detected platform: ${platform}"

    # Determine file extension and archive format
    local ext archive_ext
    if [[ "$platform" == *"windows"* ]]; then
        ext=".exe"
        archive_ext=".zip"
    else
        ext=""
        archive_ext=".tar.gz"
    fi

    # Download URL
    local filename="${BINARY_NAME}-${version}-${platform}${archive_ext}"
    local download_url="${BASE_URL}/download/${TAG_PREFIX}${version}/${filename}"

    info "Downloading from: ${download_url}"
    download_file "$download_url" "${temp_dir}/${filename}"

    # Download signature and public key (ephemeral signing uses .sig extension)
    download_file "${download_url}.sig" "${temp_dir}/${filename}.sig" || warn "Signature file not found"
    download_file "${BASE_URL}/download/${TAG_PREFIX}${version}/minisign.pub" "${temp_dir}/minisign.pub" || warn "Public key not found"

    # Verify if signature and public key were downloaded
    if [[ -f "${temp_dir}/${filename}.sig" ]] && [[ -f "${temp_dir}/minisign.pub" ]]; then
        verify_binary "${temp_dir}/${filename}" "${temp_dir}/minisign.pub"
    fi

    # Extract binary
    info "Extracting binary..."
    cd "$temp_dir"
    if [[ "$archive_ext" == ".zip" ]]; then
        unzip -q "$filename"
    else
        tar -xzf "$filename"
    fi

    # Create install directory if it doesn't exist
    mkdir -p "$install_dir"

    # Install binary
    local binary_name="${BINARY_NAME}${ext}"
    if [[ -f "$binary_name" ]]; then
        info "Installing to ${install_dir}/${binary_name}"
        mv "$binary_name" "${install_dir}/"
        chmod +x "${install_dir}/${binary_name}"
    else
        error "Binary ${binary_name} not found in archive"
    fi

    # Verify installation
    if "${install_dir}/${binary_name}" --version >/dev/null 2>&1; then
        info "Installation successful!"
        info "Binary installed to: ${install_dir}/${binary_name}"

        # Check if install_dir is in PATH
        if ! echo "$PATH" | grep -q "$install_dir"; then
            warn "${install_dir} is not in your PATH"
            warn "Add it to your PATH by adding this to your shell configuration:"
            warn "  export PATH=\"${install_dir}:\$PATH\""
        fi
    else
        error "Installation verification failed"
    fi
}

# Show help
show_help() {
    cat << EOF
Generic install script for ${REPO}

USAGE:
    $0 [OPTIONS] [VERSION]

OPTIONS:
    -h, --help          Show this help message
    -d, --dir DIR       Install directory (default: \$HOME/.local/bin)
    -t, --tag TAG       Install specific release tag
    -b, --binary BINARY Binary to install (default: ${BINARY_DEFAULT})

ENVIRONMENT VARIABLES:
    BINARY_NAMES        Space-separated list of available binaries (required)
    BINARY_DEFAULT      Default binary to install (required)
    REPO                 GitHub repo (required)
    TAG_PREFIX           Release tag prefix (default: \${BINARY_DEFAULT}-)
    INSTALL_DIR          Override default install directory

AVAILABLE BINARIES:
$(for bin in $BINARY_NAMES; do echo "    - $bin"; done)

EXAMPLES:
    # Install latest version (uses default binary: ${BINARY_DEFAULT})
    BINARY_NAMES=\"${BINARY_NAMES}\" BINARY_DEFAULT=${BINARY_DEFAULT} REPO=org/repo $0

    # Install specific binary
    BINARY_NAMES=\"${BINARY_NAMES}\" BINARY_DEFAULT=${BINARY_DEFAULT} REPO=org/repo $0 --binary cli

    # Install specific version
    BINARY_NAMES=\"${BINARY_NAMES}\" BINARY_DEFAULT=${BINARY_DEFAULT} REPO=org/repo $0 0.1.0

    # Install to custom directory
    BINARY_NAMES=\"${BINARY_NAMES}\" BINARY_DEFAULT=${BINARY_DEFAULT} REPO=org/repo INSTALL_DIR=/usr/local/bin $0

    # Install specific tag
    BINARY_NAMES=\"${BINARY_NAMES}\" BINARY_DEFAULT=${BINARY_DEFAULT} REPO=org/repo $0 --tag ${BINARY_DEFAULT}-v0.1.0

SUPPORTED PROJECTS:
    - warcraft-rs   BINARY_NAMES=\"warcraft-rs\" BINARY_DEFAULT=warcraft-rs REPO=wowemulation-dev/warcraft-rs
    - cascette-rs   BINARY_NAMES=\"ribbit-server cli\" BINARY_DEFAULT=ribbit-server REPO=wowemulation-dev/cascette-rs
    - rilua        BINARY_NAMES=\"rilua\" BINARY_DEFAULT=rilua REPO=wowemulation-dev/rilua
    - wow-patcher   BINARY_NAMES=\"wow-patcher\" BINARY_DEFAULT=wow-patcher REPO=wowemulation-dev/wow-patcher
    - recast-rs    BINARY_NAMES=\"recast-cli\" BINARY_DEFAULT=recast-cli REPO=wowemulation-dev/recast-rs
EOF
}

# Parse command line arguments
main() {
    local version=""
    local tag=""
    local binary=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -d|--dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            -t|--tag)
                tag="$2"
                shift 2
                ;;
            -b|--binary)
                binary="$2"
                shift 2
                ;;
            -*)
                error "Unknown option: $1"
                ;;
            *)
                version="$1"
                shift
                ;;
        esac
    done

    # Validate binary selection if provided
    if [[ -n "$binary" ]]; then
        if [[ ! " $BINARY_NAMES " =~ " $binary " ]]; then
            echo "Error: Unknown binary '$binary'"
            echo "Available binaries: $BINARY_NAMES"
            exit 1
        fi
        BINARY_NAME="$binary"
    fi

    # Extract version from tag if provided
    if [[ -n "$tag" ]]; then
        version="${tag#"${TAG_PREFIX}"}"
    fi

    install "$version"
}

# Run main function
main "$@"
