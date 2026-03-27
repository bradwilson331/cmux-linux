#!/usr/bin/env bash
set -euo pipefail
# Build and install cmuxd-remote to the XDG data path for local development.
# Release builds use scripts/build_remote_daemon_release_assets.sh instead.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DAEMON_SRC="${REPO_ROOT}/daemon/remote"

if ! command -v go >/dev/null 2>&1; then
    echo "error: Go is required. Install from https://go.dev/dl/" >&2
    exit 1
fi

DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}"
INSTALL_DIR="${DATA_DIR}/cmux/bin"
mkdir -p "$INSTALL_DIR"

# Detect host architecture
GOARCH="$(go env GOARCH)"
GOOS="$(go env GOOS)"
OUTPUT="${INSTALL_DIR}/cmuxd-remote-${GOOS}-${GOARCH}"

echo "Building cmuxd-remote for ${GOOS}/${GOARCH}..."
(cd "$DAEMON_SRC" && CGO_ENABLED=0 go build -trimpath -o "$OUTPUT" ./cmd/cmuxd-remote)
chmod 755 "$OUTPUT"
echo "Installed: ${OUTPUT}"
