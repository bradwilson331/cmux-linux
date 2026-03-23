#!/usr/bin/env bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Install system dependencies required for cargo build (GTK4 dev headers, libclang for bindgen)
echo "==> Checking system dependencies..."
if ! pkg-config --exists gtk4 2>/dev/null; then
    echo "==> Installing GTK4 development headers..."
    if command -v apt-get &>/dev/null; then
        sudo apt-get install -y libgtk-4-dev libclang-dev
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y gtk4-devel clang-devel
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm gtk4 clang
    else
        echo "ERROR: Cannot install GTK4 dev headers automatically."
        echo "Please install: libgtk-4-dev (Debian/Ubuntu) or gtk4-devel (Fedora) or gtk4 (Arch)"
        exit 1
    fi
fi

echo "==> Building libghostty.a from ghostty submodule..."
cd ghostty

# Verify submodule is initialized
if [ ! -f "build.zig" ]; then
    echo "ERROR: ghostty submodule not initialized. Run: git submodule update --init --recursive"
    exit 1
fi

zig build \
    -Dapp-runtime=none \
    -Doptimize=ReleaseFast \
    -Dgtk-x11=true \
    -Dgtk-wayland=true

echo "==> libghostty.a built at: $(pwd)/zig-out/lib/libghostty.a"
ls -lh zig-out/lib/libghostty.a
