#!/bin/bash
# Agentd installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/agentd/main/install.sh | bash

set -euo pipefail

# Configuration
REPO="chrischeng-c4/agentd"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
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

    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"

    # Install binary
    mv "$tmp_dir/agentd" "$INSTALL_DIR/$BINARY_NAME"

    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    success "Installed agentd to $INSTALL_DIR/$BINARY_NAME"

    # Create agentd-mcp symlink for MCP server
    # Claude Code blocks execution of binaries registered as MCP tools by name,
    # so we use agentd-mcp as the MCP command to avoid this restriction
    ln -sf "$BINARY_NAME" "$INSTALL_DIR/agentd-mcp"
    success "Created agentd-mcp symlink for MCP server"
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

    # Check if INSTALL_DIR is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "Note: $INSTALL_DIR is not in your PATH"
        echo ""
        info "Add to your shell config (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        info "Then reload your shell or run:"
        echo "  source ~/.zshrc  # or ~/.bashrc"
        echo ""
    fi

    # Verify installation
    if [[ -x "$INSTALL_DIR/$BINARY_NAME" ]]; then
        success "Installation successful!"
        echo ""
        "$INSTALL_DIR/$BINARY_NAME" --version
        echo ""

        # Upgrade configs if in an agentd project
        check_and_upgrade_configs

        info "Next steps:"
        echo "  1. Navigate to your project directory"
        echo "  2. Run: agentd init"
        echo ""
    else
        error "Installation failed - binary not found"
        exit 1
    fi
}

main "$@"
