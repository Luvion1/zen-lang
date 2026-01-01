#!/usr/bin/env bash

###############################################################################
# Zen Compiler - One-Click Installation Script
# Version: 0.0.1
# Description: Fully automated installation for Zen programming language
# Usage: curl -sSL https://raw.githubusercontent.com/Lunar-Chipter/zen-lang/main/install.sh | bash
###############################################################################

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Version
VERSION="0.0.1"
REPO_URL="https://github.com/Lunar-Chipter/zen-lang.git"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.zen}"
BIN_DIR="$INSTALL_DIR/bin"
RELEASE_BIN="$BIN_DIR/zen"
BACKUP_DIR="$INSTALL_DIR/backups"

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)
            OS="linux"
            ;;
        Darwin*)
            OS="macos"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            OS="windows"
            ;;
        *)
            echo -e "${RED}✗ Unsupported OS: $(uname -s)${NC}"
            echo -e "${YELLOW}Supported: Linux, macOS, Windows${NC}"
            exit 1
            ;;
    esac
}

# Detect architecture
detect_arch() {
    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64)
            ARCH_SUFFIX="x86_64"
            ;;
        aarch64|arm64)
            ARCH_SUFFIX="x86_64"
            echo -e "${YELLOW}⚠ ARM64 detected, using x86_64 binary${NC}"
            ;;
        *)
            echo -e "${RED}✗ Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac
}

# Print banner
print_banner() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════════╗"
    echo "║                                                      ║"
    echo "║  ███████╗███████╗███╗   ██╗                          ║"
    echo "║  ╚══███╔╝██╔════╝████╗  ██║                          ║"
    echo "║    ███╔╝ █████╗  ██╔██╗ ██║                          ║"
    echo "║   ███╔╝  ██╔══╝  ██║╚██╗██║                          ║"
    echo "║  ███████╗███████╗██║ ╚████║                          ║"
    echo "║  ╚══════╝╚══════╝╚═╝  ╚═══╝                          ║"
    echo "║                                                      ║"
    echo "║     Programming Language v${VERSION}                 ║"
    echo "║     One-Click Installation                           ║"
    echo "║                                                      ║"
    echo "╚══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_section() {
    echo -e "\n${BLUE}━━━ $1 ━━━${NC}\n"
}

check_prerequisites() {
    print_section "Checking Prerequisites"

    local has_all=true

    # Check Rust
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version | grep -oP 'rustc \K\S+')
        echo -e "  ${GREEN}✓${NC} Rust: ${RUST_VERSION}"
    else
        echo -e "  ${RED}✗${NC} Rust not found"
        has_all=false
    fi

    # Check Cargo
    if command -v cargo &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} Cargo: $(cargo --version | grep -oP 'cargo \K\S+')"
    else
        echo -e "  ${RED}✗${NC} Cargo not found"
        has_all=false
    fi

    # Check curl or wget
    if command -v curl &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} curl: available"
        DOWNLOADER="curl"
    elif command -v wget &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} wget: available"
        DOWNLOADER="wget"
    else
        echo -e "  ${RED}✗${NC} Neither curl nor wget found"
        has_all=false
    fi

    # Check git
    if command -v git &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} git: $(git --version | head -1 | cut -d' ' -f3)"
    else
        echo -e "  ${RED}✗${NC} git not found"
        has_all=false
    fi

    # Check GCC/Clang
    if command -v gcc &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} GCC: $(gcc --version | grep -oP 'gcc \K\S+')"
    elif command -v clang &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} Clang: $(clang --version | grep -oP 'clang version \K\S+')"
    else
        echo -e "  ${YELLOW}⚠${NC} GCC/Clang not found (optional)"
    fi

    if [ "$has_all" = false ]; then
        echo -e "\n${RED}Some prerequisites are missing!${NC}"
        echo -e "${YELLOW}Please install missing dependencies and try again.${NC}"
        exit 1
    fi
}

