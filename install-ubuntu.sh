#!/usr/bin/env bash
#
# install-ubuntu.sh - Build and install fzf from source on Ubuntu
#
# This script downloads fzf source code from GitHub, compiles the release binary,
# and installs it system-wide, providing an experience similar to `apt install fzf`.
#

set -euo pipefail

# Configuration
readonly FZF_VERSION="${FZF_VERSION:-0.72.0}"
readonly PREFIX="${PREFIX:-/usr/local}"
readonly INSTALL_SHELL_INTEGRATION="${INSTALL_SHELL_INTEGRATION:-true}"
readonly FZF_REPO="${FZF_REPO:-https://github.com/junegunn/fzf}"

# Build directory (temp or local)
BUILD_DIR=""
SOURCE_DIR=""

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Error handling
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        log_error "Installation failed with exit code $exit_code"
    fi

    # Clean up temp directory if we created one
    if [[ -n "${BUILD_DIR:-}" && -d "$BUILD_DIR" && "$BUILD_DIR" != "$SOURCE_DIR" ]]; then
        rm -rf "$BUILD_DIR"
    fi

    exit $exit_code
}
trap cleanup EXIT

# Check if running on Ubuntu
check_ubuntu() {
    log_info "Checking Ubuntu environment..."

    if ! grep -qE "^(ID|ID_LIKE)=.*ubuntu" /etc/os-release 2>/dev/null; then
        log_warn "This script is designed for Ubuntu. Continuing anyway..."
    else
        log_info "Ubuntu detected."
    fi
}

