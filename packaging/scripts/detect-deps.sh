#!/usr/bin/env bash
set -euo pipefail

# detect-deps.sh -- Map runtime shared library dependencies to distro package names
# Usage: ./detect-deps.sh [--json] <binary>
# Output: TSV table (library, debian_pkg, fedora_pkg) + optional JSON (--json flag)

JSON_MODE=false

if [[ "${1:-}" == "--json" ]]; then
    JSON_MODE=true
    shift
fi

BINARY="${1:?Usage: detect-deps.sh [--json] <binary>}"

if [[ ! -f "$BINARY" ]]; then
    echo "ERROR: Binary not found: $BINARY" >&2
    exit 1
fi

# Static fallback: lib soname -> Fedora package name
# Used when rpm is not available (e.g., running on Debian/Ubuntu)
declare -A FEDORA_FALLBACK=(
    ["libgtk-4.so.1"]="gtk4"
    ["libfontconfig.so.1"]="fontconfig"
    ["libfreetype.so.6"]="freetype"
    ["libonig.so.5"]="oniguruma"
    ["libGL.so.1"]="mesa-libGL"
    ["libEGL.so.1"]="mesa-libEGL"
    ["libharfbuzz.so.0"]="harfbuzz"
    ["libgio-2.0.so.0"]="glib2"
    ["libgobject-2.0.so.0"]="glib2"
    ["libglib-2.0.so.0"]="glib2"
    ["libcairo.so.2"]="cairo"
    ["libcairo-gobject.so.2"]="cairo-gobject"
    ["libpango-1.0.so.0"]="pango"
    ["libpangocairo-1.0.so.0"]="pango"
    ["libpangoft2-1.0.so.0"]="pango"
    ["libgdk_pixbuf-2.0.so.0"]="gdk-pixbuf2"
    ["libepoxy.so.0"]="libepoxy"
    ["libX11.so.6"]="libX11"
    ["libXi.so.6"]="libXi"
    ["libXrandr.so.2"]="libXrandr"
    ["libXcursor.so.1"]="libXcursor"
    ["libwayland-client.so.0"]="wayland-devel"
    ["libxkbcommon.so.0"]="libxkbcommon"
    ["libgraphene-1.0.so.0"]="graphene"
    ["libvulkan.so.1"]="vulkan-loader"
    ["libfribidi.so.0"]="fribidi"
    ["libpixman-1.so.0"]="pixman"
    ["libpng16.so.16"]="libpng"
    ["libz.so.1"]="zlib"
)

# Static fallback: lib soname -> Debian package name
# Used when dpkg is not available (e.g., running on Fedora)
declare -A DEBIAN_FALLBACK=(
    ["libgtk-4.so.1"]="libgtk-4-1"
    ["libfontconfig.so.1"]="libfontconfig1"
    ["libfreetype.so.6"]="libfreetype6"
    ["libonig.so.5"]="libonig5"
    ["libGL.so.1"]="libgl1"
    ["libEGL.so.1"]="libegl1"
    ["libharfbuzz.so.0"]="libharfbuzz0b"
    ["libgio-2.0.so.0"]="libglib2.0-0"
    ["libgobject-2.0.so.0"]="libglib2.0-0"
    ["libglib-2.0.so.0"]="libglib2.0-0"
    ["libcairo.so.2"]="libcairo2"
    ["libcairo-gobject.so.2"]="libcairo-gobject2"
    ["libpango-1.0.so.0"]="libpango-1.0-0"
    ["libpangocairo-1.0.so.0"]="libpangocairo-1.0-0"
    ["libpangoft2-1.0.so.0"]="libpangoft2-1.0-0"
    ["libgdk_pixbuf-2.0.so.0"]="libgdk-pixbuf-2.0-0"
    ["libepoxy.so.0"]="libepoxy0"
    ["libX11.so.6"]="libx11-6"
    ["libxkbcommon.so.0"]="libxkbcommon0"
    ["libgraphene-1.0.so.0"]="libgraphene-1.0-0"
    ["libvulkan.so.1"]="libvulkan1"
    ["libfribidi.so.0"]="libfribidi0"
    ["libpng16.so.16"]="libpng16-16"
    ["libz.so.1"]="zlib1g"
)

# System libs to skip (always present, not package dependencies)
SKIP_PATTERN='^(libc\.so|libm\.so|libpthread|libdl|librt\.so|linux-vdso|ld-linux|libgcc_s|libstdc\+\+)'

HAS_DPKG=false
HAS_RPM=false
command -v dpkg &>/dev/null && HAS_DPKG=true
command -v rpm &>/dev/null && HAS_RPM=true

# Collect unique library sonames from ldd
mapfile -t LIBS < <(ldd "$BINARY" 2>/dev/null | awk '/=>/ {print $1}' | grep -v -E "$SKIP_PATTERN" | sort -u)

if $JSON_MODE; then
    echo "["
fi

FIRST=true
for lib in "${LIBS[@]}"; do
    # Debian package resolution
    if $HAS_DPKG; then
        deb_pkg=$(dpkg -S "*/$lib" 2>/dev/null | head -1 | cut -d: -f1) || true
        deb_pkg="${deb_pkg:-${DEBIAN_FALLBACK[$lib]:-UNKNOWN}}"
    else
        deb_pkg="${DEBIAN_FALLBACK[$lib]:-UNKNOWN}"
    fi

    # Fedora package resolution
    if $HAS_RPM; then
        lib_path=$(ldconfig -p 2>/dev/null | grep "$lib" | awk '{print $NF}' | head -1) || true
        if [[ -n "$lib_path" ]]; then
            fed_pkg=$(rpm -qf "$lib_path" 2>/dev/null | head -1) || true
            fed_pkg="${fed_pkg:-${FEDORA_FALLBACK[$lib]:-UNKNOWN}}"
        else
            fed_pkg="${FEDORA_FALLBACK[$lib]:-UNKNOWN}"
        fi
    else
        fed_pkg="${FEDORA_FALLBACK[$lib]:-UNKNOWN}"
    fi

    if $JSON_MODE; then
        $FIRST || echo ","
        FIRST=false
        printf '  {"library": "%s", "debian": "%s", "fedora": "%s"}' "$lib" "$deb_pkg" "$fed_pkg"
    else
        printf '%s\t%s\t%s\n' "$lib" "$deb_pkg" "$fed_pkg"
    fi
done

if $JSON_MODE; then
    echo ""
    echo "]"
else
    # Summary: unique package names
    echo ""
    echo "--- Debian packages (unique) ---"
    ldd "$BINARY" 2>/dev/null | awk '/=>/ {print $1}' | grep -v -E "$SKIP_PATTERN" | sort -u | while read -r lib; do
        if $HAS_DPKG; then
            dpkg -S "*/$lib" 2>/dev/null | head -1 | cut -d: -f1
        else
            echo "${DEBIAN_FALLBACK[$lib]:-UNKNOWN}"
        fi
    done | sort -u

    echo ""
    echo "--- Fedora packages (unique) ---"
    ldd "$BINARY" 2>/dev/null | awk '/=>/ {print $1}' | grep -v -E "$SKIP_PATTERN" | sort -u | while read -r lib; do
        echo "${FEDORA_FALLBACK[$lib]:-UNKNOWN}"
    done | sort -u
fi