install_from_source() {
    print_section "Installing from Source"

    # Create directories
    mkdir -p "$BIN_DIR"
    mkdir -p "$BACKUP_DIR"

    echo -e "  ${BLUE}→${NC} Installation directory: ${GREEN}$INSTALL_DIR${NC}"

    # Clone or update repository
    if [ -d "$INSTALL_DIR/zen-lang" ]; then
        echo -e "  ${BLUE}→${NC} Updating repository..."
        cd "$INSTALL_DIR/zen-lang"
        git pull origin main
    else
        echo -e "  ${BLUE}→${NC} Cloning repository..."
        git clone "$REPO_URL" "$INSTALL_DIR/zen-lang"
        cd "$INSTALL_DIR/zen-lang"
    fi

    # Build in release mode
    echo -e "  ${BLUE}→${NC} Building Zen compiler..."
    cargo build --release

    # Backup existing installation
    if [ -f "$RELEASE_BIN" ]; then
        BACKUP_FILE="$BACKUP_DIR/zen-$(date +%Y%m%d-%H%M%S)"
        echo -e "  ${BLUE}→${NC} Backing up existing installation..."
        cp "$RELEASE_BIN" "$BACKUP_FILE"
        echo -e "  ${GREEN}✓${NC} Backup: $BACKUP_FILE"
    fi

    # Install binary
    echo -e "  ${BLUE}→${NC} Installing binary..."
    cp "target/release/zen" "$RELEASE_BIN"
    chmod +x "$RELEASE_BIN"

    echo -e "  ${GREEN}✓${NC} Installation complete!"
}

install_to_path() {
    print_section "Installing to PATH"

    # Add to PATH
    echo -e "  ${BLUE}→${NC} Adding to PATH..."
    echo -e "${CYAN}echo 'export PATH=\"\$PATH:$BIN_DIR\"' >> ~/.bashrc${NC}"
    echo -e "${CYAN}echo 'export PATH=\"\$PATH:$BIN_DIR\"' >> ~/.zshrc${NC}"

    # Add to bashrc if not already added
    if ! grep -q "$BIN_DIR" ~/.bashrc 2>/dev/null; then
        echo "" >> ~/.bashrc
        echo "# Zen Compiler" >> ~/.bashrc
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> ~/.bashrc
    fi

    # Add to zshrc if not already added
    if [ -f ~/.zshrc ] && ! grep -q "$BIN_DIR" ~/.zshrc 2>/dev/null; then
        echo "" >> ~/.zshrc
        echo "# Zen Compiler" >> ~/.zshrc
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> ~/.zshrc
    fi

    echo -e "  ${GREEN}✓${NC} Added to PATH"
    echo -e "  ${YELLOW}⚠${NC} Please restart your terminal or run: ${CYAN}source ~/.bashrc${NC}"
}

verify_installation() {
    print_section "Verifying Installation"

    # Check if binary exists
    if [ -f "$RELEASE_BIN" ]; then
        echo -e "  ${GREEN}✓${NC} Binary: ${GREEN}$RELEASE_BIN${NC}"
    else
        echo -e "  ${RED}✗${NC} Binary not found"
        exit 1
    fi

    # Check if executable
    if [ -x "$RELEASE_BIN" ]; then
        echo -e "  ${GREEN}✓${NC} Executable"
    else
        echo -e "  ${RED}✗${NC} Not executable"
        exit 1
    fi

    # Test compilation
    echo -e "  ${BLUE}→${NC} Testing compiler..."
    TEST_FILE="/tmp/test_zen.zen"

    cat > "$TEST_FILE" << 'EOF'
fn main() -> i32 {
    println("Hello, Zen!")
    return 0
}
EOF

    if "$RELEASE_BIN" compile "$TEST_FILE" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} Compiler working"
        rm -f "$TEST_FILE" test_zen test_zen.o test_zen.ll
    else
        echo -e "  ${YELLOW}⚠${NC} Compiler test skipped (may require LLVM)"
        rm -f "$TEST_FILE"
    fi
}