# Check and install build dependencies
check_dependencies() {
    log_info "Checking build dependencies..."

    local missing_deps=()

    # Check for essential tools
    for cmd in git curl tar make; do
        if ! command -v "$cmd" &>/dev/null; then
            missing_deps+=("$cmd")
        fi
    done

    # Check for Rust toolchain
    if ! command -v rustc &>/dev/null; then
        log_warn "Rust not found. Will attempt to install via rustup."
        missing_deps+=("rust")
    elif ! command -v cargo &>/dev/null; then
        log_error "Rust found but Cargo is missing. Please reinstall Rust."
        exit 1
    else
        local rust_version
        rust_version=$(rustc --version | cut -d' ' -f2)
        log_info "Found Rust version: $rust_version"
    fi

    # If we have missing deps, try to install them
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_info "Missing dependencies: ${missing_deps[*]}"

        # Check if we can use apt
        if ! command -v apt-get &>/dev/null; then
            log_error "apt-get not found. Please install the following manually: ${missing_deps[*]}"
            exit 1
        fi

        # Try to install with sudo, or check if we're root
        if [[ $EUID -ne 0 ]]; then
            if ! sudo -n apt-get update &>/dev/null 2>&1; then
                log_error "Root privileges required to install dependencies. Please run with sudo or install manually:"
                log_error "  sudo apt-get update && sudo apt-get install -y git curl tar make"
                exit 1
            fi
        fi

        log_info "Installing dependencies..."
        local apt_deps=()
        for dep in "${missing_deps[@]}"; do
            case "$dep" in
                git|curl|tar|make) apt_deps+=("$dep") ;;
                rust) ;; # Handle Rust separately
            esac
        done

        if [[ ${#apt_deps[@]} -gt 0 ]]; then
            if [[ $EUID -eq 0 ]]; then
                apt-get update && apt-get install -y "${apt_deps[@]}"
            else
                sudo apt-get update && sudo apt-get install -y "${apt_deps[@]}"
            fi
        fi

        # Install Rust if needed
        if [[ " ${missing_deps[*]} " =~ " rust " ]]; then
            install_rust
        fi
    fi

    log_info "All dependencies satisfied."
}

# Install Rust via rustup
install_rust() {
    log_info "Installing Rust via rustup..."

    if [[ -d "$HOME/.cargo/bin" ]]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi

    if ! command -v rustc &>/dev/null; then
        log_info "Downloading and installing rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        export PATH="$HOME/.cargo/bin:$PATH"
    fi

    # Verify installation
    if ! command -v rustc &>/dev/null || ! command -v cargo &>/dev/null; then
        log_error "Failed to install Rust. Please install manually from https://rustup.rs/"
        exit 1
    fi

    log_info "Rust installed successfully."
}

# Fetch fzf source code from GitHub
fetch_source_code() {
    log_info "Fetching fzf source code (version: $FZF_VERSION)..."

    # Check for local source first (if running from repo)
    local script_dir
    script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

    if [[ -f "$script_dir/Cargo.toml" || -f "$script_dir/go.mod" ]]; then
        log_info "Found local source code at: $script_dir"
        SOURCE_DIR="$script_dir"
        BUILD_DIR="$script_dir"
        return 0
    fi

    # Need to download source from GitHub
    BUILD_DIR=$(mktemp -d /tmp/fzf-build.XXXXXX)
    log_info "Using temporary build directory: $BUILD_DIR"

    local download_url="${FZF_REPO}/archive/refs/tags/v${FZF_VERSION}.tar.gz"
    local tarball="$BUILD_DIR/fzf-${FZF_VERSION}.tar.gz"

    log_info "Downloading source from: $download_url"

    # Download with curl or wget, with proper error handling
    local download_success=false

    if command -v curl &>/dev/null; then
        if curl -fL --connect-timeout 30 --max-time 300 -o "$tarball" "$download_url" 2>&1; then
            download_success=true
        else
            log_error "curl download failed"
        fi
    elif command -v wget &>/dev/null; then
        if wget --timeout=30 --tries=3 -O "$tarball" "$download_url" 2>&1; then
            download_success=true
        else
            log_error "wget download failed"
        fi
    else
        log_error "Neither curl nor wget is available. Cannot download source code."
        exit 1
    fi

    if [[ "$download_success" != "true" ]]; then
        log_error "Failed to download fzf source from: $download_url"
        log_error "Please check:"
        log_error "  1. Your internet connection is working"
        log_error "  2. The version $FZF_VERSION exists (see https://github.com/junegunn/fzf/releases)"
        log_error "  3. You may try a different version with: FZF_VERSION=x.x.x $0"
        exit 1
    fi

    # Verify tarball is valid
    if [[ ! -f "$tarball" ]] || [[ ! -s "$tarball" ]]; then
        log_error "Downloaded file is missing or empty"
        exit 1
    fi

    log_info "Extracting source code..."

    if ! tar -xzf "$tarball" -C "$BUILD_DIR" 2>&1; then
        log_error "Failed to extract tarball"
        exit 1
    fi

    # Find the extracted directory
    SOURCE_DIR=$(find "$BUILD_DIR" -maxdepth 1 -type d -name "fzf-*" | head -1)

    if [[ -z "$SOURCE_DIR" ]] || [[ ! -d "$SOURCE_DIR" ]]; then
        log_error "Could not find extracted source directory"
        exit 1
    fi

    log_info "Source code extracted to: $SOURCE_DIR"

    # Verify it's a valid fzf source
    if [[ ! -f "$SOURCE_DIR/Cargo.toml" && ! -f "$SOURCE_DIR/go.mod" && ! -f "$SOURCE_DIR/main.go" ]]; then
        log_error "Downloaded source does not appear to be valid fzf source code"
        log_error "Expected to find Cargo.toml (Rust) or go.mod/main.go (Go)"
        exit 1
    fi

    # Clean up tarball
    rm -f "$tarball"
}

# Detect project language and build
build_fzf() {
    log_info "Detecting project type..."

    cd "$SOURCE_DIR"

    if [[ -f "Cargo.toml" ]]; then
        log_info "Rust project detected."
        build_fzf_rust
    elif [[ -f "go.mod" ]] || [[ -f "main.go" ]]; then
        log_info "Go project detected."
        build_fzf_go
    else
        log_error "Unknown project type. Cannot determine build method."
        exit 1
    fi
}

# Build Rust version
build_fzf_rust() {
    log_info "Building fzf from Rust source..."

    # Ensure we have the right Rust version
    if ! command -v rustc &>/dev/null; then
        log_error "Rust compiler not found. Cannot build."
        exit 1
    fi

    log_info "Building release binary (this may take a few minutes)..."

    # Build release version
    if ! cargo build --release; then
        log_error "Build failed. Please check the error messages above."
        exit 1
    fi

    # Verify binary was created
    if [[ ! -f "$SOURCE_DIR/target/release/fzf" ]]; then
        log_error "Build completed but binary not found at expected location."
        exit 1
    fi

    log_info "Build successful."
}

# Build Go version
build_fzf_go() {
    log_info "Building fzf from Go source..."

    # Check for Go
    if ! command -v go &>/dev/null; then
        log_error "Go compiler not found but Go source detected."
        log_info "Attempting to install Go..."

        if [[ $EUID -eq 0 ]]; then
            apt-get update && apt-get install -y golang-go
        else
            sudo apt-get update && sudo apt-get install -y golang-go
        fi

        if ! command -v go &>/dev/null; then
            log_error "Failed to install Go. Please install manually."
            exit 1
        fi
    fi

    local go_version
    go_version=$(go version | cut -d' ' -f3)
    log_info "Using Go version: $go_version"

    log_info "Building release binary..."

    # Build with version info
    local ldflags="-s -w -X main.version=$FZF_VERSION -X main.revision=source-build"

    if ! go build -ldflags "$ldflags" -o "$SOURCE_DIR/fzf" .; then
        log_error "Build failed. Please check the error messages above."
        exit 1
    fi

    # Verify binary was created
    if [[ ! -f "$SOURCE_DIR/fzf" ]]; then
        log_error "Build completed but binary not found at expected location."
        exit 1
    fi

    log_info "Build successful."
}

# Get the path to the built binary
get_binary_path() {
    if [[ -f "$SOURCE_DIR/target/release/fzf" ]]; then
        echo "$SOURCE_DIR/target/release/fzf"
    elif [[ -f "$SOURCE_DIR/fzf" ]]; then
        echo "$SOURCE_DIR/fzf"
    else
        log_error "Cannot find built binary"
        exit 1
    fi
}

# Install the binary
install_binary() {
    log_info "Installing fzf binary to $PREFIX/bin..."

    local binary
    binary=$(get_binary_path)

    # Check if destination directory exists
    if [[ ! -d "$PREFIX/bin" ]]; then
        log_info "Creating directory: $PREFIX/bin"
        if [[ $EUID -eq 0 ]]; then
            mkdir -p "$PREFIX/bin"
        else
            sudo mkdir -p "$PREFIX/bin"
        fi
    fi

    # Install binary
    if [[ $EUID -eq 0 ]]; then
        cp "$binary" "$PREFIX/bin/fzf"
        chmod 755 "$PREFIX/bin/fzf"
    else
        sudo cp "$binary" "$PREFIX/bin/fzf"
        sudo chmod 755 "$PREFIX/bin/fzf"
    fi

    log_info "Binary installed to: $PREFIX/bin/fzf"
}

# Install shell integration files
install_shell_integration() {
    if [[ "$INSTALL_SHELL_INTEGRATION" != "true" ]]; then
        log_info "Skipping shell integration installation."
        return 0
    fi

    log_info "Installing shell integration files..."

    local shell_dir="$PREFIX/share/fzf"

    # Create shell integration directory
    if [[ $EUID -eq 0 ]]; then
        mkdir -p "$shell_dir"
    else
        sudo mkdir -p "$shell_dir"
    fi

    # Install shell scripts if they exist in the source
    local shell_scripts=("completion.bash" "completion.zsh" "key-bindings.bash" "key-bindings.zsh")

    if [[ -d "$SOURCE_DIR/shell" ]]; then
        for script in "${shell_scripts[@]}"; do
            if [[ -f "$SOURCE_DIR/shell/$script" ]]; then
                if [[ $EUID -eq 0 ]]; then
                    cp "$SOURCE_DIR/shell/$script" "$shell_dir/"
                else
                    sudo cp "$SOURCE_DIR/shell/$script" "$shell_dir/"
                fi
                log_info "Installed: $script"
            fi
        done
    else
        log_warn "Shell integration scripts not found in source."
    fi

    # Install man page if available
    if [[ -d "$SOURCE_DIR/man" ]]; then
        local man_dir="$PREFIX/share/man/man1"
        if [[ $EUID -eq 0 ]]; then
            mkdir -p "$man_dir"
        else
            sudo mkdir -p "$man_dir"
        fi

        for manpage in "$SOURCE_DIR/man"/*.1; do
            if [[ -f "$manpage" ]]; then
                if [[ $EUID -eq 0 ]]; then
                    cp "$manpage" "$man_dir/"
                else
                    sudo cp "$manpage" "$man_dir/"
                fi
                log_info "Installed man page: $(basename "$manpage")"
            fi
        done
    fi
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."

    # Check binary exists and is executable
    if [[ ! -x "$PREFIX/bin/fzf" ]]; then
        log_error "fzf binary not found or not executable at $PREFIX/bin/fzf"
        exit 1
    fi

    # Get version
    local installed_version
    installed_version=$("$PREFIX/bin/fzf" --version 2>&1 | head -1)

    if [[ $? -ne 0 ]]; then
        log_error "fzf --version failed. Installation may be corrupted."
        exit 1
    fi

    log_info "Successfully installed: $installed_version"
}

# Print post-installation instructions
print_instructions() {
    echo ""
    log_info "Installation complete!"
    echo ""
    echo "fzf has been installed to: $PREFIX/bin/fzf"
    echo ""

    # Check if in PATH
    if command -v fzf &>/dev/null; then
        log_info "fzf is available in your PATH."
    else
        log_warn "fzf is not in your current PATH."
        echo ""
        echo "To use fzf, you may need to add $PREFIX/bin to your PATH:"
        echo "  export PATH=\"$PREFIX/bin:\$PATH\""
        echo ""
        echo "Add this line to your ~/.bashrc or ~/.zshrc to make it permanent."
    fi

    if [[ "$INSTALL_SHELL_INTEGRATION" == "true" ]]; then
        echo ""
        echo "To enable shell integration, add the following to your shell config:"
        echo ""
        echo "For bash (~/.bashrc):"
        echo "  source $PREFIX/share/fzf/completion.bash"
        echo "  source $PREFIX/share/fzf/key-bindings.bash"
        echo ""
        echo "For zsh (~/.zshrc):"
        echo "  source $PREFIX/share/fzf/completion.zsh"
        echo "  source $PREFIX/share/fzf/key-bindings.zsh"
        echo ""
    fi

    echo "For more information, see: https://github.com/junegunn/fzf"
}

# Main function
main() {
    echo "====================================="
    echo "fzf Source Installation Script"
    echo "Version: $FZF_VERSION"
    echo "====================================="
    echo ""

    # Run installation steps
    check_ubuntu
    check_dependencies
    fetch_source_code
    build_fzf
    install_binary
    install_shell_integration
    verify_installation
    print_instructions

    log_info "Done!"
}

# Show help
if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    cat << EOF
Usage: $0 [OPTIONS]

Build and install fzf from source on Ubuntu.

Environment Variables:
  FZF_VERSION             Version to install (default: 0.72.0)
  PREFIX                  Installation prefix (default: /usr/local)
  INSTALL_SHELL_INTEGRATION  Install shell integration files (default: true)
  FZF_REPO                GitHub repository URL (default: https://github.com/junegunn/fzf)

Options:
  -h, --help              Show this help message

Examples:
  # Standard installation
  sudo $0

  # Install to custom location
  PREFIX=/opt/fzf $0

  # Install without shell integration
  INSTALL_SHELL_INTEGRATION=false $0

  # Build specific version
  FZF_VERSION=0.71.0 $0

  # Build from a fork
  FZF_REPO=https://github.com/myfork/fzf $0

EOF
    exit 0
fi

# Run main function
main
