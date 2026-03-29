#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PASS=0
FAIL=0

check() {
    local desc="$1"; shift
    if "$@" >/dev/null 2>&1; then
        echo "PASS: $desc"
        PASS=$((PASS + 1))
    else
        echo "FAIL: $desc"
        FAIL=$((FAIL + 1))
    fi
}

# META-01: Reverse-DNS ID consistency
check "META-01: .desktop has correct ID" \
    grep -q 'Icon=com.cmux_lx.terminal' "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.desktop"
check "META-01: metainfo has correct ID" \
    grep -q '<id>com.cmux_lx.terminal</id>' "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.metainfo.xml"

# META-02: AppStream metainfo validates
check "META-02: appstreamcli validate" \
    appstreamcli validate --no-net "$REPO_ROOT/packaging/desktop/com.cmux_lx.terminal.metainfo.xml"

# META-03: Icons exist at correct sizes
for SIZE in 48 128 256; do
    check "META-03: ${SIZE}px icon exists" \
        test -f "$REPO_ROOT/packaging/icons/hicolor/${SIZE}x${SIZE}/apps/com.cmux_lx.terminal.png"
done

# META-04: Shell completions exist
check "META-04: bash completion" test -f "$REPO_ROOT/packaging/completions/cmux.bash"
check "META-04: zsh completion" test -f "$REPO_ROOT/packaging/completions/_cmux"
check "META-04: fish completion" test -f "$REPO_ROOT/packaging/completions/cmux.fish"

# META-05: Man page exists and renders
check "META-05: man page exists" test -f "$REPO_ROOT/packaging/man/cmux.1"
check "META-05: man page renders" man -l "$REPO_ROOT/packaging/man/cmux.1"

# BUILD-02: Dependency detection script exists and is executable
check "BUILD-02: detect-deps.sh exists" test -x "$REPO_ROOT/packaging/scripts/detect-deps.sh"

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
