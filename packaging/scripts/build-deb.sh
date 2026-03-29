#!/usr/bin/env bash
set -euo pipefail

# build-deb.sh -- Build a .deb package from pre-built cmux binaries
# Usage: ./build-deb.sh [cmux-app] [cmux-cli] [cmuxd-remote]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Binary paths (positional args with defaults)
CMUX_APP="${1:-$REPO_ROOT/target/release/cmux-app}"
CMUX_CLI="${2:-$REPO_ROOT/target/release/cmux}"
CMUXD_REMOTE="${3:-$REPO_ROOT/daemon/remote/cmuxd-remote}"

# Extract version from Cargo.toml
VERSION=$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

# Verify all binaries exist
for bin in "$CMUX_APP" "$CMUX_CLI" "$CMUXD_REMOTE"; do
    if [[ ! -f "$bin" ]]; then
        echo "ERROR: Binary not found: $bin" >&2
        exit 1
    fi
done

OUTPUT_DIR="${REPO_ROOT}/dist"
mkdir -p "$OUTPUT_DIR"

# Create staging directory with cleanup trap
PKG_ROOT=$(mktemp -d)
trap 'rm -rf "$PKG_ROOT"' EXIT

# Install binaries
install -Dm0755 "$CMUX_APP" "$PKG_ROOT/usr/bin/cmux-app"
install -Dm0755 "$CMUX_CLI" "$PKG_ROOT/usr/bin/cmux"
install -Dm0755 "$CMUXD_REMOTE" "$PKG_ROOT/usr/lib/cmux/cmuxd-remote"

# Desktop metadata
install -Dm0644 "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.desktop" \
    "$PKG_ROOT/usr/share/applications/com.cmux_lx.terminal.desktop"
install -Dm0644 "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.metainfo.xml" \
    "$PKG_ROOT/usr/share/metainfo/com.cmux_lx.terminal.metainfo.xml"

# Icons
for size in 48x48 128x128 256x256; do
    install -Dm0644 "$REPO_ROOT/packaging/icons/hicolor/${size}/apps/com.cmux_lx.terminal.png" \
        "$PKG_ROOT/usr/share/icons/hicolor/${size}/apps/com.cmux_lx.terminal.png"
done

# Shell completions
install -Dm0644 "$REPO_ROOT/packaging/completions/cmux.bash" \
    "$PKG_ROOT/usr/share/bash-completion/completions/cmux"
install -Dm0644 "$REPO_ROOT/packaging/completions/_cmux" \
    "$PKG_ROOT/usr/share/zsh/vendor-completions/_cmux"
install -Dm0644 "$REPO_ROOT/packaging/completions/cmux.fish" \
    "$PKG_ROOT/usr/share/fish/vendor_completions.d/cmux.fish"

# Man page (gzipped)
mkdir -p "$PKG_ROOT/usr/share/man/man1"
gzip -9n < "$REPO_ROOT/packaging/man/cmux.1" > "$PKG_ROOT/usr/share/man/man1/cmux.1.gz"

# DEBIAN/control
mkdir -p "$PKG_ROOT/DEBIAN"
cat > "$PKG_ROOT/DEBIAN/control" << CTRL
Package: cmux
Version: ${VERSION}
Architecture: amd64
Maintainer: cmux <noreply@cmux.dev>
Section: x11
Priority: optional
Depends: libgtk-4-1, libfontconfig1, libfreetype6, libonig5, libgl1, libegl1, libharfbuzz0b, libglib2.0-0, libcairo2, libpango-1.0-0, libpangocairo-1.0-0, libpangoft2-1.0-0, libepoxy0, libxkbcommon0, libgraphene-1.0-0
Homepage: https://cmux.dev
Description: GPU-accelerated terminal multiplexer
 cmux provides tabs, splits, workspaces, and socket CLI control
 powered by Ghostty's GPU-accelerated terminal rendering.
CTRL

# Build the .deb
DEB_FILE="$OUTPUT_DIR/cmux_${VERSION}_amd64.deb"
dpkg-deb --build --root-owner-group "$PKG_ROOT" "$DEB_FILE"
echo "Built: $DEB_FILE"
