#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Ensure rpmbuild is available
command -v rpmbuild &>/dev/null || {
    echo "ERROR: rpmbuild not found. Install with: sudo apt install rpm" >&2
    exit 1
}

# Binary paths (override via positional args or environment)
CMUX_APP="${1:-${REPO_ROOT}/target/release/cmux-app}"
CMUX_CLI="${2:-${REPO_ROOT}/target/release/cmux}"
CMUXD_REMOTE="${3:-${REPO_ROOT}/daemon/remote/cmuxd-remote}"
AGENT_BROWSER="${4:-${REPO_ROOT}/target/release/agent-browser}"

# Extract version from Cargo.toml
VERSION=$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

# Verify binaries exist
for bin in "$CMUX_APP" "$CMUX_CLI" "$CMUXD_REMOTE" "$AGENT_BROWSER"; do
    if [[ ! -f "$bin" ]]; then
        echo "ERROR: Binary not found: $bin" >&2
        exit 1
    fi
done

OUTPUT_DIR="${REPO_ROOT}/dist"
mkdir -p "$OUTPUT_DIR"

# Create temporary build directory
BUILD_DIR=$(mktemp -d)
trap 'rm -rf "$BUILD_DIR"' EXIT

STAGING="$BUILD_DIR/SOURCES"
mkdir -p "$STAGING/icons"
mkdir -p "$BUILD_DIR/rpmbuild"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Stage binaries
cp "$CMUX_APP" "$STAGING/cmux-app"
cp "$CMUX_CLI" "$STAGING/cmux"
cp "$CMUXD_REMOTE" "$STAGING/cmuxd-remote"
cp "$AGENT_BROWSER" "$STAGING/agent-browser"

# Desktop metadata
cp "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.desktop" "$STAGING/"
cp "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.metainfo.xml" "$STAGING/"

# Icons (flatten to simple names for spec)
cp "$REPO_ROOT/packaging/icons/hicolor/48x48/apps/com.cmux_lx.terminal.png" "$STAGING/icons/48x48.png"
cp "$REPO_ROOT/packaging/icons/hicolor/128x128/apps/com.cmux_lx.terminal.png" "$STAGING/icons/128x128.png"
cp "$REPO_ROOT/packaging/icons/hicolor/256x256/apps/com.cmux_lx.terminal.png" "$STAGING/icons/256x256.png"

# Shell completions
cp "$REPO_ROOT/packaging/completions/cmux.bash" "$STAGING/"
cp "$REPO_ROOT/packaging/completions/_cmux" "$STAGING/"
cp "$REPO_ROOT/packaging/completions/cmux.fish" "$STAGING/"

# Man page (gzipped)
gzip -9n < "$REPO_ROOT/packaging/man/cmux.1" > "$STAGING/cmux.1.gz"

# Skills (D-13: only cmux and cmux-browser)
for skill in cmux cmux-browser; do
    cp -r "$REPO_ROOT/skills/$skill" "$STAGING/skills-$skill"
done

# Package CLAUDE.md (D-14)
cp "$REPO_ROOT/packaging/CLAUDE.md" "$STAGING/CLAUDE.md"

# Build the RPM
rpmbuild -bb \
    --define "_cmux_version ${VERSION}" \
    --define "_sourcedir ${STAGING}" \
    --define "_topdir ${BUILD_DIR}/rpmbuild" \
    "$REPO_ROOT/packaging/rpm/cmux.spec"

# Copy output to dist/
RPM_FILE=$(find "$BUILD_DIR/rpmbuild/RPMS" -name "*.rpm" | head -1)
if [[ -z "$RPM_FILE" ]]; then
    echo "ERROR: rpmbuild produced no output" >&2
    exit 1
fi
cp "$RPM_FILE" "$OUTPUT_DIR/"
echo "Built: $OUTPUT_DIR/$(basename "$RPM_FILE")"
