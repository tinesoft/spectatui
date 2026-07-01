#!/bin/sh
# Build and package a spectatui release artifact for the current or specified platform.
#
# Usage:
#   sh scripts/build-artifacts.sh                              # local: auto-detect + build
#   sh scripts/build-artifacts.sh --target x86_64-unknown-linux-gnu --target-dir target  # CI: package only
#
# Options:
#   --target <triple>    Target triple (e.g. x86_64-unknown-linux-gnu).
#                        When provided, cargo build is skipped — binary must already exist.
#   --target-dir <dir>   Cargo target directory (default: dist/target, per .cargo/config.toml).
#                        In CI this is typically 'target'.

set -eu

# ── defaults ──────────────────────────────────────────────────────────────────

TARGET=""
TARGET_DIR="dist/target"

# ── argument parsing ──────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        --target)     TARGET="$2";     shift 2 ;;
        --target-dir) TARGET_DIR="$2"; shift 2 ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 1 ;;
    esac
done

# ── guard: must run from workspace root ──────────────────────────────────────

[ -f "crates/spectatui/Cargo.toml" ] || {
    printf 'error: run this script from the workspace root\n' >&2
    exit 1
}

# ── build step (local mode only) ──────────────────────────────────────────────

if [ -z "$TARGET" ]; then
    # Detect host platform → target triple
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) printf 'error: unsupported architecture: %s\n' "$ARCH" >&2; exit 1 ;;
    esac

    case "$OS" in
        Linux)  TARGET="${ARCH}-unknown-linux-gnu" ;;
        Darwin) TARGET="${ARCH}-apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*)
            TARGET="${ARCH}-pc-windows-msvc"
            ;;
        *) printf 'error: unsupported OS: %s\n' "$OS" >&2; exit 1 ;;
    esac

    printf '» Building release binary for %s...\n' "$TARGET"
    cargo build --release -p spectatui
    BIN_DIR="${TARGET_DIR}/release"
else
    # CI mode: binary already built, locate it under the target triple subdir
    BIN_DIR="${TARGET_DIR}/${TARGET}/release"
fi

# ── locate binary ─────────────────────────────────────────────────────────────

case "$TARGET" in
    *windows*) BIN_NAME="spectatui.exe" ;;
    *)         BIN_NAME="spectatui" ;;
esac

BIN_PATH="${BIN_DIR}/${BIN_NAME}"
[ -f "$BIN_PATH" ] || {
    printf 'error: binary not found at %s\n' "$BIN_PATH" >&2
    exit 1
}

# ── read version ──────────────────────────────────────────────────────────────

VERSION="$(grep '^version' crates/spectatui/Cargo.toml | head -1 | sed 's/.*= *"\(.*\)"/\1/')"

# ── package ───────────────────────────────────────────────────────────────────

OUT_DIR="dist/artifacts"
mkdir -p "$OUT_DIR"

STAGING="${OUT_DIR}/spectatui-v${VERSION}-${TARGET}"
mkdir -p "$STAGING"
cp "$BIN_PATH" "$STAGING/"
cp LICENSE README.md "$STAGING/" 2>/dev/null || true

case "$TARGET" in
    *windows*)
        ARCHIVE="${STAGING}.zip"
        if command -v zip >/dev/null 2>&1; then
            (cd "$OUT_DIR" && zip -r "$(basename "$ARCHIVE")" "$(basename "$STAGING")")
        elif command -v 7z >/dev/null 2>&1; then
            # Git Bash on GitHub windows runners has no zip, but 7z is on PATH.
            # Don't fall back to tar: Git Bash resolves GNU tar, which silently
            # writes a plain tar archive when given a .zip name.
            (cd "$OUT_DIR" && 7z a -tzip "$(basename "$ARCHIVE")" "$(basename "$STAGING")")
        else
            printf 'error: no zip tool found (zip or 7z)\n' >&2
            exit 1
        fi
        ;;
    *)
        ARCHIVE="${STAGING}.tar.gz"
        tar czf "$ARCHIVE" -C "$OUT_DIR" "$(basename "$STAGING")"
        ;;
esac

rm -rf "$STAGING"

# ── done ─────────────────────────────────────────────────────────────────────

printf '\n✓ artifact: %s\n' "$ARCHIVE"