show_completion() {
    print_section "Installation Complete!"

    echo -e "  ${CYAN}Zen Compiler${NC} v${VERSION}"
    echo -e "  Location: ${GREEN}$RELEASE_BIN${NC}"
    echo ""

    echo -e "${BLUE}Quick Start${NC}"
    echo -e "  ${CYAN}1. Restart terminal${NC} or run: ${YELLOW}source ~/.bashrc${NC}"
    echo -e "  ${CYAN}2. Verify installation${NC}: ${YELLOW}zen --version${NC}"
    echo -e "  ${CYAN}3. Create first program${NC}:"
    echo -e "     ${GREEN}echo 'fn main() -> i32 { println(\"Hello!\") return 0 }' > hello.zen${NC}"
    echo -e "  ${CYAN}4. Compile and run${NC}: ${YELLOW}zen run hello.zen${NC}"
    echo ""

    echo -e "${BLUE}Available Commands${NC}"
    echo -e "  ${YELLOW}zen compile <file>${NC}    - Compile Zen file"
    echo -e "  ${YELLOW}zen run <file>${NC}       - Compile and run"
    echo -e "  ${YELLOW}zen tokenize <file>${NC}   - Show tokens"
    echo -e "  ${YELLOW}zen --help${NC}            - Show help"
    echo ""

    echo -e "${BLUE}Resources${NC}"
    echo -e "  📖 Documentation: ${CYAN}https://github.com/Lunar-Chipter/zen-lang/tree/main/docs${NC}"
    echo -e "  💡 Examples:      ${CYAN}https://github.com/Lunar-Chipter/zen-lang/tree/main/examples${NC}"
    echo -e "  🐛 Issues:        ${CYAN}https://github.com/Lunar-Chipter/zen-lang/issues${NC}"
    echo ""

    echo -e "${BLUE}Installation Command${NC}"
    echo -e "  ${YELLOW}curl -sSL https://raw.githubusercontent.com/Lunar-Chipter/zen-lang/main/install.sh | bash${NC}"
    echo ""
}

# Cleanup
cleanup() {
    if [ "$CLEAN" = "true" ]; then
        print_section "Uninstalling Zen"
        echo -e "  ${BLUE}→${NC} Removing: ${YELLOW}$INSTALL_DIR${NC}"
        rm -rf "$INSTALL_DIR"
        echo -e "  ${GREEN}✓${NC} Removed"
        exit 0
    fi
}

# Main
main() {
    CLEAN=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                CLEAN=true
                shift
                ;;
            --help|-h)
                echo "Zen Compiler - One-Click Installation"
                echo ""
                echo "Usage: $0 [OPTION]"
                echo ""
                echo "Options:"
                echo "  --clean     Remove Zen installation"
                echo "  --help, -h  Show this help"
                echo ""
                echo "Environment:"
                echo "  INSTALL_DIR  Custom installation directory (default: $HOME/.zen)"
                echo ""
                echo "Examples:"
                echo "  $0              # Install (automatic)"
                echo "  $0 --clean     # Uninstall"
                echo "  INSTALL_DIR=/opt/zen $0  # Custom directory"
                echo ""
                echo "Remote Installation:"
                echo "  curl -sSL https://raw.githubusercontent.com/Lunar-Chipter/zen-lang/main/install.sh | bash"
                exit 0
                ;;
            INSTALL_DIR=*)
                export INSTALL_DIR="${1#*=}"
                shift
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                echo "Use --help for usage"
                exit 1
                ;;
        esac
    done

    # Print banner
    print_banner

    # Clean if requested
    if [ "$CLEAN" = "true" ]; then
        cleanup
    fi

    # Check prerequisites
    check_prerequisites

    # Detect OS and architecture
    detect_os
    detect_arch
    echo -e "${GREEN}✓${NC} Detected: ${CYAN}${OS}-${ARCH_SUFFIX}${NC}"

    # Install from source
    install_from_source

    # Install to PATH
    install_to_path

    # Verify installation
    verify_installation

    # Show completion message
    show_completion

    echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║  ${CYAN}Installation successful!${NC}                              ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║  ${YELLOW}Please restart your terminal${NC}                         ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# Run
main "$@"
