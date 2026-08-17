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

# Verify submodule is initialized
if [ ! -f "ghostty/build.zig" ]; then
    echo "ERROR: ghostty submodule not initialized. Run: git submodule update --init --recursive"
    exit 1
fi

# Ghostty pins an exact Zig version and hard-errors at comptime on any other
# version, so we cannot rely on whatever `zig` the distro ships. Read the
# required version out of build.zig.zon and provision it locally if needed.
ZIG_VERSION="$(sed -n 's/.*\.minimum_zig_version = "\([^"]*\)".*/\1/p' ghostty/build.zig.zon)"
if [ -z "$ZIG_VERSION" ]; then
    echo "ERROR: could not read minimum_zig_version from ghostty/build.zig.zon"
    exit 1
fi

ZIG_BIN=""
if command -v zig &>/dev/null && [ "$(zig version)" = "$ZIG_VERSION" ]; then
    ZIG_BIN="$(command -v zig)"
else
    case "$(uname -m)" in
        x86_64)  ZIG_ARCH="x86_64" ;;
        aarch64|arm64) ZIG_ARCH="aarch64" ;;
        *) echo "ERROR: unsupported architecture $(uname -m) for automatic Zig download"; exit 1 ;;
    esac
    ZIG_DIR="$REPO_ROOT/zig-${ZIG_ARCH}-linux-${ZIG_VERSION}"
    if [ ! -x "$ZIG_DIR/zig" ]; then
        ZIG_TARBALL="zig-${ZIG_ARCH}-linux-${ZIG_VERSION}.tar.xz"
        echo "==> Zig $ZIG_VERSION required (found: $(command -v zig &>/dev/null && zig version || echo none))"
        echo "==> Downloading $ZIG_TARBALL..."
        curl -fsSL -o "/tmp/$ZIG_TARBALL" "https://ziglang.org/download/${ZIG_VERSION}/${ZIG_TARBALL}"
        tar -xf "/tmp/$ZIG_TARBALL" -C "$REPO_ROOT"
        rm -f "/tmp/$ZIG_TARBALL"
    fi
    ZIG_BIN="$ZIG_DIR/zig"
fi

echo "==> Using Zig $("$ZIG_BIN" version) at $ZIG_BIN"
cd ghostty

"$ZIG_BIN" build \
    -Dapp-runtime=none \
    -Doptimize=ReleaseFast \
    -Dcpu=baseline \
    -Dgtk-x11=true \
    -Dgtk-wayland=true

# Ghostty installs the embedded static library as `ghostty-internal.a`, but
# build.rs links it as `-lghostty` (which requires the `lib` prefix). Provide
# the conventionally-named alias next to it.
if [ ! -f zig-out/lib/libghostty.a ] && [ -f zig-out/lib/ghostty-internal.a ]; then
    ln -sf ghostty-internal.a zig-out/lib/libghostty.a
fi

if [ ! -f zig-out/lib/libghostty.a ]; then
    echo "ERROR: neither zig-out/lib/libghostty.a nor zig-out/lib/ghostty-internal.a was produced"
    exit 1
fi

echo "==> libghostty.a built at: $(pwd)/zig-out/lib/libghostty.a"
ls -lhL zig-out/lib/libghostty.a

# libghostty.a is a "fat" archive of ghostty's own objects, but it does not
# carry the toolchain/third-party libraries those objects reference:
#
#   libc++ / libc++abi  the vendored glslang, SPIRV-Cross and Dear ImGui are
#                       C++ compiled by Zig against its own bundled libc++
#                       (symbols in the `std::__1::` ABI namespace), not the
#                       system libstdc++.
#   libxml2             the vendored fontconfig's fcxml.o needs it. We use
#                       Zig's build rather than the system libxml2 so it is
#                       guaranteed ABI-compatible with the fontconfig objects
#                       in the archive.
#
# Copy them next to libghostty.a so build.rs can link them from a stable path
# instead of guessing content-hashed cache directories.
echo "==> Collecting the toolchain libraries libghostty.a links against..."
ZIG_GLOBAL_CACHE="$("$ZIG_BIN" env | sed -n 's/.*"global_cache_dir"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
: "${ZIG_GLOBAL_CACHE:=$HOME/.cache/zig}"

# Zig may build both a Debug and a ReleaseFast variant of these during one
# build. They are ABI-identical, so pick the smallest (the optimized one, which
# is not carrying debug info) to keep the resulting binary small.
for lib in c++ c++abi xml2; do
    src="$(find "$ZIG_GLOBAL_CACHE" .zig-cache -name "lib${lib}.a" -printf '%s\t%p\n' 2>/dev/null \
        | sort -n | head -1 | cut -f2)"
    if [ -z "$src" ]; then
        echo "ERROR: could not find lib${lib}.a in the Zig caches."
        echo "       Searched: $ZIG_GLOBAL_CACHE and $(pwd)/.zig-cache"
        echo "       libghostty.a needs it to link. Try removing"
        echo "       ghostty/.zig-cache and re-running this script."
        exit 1
    fi
    cp -f "$src" "zig-out/lib/lib${lib}.a"
    echo "    lib${lib}.a <- $src"
done
