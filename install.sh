#!/bin/bash
# Specter installation script
# Usage: curl -fsSL https://raw.githubusercontent.com/your-repo/agentd/main/install.sh | sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo ""
echo -e "${CYAN}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
echo -e "${CYAN}┃  Specter Installation Script   ┃${NC}"
echo -e "${CYAN}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
echo ""

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

echo -e "${CYAN}Detecting system...${NC}"
echo "  OS: $OS"
echo "  Architecture: $ARCH"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust is not installed.${NC}"
    echo ""
    echo "Would you like to install Rust now? (y/n)"
    read -r INSTALL_RUST

    if [ "$INSTALL_RUST" = "y" ] || [ "$INSTALL_RUST" = "Y" ]; then
        echo -e "${CYAN}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}✓ Rust installed successfully${NC}"
    else
        echo -e "${RED}✗ Rust is required to install Specter${NC}"
        echo "  Install Rust from: https://rustup.rs"
        exit 1
    fi
else
    echo -e "${GREEN}✓ Rust is installed${NC}"
fi

# Check Rust version
RUST_VERSION=$(rustc --version | awk '{print $2}')
echo "  Rust version: $RUST_VERSION"
echo ""

# Check if Git is installed
if ! command -v git &> /dev/null; then
    echo -e "${RED}✗ Git is not installed${NC}"
    echo "  Please install Git and try again"
    exit 1
else
    echo -e "${GREEN}✓ Git is installed${NC}"
fi
echo ""

# Clone or update Specter repository
SPECTER_DIR="$HOME/.agentd-install"

if [ -d "$SPECTER_DIR" ]; then
    echo -e "${CYAN}Updating existing Specter repository...${NC}"
    cd "$SPECTER_DIR"
    git pull
else
    echo -e "${CYAN}Cloning Specter repository...${NC}"
    git clone https://github.com/your-repo/agentd.git "$SPECTER_DIR"
    cd "$SPECTER_DIR"
fi
echo ""

# Build and install
echo -e "${CYAN}Building and installing Specter...${NC}"
echo "  This may take a few minutes..."
echo ""

if cargo install --path . --locked; then
    echo ""
    echo -e "${GREEN}✓ Specter installed successfully!${NC}"
else
    echo ""
    echo -e "${RED}✗ Installation failed${NC}"
    exit 1
fi

# Verify installation
INSTALL_PATH="$HOME/.cargo/bin/agentd"
if [ -f "$INSTALL_PATH" ]; then
    echo ""
    echo -e "${GREEN}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
    echo -e "${GREEN}┃   Installation Complete! 🎭    ┃${NC}"
    echo -e "${GREEN}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
    echo ""
    echo "  Installed to: $INSTALL_PATH"
    echo ""

    # Check if cargo bin is in PATH
    if echo "$PATH" | grep -q "$HOME/.cargo/bin"; then
        echo -e "${GREEN}✓ $HOME/.cargo/bin is in your PATH${NC}"
        echo ""
        echo "You can now run:"
        echo -e "  ${CYAN}agentd --version${NC}"
        echo -e "  ${CYAN}agentd --help${NC}"
    else
        echo -e "${YELLOW}⚠ $HOME/.cargo/bin is NOT in your PATH${NC}"
        echo ""
        echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo -e "  ${CYAN}export PATH=\"\$HOME/.cargo/bin:\$PATH\"${NC}"
        echo ""
        echo "Then reload your shell:"
        echo -e "  ${CYAN}source ~/.bashrc${NC}  # or source ~/.zshrc"
    fi

    echo ""
    echo "Next steps:"
    echo -e "  1. Initialize a project:  ${CYAN}agentd init${NC}"
    echo -e "  2. Configure AI scripts:  ${CYAN}cp $SPECTER_DIR/examples/scripts/* .agentd/scripts/${NC}"
    echo -e "  3. Read the docs:         ${CYAN}https://github.com/your-repo/agentd${NC}"
    echo ""
else
    echo -e "${RED}✗ Installation verification failed${NC}"
    exit 1
fi

# Cleanup option
echo ""
echo "Remove installation files? (y/n) [Default: n]"
read -r CLEANUP
if [ "$CLEANUP" = "y" ] || [ "$CLEANUP" = "Y" ]; then
    rm -rf "$SPECTER_DIR"
    echo -e "${GREEN}✓ Cleaned up installation files${NC}"
fi

echo ""
echo -e "${CYAN}Happy coding with Specter! 🎭${NC}"
echo ""
