#!/usr/bin/env bash
set -euo pipefail

# validate-deb.sh -- Validate a built .deb package for correct structure
# Usage: ./validate-deb.sh [path-to-deb]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Find the .deb file
if [[ -n "${1:-}" ]]; then
    DEB="$1"
else
    # shellcheck disable=SC2012
    DEB=$(ls -t "$REPO_ROOT"/dist/cmux_*_amd64.deb 2>/dev/null | head -1)
    if [[ -z "$DEB" ]]; then
        echo "ERROR: No .deb file found in dist/" >&2
        exit 1
    fi
fi

if [[ ! -f "$DEB" ]]; then
    echo "ERROR: .deb file not found: $DEB" >&2
    exit 1
fi

echo "Validating: $DEB"
echo ""

PASS=0
FAIL=0

check() {
    local desc="$1" cmd="$2"
    if eval "$cmd" &>/dev/null; then
        echo "  PASS: $desc"
        ((PASS++))
    else
        echo "  FAIL: $desc"
        ((FAIL++))
    fi
}

# Cache file listing and control output
FILE_LIST=$(dpkg-deb -c "$DEB")
CONTROL=$(dpkg-deb -f "$DEB")

# --- File listing checks ---
echo "File listing:"

check "usr/bin/cmux-app exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/bin/cmux-app"'

# Anchored match to avoid matching cmux-app
check "usr/bin/cmux exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/bin/cmux[[:space:]]"'

check "usr/lib/cmux/cmuxd-remote exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/lib/cmux/cmuxd-remote"'

check "usr/lib/cmux/agent-browser exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/lib/cmux/agent-browser"'

check "desktop entry exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/applications/com.cmux_lx.terminal.desktop"'

check "metainfo exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/metainfo/com.cmux_lx.terminal.metainfo.xml"'

check "48x48 icon exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/icons/hicolor/48x48/apps/com.cmux_lx.terminal.png"'

check "128x128 icon exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/icons/hicolor/128x128/apps/com.cmux_lx.terminal.png"'

check "256x256 icon exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/icons/hicolor/256x256/apps/com.cmux_lx.terminal.png"'

check "bash completion exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/bash-completion/completions/cmux"'

check "zsh completion exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/zsh/vendor-completions/_cmux"'

check "fish completion exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/fish/vendor_completions.d/cmux.fish"'

check "man page exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/man/man1/cmux.1.gz"'

# --- Skills & CLAUDE.md checks (Phase 12.1) ---
echo ""
echo "Skills:"

check "cmux skill SKILL.md exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/cmux/skills/cmux/SKILL.md"'

check "cmux-browser skill SKILL.md exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/cmux/skills/cmux-browser/SKILL.md"'

check "cmux-browser commands.md exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/cmux/skills/cmux-browser/references/commands.md"'

check "CLAUDE.md exists" \
    'echo "$FILE_LIST" | grep -q "\./usr/share/cmux/CLAUDE.md"'

check "no cmux-debug-windows skill packaged (D-13)" \
    '! echo "$FILE_LIST" | grep -q "cmux-debug-windows"'

check "no release skill packaged (D-13)" \
    '! echo "$FILE_LIST" | grep -q "skills/release"'

# --- Metadata checks ---
echo ""
echo "Metadata:"

check "Package: cmux" \
    'echo "$CONTROL" | grep -q "^Package: cmux$"'

check "Architecture: amd64" \
    'echo "$CONTROL" | grep -q "^Architecture: amd64$"'

check "Version is non-empty" \
    'echo "$CONTROL" | grep -qP "^Version: .+"'

check "Depends contains libgtk-4-1" \
    'echo "$CONTROL" | grep "^Depends:" | grep -q "libgtk-4-1"'

check "Depends contains libfontconfig1" \
    'echo "$CONTROL" | grep "^Depends:" | grep -q "libfontconfig1"'

check "Depends contains libfreetype6" \
    'echo "$CONTROL" | grep "^Depends:" | grep -q "libfreetype6"'

check "Depends contains libonig5" \
    'echo "$CONTROL" | grep "^Depends:" | grep -q "libonig5"'

check "Depends contains libgl1" \
    'echo "$CONTROL" | grep "^Depends:" | grep -q "libgl1"'

check "Depends contains libharfbuzz0b" \
    'echo "$CONTROL" | grep "^Depends:" | grep -q "libharfbuzz0b"'

check "Depends contains libglib2.0-0" \
    'echo "$CONTROL" | grep "^Depends:" | grep -q "libglib2.0-0"'

check "Depends contains libcairo2" \
    'echo "$CONTROL" | grep "^Depends:" | grep -q "libcairo2"'

echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]]
