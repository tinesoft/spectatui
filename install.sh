#!/bin/sh
# spectatui installer — https://github.com/tinesoft/spectatui
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/tinesoft/spectatui/main/install.sh | sh
#   curl -fsSL ... | sh -s -- --version 1.0.0
#   curl -fsSL ... | sh -s -- --to /usr/local/bin

set -eu

REPO="tinesoft/spectatui"
BINARY="spectatui"

# ── helpers ──────────────────────────────────────────────────────────────────

die()  { printf '\033[0;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
info() { printf '\033[0;32m  >\033[0m %s\n' "$*"; }

need() {
    command -v "$1" >/dev/null 2>&1 || die "required tool not found: $1"
}

# ── argument parsing ──────────────────────────────────────────────────────────

VERSION=""
INSTALL_DIR=""

while [ $# -gt 0 ]; do
    case "$1" in
        --version) VERSION="$2"; shift 2 ;;
        --to)      INSTALL_DIR="$2"; shift 2 ;;
        --)        shift; break ;;
        -*)        die "unknown option: $1" ;;
        *)         break ;;
    esac
done

# ── platform detection ────────────────────────────────────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  ;;
    Darwin) ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
        die "Windows is not supported by this script. Download the .zip from https://github.com/$REPO/releases or run: cargo install $BINARY"
        ;;
    *)
        die "unsupported OS: $OS"
        ;;
esac

case "$ARCH" in
    x86_64|amd64)  ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)             die "unsupported architecture: $ARCH" ;;
esac

case "$OS" in
    Linux)  TARGET="${ARCH}-unknown-linux-gnu"; EXT="tar.gz" ;;
    Darwin) TARGET="${ARCH}-apple-darwin";      EXT="tar.gz" ;;
esac

# ── resolve version ───────────────────────────────────────────────────────────

need curl

if [ -z "$VERSION" ]; then
    info "Fetching latest release..."
    # POSIX BRE only — GNU-isms like \? are literals in BSD (macOS) sed
    VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
    [ -n "$VERSION" ] || die "could not determine latest version"
fi
VERSION="${VERSION#v}"  # accept both v1.0.0 and 1.0.0
case "$VERSION" in
    *[!0-9A-Za-z.+-]*|'') die "could not parse release version: $VERSION" ;;
esac

info "Installing $BINARY v$VERSION for $TARGET"

# ── download & verify ─────────────────────────────────────────────────────────

ARCHIVE="${BINARY}-v${VERSION}-${TARGET}.${EXT}"
BASE_URL="https://github.com/$REPO/releases/download/v${VERSION}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$BASE_URL/$ARCHIVE"        -o "$TMP/$ARCHIVE"
curl -fsSL "$BASE_URL/sha256sums.txt"  -o "$TMP/sha256sums.txt"

info "Verifying checksum..."
cd "$TMP"
sums_line="$(grep -F "$ARCHIVE" sha256sums.txt || true)"
[ -n "$sums_line" ] || die "no checksum entry for $ARCHIVE"
if command -v sha256sum >/dev/null 2>&1; then
    printf '%s\n' "$sums_line" | sha256sum -c -
elif command -v shasum >/dev/null 2>&1; then
    printf '%s\n' "$sums_line" | shasum -a 256 -c -
else
    die "no checksum tool found (sha256sum or shasum)"
fi
cd - >/dev/null

# ── extract ───────────────────────────────────────────────────────────────────

info "Extracting..."
tar xzf "$TMP/$ARCHIVE" -C "$TMP"
BIN_PATH="$(find "$TMP" -name "$BINARY" -type f | head -1)"
[ -n "$BIN_PATH" ] || die "binary not found in archive"
chmod +x "$BIN_PATH"

# ── choose install dir ────────────────────────────────────────────────────────

if [ -z "$INSTALL_DIR" ]; then
    if [ -n "${CARGO_HOME:-}" ] && [ -d "$CARGO_HOME/bin" ]; then
        INSTALL_DIR="$CARGO_HOME/bin"
    elif [ -d "$HOME/.cargo/bin" ]; then
        INSTALL_DIR="$HOME/.cargo/bin"
    elif [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        INSTALL_DIR="/usr/local/bin"
    fi
fi

# ── install ───────────────────────────────────────────────────────────────────

if [ -w "$INSTALL_DIR" ]; then
    cp "$BIN_PATH" "$INSTALL_DIR/$BINARY"
else
    info "Requesting sudo to write to $INSTALL_DIR"
    sudo cp "$BIN_PATH" "$INSTALL_DIR/$BINARY"
fi

info "Installed to $INSTALL_DIR/$BINARY"
printf '\n\033[0;32m%s v%s installed!\033[0m Run: %s\n\n' "$BINARY" "$VERSION" "$BINARY"
