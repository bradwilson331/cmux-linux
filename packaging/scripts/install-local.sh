#!/usr/bin/env bash
set -euo pipefail

# install-local.sh -- Install pre-built cmux binaries and desktop assets into a
# prefix. This is the install path for distros without a native package here
# (Arch and friends) and for testing a build without root.
#
# Usage:
#   ./packaging/scripts/install-local.sh                 # installs to ~/.local
#   PREFIX=/usr/local ./packaging/scripts/install-local.sh   # system-wide (needs root)
#
# Build the binaries first:
#   ./scripts/setup-linux.sh && cargo build --release --workspace

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

PREFIX="${PREFIX:-$HOME/.local}"

CMUX_APP="$REPO_ROOT/target/release/cmux-app"
CMUX_CLI="$REPO_ROOT/target/release/cmux"
AGENT_BROWSER="$REPO_ROOT/target/release/agent-browser"

for bin in "$CMUX_APP" "$CMUX_CLI" "$AGENT_BROWSER"; do
    if [ ! -f "$bin" ]; then
        echo "ERROR: missing $bin"
        echo "       Run: ./scripts/setup-linux.sh && cargo build --release --workspace"
        exit 1
    fi
done

echo "==> Installing cmux to $PREFIX"

# Binaries. cmux-app.bin is the real binary; cmux-app is the backend-detecting
# wrapper, which resolves cmux-app.bin next to itself.
install -Dm0755 "$CMUX_APP" "$PREFIX/bin/cmux-app.bin"
install -Dm0755 "$REPO_ROOT/packaging/scripts/cmux-app-wrapper.sh" "$PREFIX/bin/cmux-app"
install -Dm0755 "$CMUX_CLI" "$PREFIX/bin/cmux"

# agent-browser goes next to the cmux binaries so the runtime lookup in
# src/browser.rs finds it (it checks PATH, then the cmux binary's directory,
# then /usr/lib/cmux).
install -Dm0755 "$AGENT_BROWSER" "$PREFIX/bin/agent-browser"

# Desktop metadata
install -Dm0644 "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.desktop" \
    "$PREFIX/share/applications/com.cmux_lx.terminal.desktop"
install -Dm0644 "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.metainfo.xml" \
    "$PREFIX/share/metainfo/com.cmux_lx.terminal.metainfo.xml"

# Icons
for size in 48x48 128x128 256x256; do
    install -Dm0644 "$REPO_ROOT/packaging/icons/hicolor/${size}/apps/com.cmux_lx.terminal.png" \
        "$PREFIX/share/icons/hicolor/${size}/apps/com.cmux_lx.terminal.png"
done

# Shell completions
install -Dm0644 "$REPO_ROOT/packaging/completions/cmux.bash" \
    "$PREFIX/share/bash-completion/completions/cmux"
install -Dm0644 "$REPO_ROOT/packaging/completions/_cmux" \
    "$PREFIX/share/zsh/site-functions/_cmux"
install -Dm0644 "$REPO_ROOT/packaging/completions/cmux.fish" \
    "$PREFIX/share/fish/vendor_completions.d/cmux.fish"

# Man page
install -d "$PREFIX/share/man/man1"
gzip -9n < "$REPO_ROOT/packaging/man/cmux.1" > "$PREFIX/share/man/man1/cmux.1.gz"

# Agent skills and packaged CLAUDE.md
for skill in cmux cmux-browser; do
    find "$REPO_ROOT/skills/$skill" -type f | while IFS= read -r f; do
        rel="${f#"$REPO_ROOT/skills/$skill/"}"
        install -Dm0644 "$f" "$PREFIX/share/cmux/skills/$skill/$rel"
    done
done
install -Dm0644 "$REPO_ROOT/packaging/CLAUDE.md" "$PREFIX/share/cmux/CLAUDE.md"

# Refresh the desktop/icon caches so the launcher entry shows up immediately.
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -qtf "$PREFIX/share/icons/hicolor" 2>/dev/null || true
fi

echo "==> Installed:"
echo "    $PREFIX/bin/cmux          (CLI)"
echo "    $PREFIX/bin/cmux-app      (GUI launcher)"
echo "    $PREFIX/bin/agent-browser (browser automation daemon)"

case ":$PATH:" in
    *":$PREFIX/bin:"*) ;;
    *) echo
       echo "NOTE: $PREFIX/bin is not on your PATH. Add it to use 'cmux' directly." ;;
esac
