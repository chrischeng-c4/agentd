#!/bin/bash
# Agentd installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/agentd/main/install.sh | bash

set -euo pipefail

# Configuration
REPO="chrischeng-c4/agentd"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="agentd"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info() { echo -e "${CYAN}$1${NC}"; }
success() { echo -e "${GREEN}$1${NC}"; }
warn() { echo -e "${YELLOW}$1${NC}"; }
error() { echo -e "${RED}$1${NC}" >&2; }

# Detect OS and architecture
detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin) os="darwin" ;;
        Linux) os="linux" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)
            error "Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Get latest release version from GitHub API
get_latest_version() {
    local version
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

    if [[ -z "$version" ]]; then
        error "Failed to get latest version"
        exit 1
    fi

    echo "$version"
}

# Download and install binary
install_binary() {
    local version="$1"
    local platform="$2"
    local download_url="https://github.com/${REPO}/releases/download/${version}/agentd-${platform}.tar.gz"
    local tmp_dir

    info "Downloading agentd ${version} for ${platform}..."

    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    # Download and extract
    if ! curl -fsSL "$download_url" | tar xz -C "$tmp_dir"; then
        error "Failed to download from: $download_url"
        error "Make sure the release exists for your platform."
        exit 1
    fi

    # Check if we need sudo
    if [[ -w "$INSTALL_DIR" ]]; then
        mv "$tmp_dir/agentd" "$INSTALL_DIR/$BINARY_NAME"
    else
        warn "Need sudo to install to $INSTALL_DIR"
        sudo mv "$tmp_dir/agentd" "$INSTALL_DIR/$BINARY_NAME"
    fi

    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    success "Installed agentd to $INSTALL_DIR/$BINARY_NAME"
}

# Check if agentd is already initialized in current directory
check_and_upgrade_configs() {
    if [[ -d "agentd" ]]; then
        info "Detected existing agentd installation, upgrading configs..."
        "$INSTALL_DIR/$BINARY_NAME" init --force
    fi
}

# Main
main() {
    echo ""
    info "=================================="
    info "   Agentd Installer"
    info "=================================="
    echo ""

    # Check dependencies
    if ! command -v curl &> /dev/null; then
        error "curl is required but not installed."
        exit 1
    fi

    # Detect platform
    local platform version
    platform=$(detect_platform)
    info "Detected platform: $platform"

    # Get latest version (or use specified version)
    version="${VERSION:-$(get_latest_version)}"
    info "Version: $version"
    echo ""

    # Install binary
    install_binary "$version" "$platform"
    echo ""

    # Verify installation
    if command -v agentd &> /dev/null; then
        success "Installation successful!"
        echo ""
        agentd --version
        echo ""

        # Upgrade configs if in an agentd project
        check_and_upgrade_configs

        echo ""
        info "Next steps:"
        echo "  1. Navigate to your project directory"
        echo "  2. Run: agentd init"
        echo ""
    else
        warn "agentd installed but not in PATH"
        warn "Add $INSTALL_DIR to your PATH"
    fi
}

main "$@"
