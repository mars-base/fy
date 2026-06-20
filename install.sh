#!/usr/bin/env bash
#
# fy - CLI translation tool installer
# Supports macOS (Apple Silicon) and Linux (x86_64)
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/wdw8276/fy/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/wdw8276/fy/main/install.sh | bash -s -- --help
#
set -euo pipefail

# ---------- config ----------
REPO="wdw8276/fy"
APP_NAME="fy"
INSTALL_DIR="/usr/local/bin"
GITHUB_API="https://api.github.com/repos/${REPO}"

# ---------- colors ----------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info()    { printf "${GREEN}[INFO]${NC} %s\n" "$*"; }
warn()    { printf "${YELLOW}[WARN]${NC} %s\n" "$*"; }
error()   { printf "${RED}[ERROR]${NC} %s\n" "$*" >&2; }
step()    { printf "${CYAN}==>${NC} %s\n" "$*"; }

# ---------- usage ----------
usage() {
    cat <<'EOF'
Usage: install.sh [OPTIONS]

Options:
  --version <tag>   Install a specific version (e.g. v0.1.1)
  --musl            (Linux only) Force musl (static) build
  --glibc           (Linux only) Force glibc (dynamic) build
  --help            Show this help message

Examples:
  install.sh                        # latest version, auto-detect
  install.sh --version v0.1.0       # specific version
  install.sh --musl                 # force musl on Linux
EOF
    exit 0
}

# ---------- parse args ----------
VERSION=""
FORCE_MUSL=false
FORCE_GLIBC=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version)
            VERSION="$2"; shift 2 ;;
        --musl)
            FORCE_MUSL=true; shift ;;
        --glibc)
            FORCE_GLIBC=true; shift ;;
        --help|-h)
            usage ;;
        *)
            error "Unknown option: $1"
            usage ;;
    esac
done

if $FORCE_MUSL && $FORCE_GLIBC; then
    error "--musl and --glibc are mutually exclusive"
    exit 1
fi

# ---------- detect platform ----------
step "Detecting platform..."

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)  PLATFORM="darwin" ;;
    Linux)   PLATFORM="linux" ;;
    *)
        error "Unsupported OS: $OS. Only macOS and Linux are supported."
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64" ;;
    arm64|aarch64)
        ARCH="arm64" ;;
    *)
        error "Unsupported architecture: $ARCH. Only x86_64 and arm64 are supported."
        exit 1
        ;;
esac

info "OS: $OS, Architecture: $ARCH"

# ---------- determine binary name ----------
if [[ "$PLATFORM" == "darwin" ]]; then
    if [[ "$ARCH" != "arm64" ]]; then
        error "macOS Intel (x86_64) has no pre-built binary. Please build from source:"
        echo "  make darwin-intel && sudo cp build/fy-darwin-amd64 ${INSTALL_DIR}/${APP_NAME}"
        exit 1
    fi
    if $FORCE_MUSL || $FORCE_GLIBC; then
        warn "--musl/--glibc are ignored on macOS"
    fi
    ASSET_PATTERN="aarch64-apple-darwin"
    FLAVOR="macOS arm64"
elif [[ "$PLATFORM" == "linux" ]]; then
    if [[ "$ARCH" != "x86_64" ]]; then
        error "Linux $ARCH has no pre-built binary. Only x86_64 is supported."
        exit 1
    fi
    # Default to musl (static, more portable)
    if $FORCE_GLIBC; then
        ASSET_PATTERN="x86_64-unknown-linux-gnu"
        FLAVOR="Linux x86_64 (glibc)"
    else
        ASSET_PATTERN="x86_64-unknown-linux-musl"
        FLAVOR="Linux x86_64 (musl)"
    fi
fi

# ---------- get latest version if not specified ----------
if [[ -z "$VERSION" ]]; then
    step "Fetching latest release tag..."
    VERSION="$(curl -fsSL "${GITHUB_API}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
    if [[ -z "$VERSION" ]]; then
        error "Failed to fetch latest version from GitHub"
        exit 1
    fi
fi
info "Version: $VERSION"

# ---------- compose download URL ----------
TARBALL="fy-${VERSION#v}-${ASSET_PATTERN}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${TARBALL}"

# ---------- create temp dir ----------
TMPDIR="$(mktemp -d)"
cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

# ---------- download ----------
step "Downloading fy ${VERSION} (${FLAVOR})..."
echo "  ${DOWNLOAD_URL}"

if command -v curl > /dev/null; then
    curl -fSL --progress-bar -o "${TMPDIR}/${TARBALL}" "$DOWNLOAD_URL"
elif command -v wget > /dev/null; then
    wget -q --show-progress -O "${TMPDIR}/${TARBALL}" "$DOWNLOAD_URL"
else
    error "Neither curl nor wget found. Please install one of them."
    exit 1
fi

# ---------- extract (or copy) ----------
step "Extracting..."
# GitHub release assets for this project are raw binaries, not tarballs
# But we keep the variable name for clarity
BINARY_SRC="${TMPDIR}/${TARBALL}"
chmod +x "$BINARY_SRC"

# ---------- install ----------
DEST="${INSTALL_DIR}/${APP_NAME}"

step "Installing to ${DEST}..."
if [[ ! -d "$INSTALL_DIR" ]]; then
    info "Creating ${INSTALL_DIR}..."
    sudo mkdir -p "$INSTALL_DIR"
fi

if [[ -f "$DEST" ]]; then
    warn "Existing ${APP_NAME} found at ${DEST}, overwriting..."
fi

sudo cp "$BINARY_SRC" "$DEST"
sudo chmod 755 "$DEST"

# ---------- verify ----------
step "Verifying installation..."
INSTALLED_VERSION="$("$DEST" -h 2>&1 | head -1 || true)"
info "$INSTALLED_VERSION"
info "${APP_NAME} installed successfully to ${DEST}"

# ---------- PATH check ----------
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    warn "${INSTALL_DIR} is not in your PATH."
    echo "  Add it with:  export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo "  Or add that line to your shell profile (~/.zshrc or ~/.bashrc)."
fi

step "Done! Try it: fy en 你好世界"
