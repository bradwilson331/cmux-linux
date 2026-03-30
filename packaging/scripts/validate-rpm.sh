#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Ensure rpm command is available
command -v rpm &>/dev/null || {
    echo "ERROR: rpm not found. Install with: sudo apt install rpm" >&2
    exit 1
}

# Accept optional path to .rpm, default to dist/cmux-*.rpm
if [[ -n "${1:-}" ]]; then
    RPM="$1"
else
    RPMS=("$REPO_ROOT"/dist/cmux-*.rpm)
    if [[ ${#RPMS[@]} -eq 0 || ! -f "${RPMS[0]}" ]]; then
        echo "ERROR: No .rpm found in dist/. Build first or pass path as argument." >&2
        exit 1
    fi
    if [[ ${#RPMS[@]} -gt 1 ]]; then
        echo "ERROR: Multiple .rpm files in dist/. Pass explicit path." >&2
        exit 1
    fi
    RPM="${RPMS[0]}"
fi

if [[ ! -f "$RPM" ]]; then
    echo "ERROR: RPM file not found: $RPM" >&2
    exit 1
fi

echo "Validating: $RPM"
echo ""

PASS=0
FAIL=0

check() {
    local desc="$1"; shift
    if "$@" >/dev/null 2>&1; then
        echo "  PASS: $desc"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: $desc"
        FAIL=$((FAIL + 1))
    fi
}

# Cache query outputs to temp files for grep compatibility
FILE_LIST_FILE=$(mktemp)
INFO_FILE=$(mktemp)
REQUIRES_FILE=$(mktemp)
trap 'rm -f "$FILE_LIST_FILE" "$INFO_FILE" "$REQUIRES_FILE"' EXIT

rpm -qpl "$RPM" > "$FILE_LIST_FILE" 2>/dev/null
rpm -qpi "$RPM" > "$INFO_FILE" 2>/dev/null
rpm -qpR "$RPM" > "$REQUIRES_FILE" 2>/dev/null

# --- File listing checks (RPM-03) ---
echo "File listing:"
check "/usr/bin/cmux-app" grep -q '/usr/bin/cmux-app' "$FILE_LIST_FILE"
check "/usr/bin/cmux (exact)" grep -qx '/usr/bin/cmux' "$FILE_LIST_FILE"
check "/usr/lib64/cmux/cmuxd-remote" grep -q '/usr/lib64/cmux/cmuxd-remote' "$FILE_LIST_FILE"
check "/usr/lib64/cmux/agent-browser" grep -q '/usr/lib64/cmux/agent-browser' "$FILE_LIST_FILE"
check ".desktop file" grep -q '/usr/share/applications/com.cmux_lx.terminal.desktop' "$FILE_LIST_FILE"
check "metainfo xml" grep -q '/usr/share/metainfo/com.cmux_lx.terminal.metainfo.xml' "$FILE_LIST_FILE"
check "icon 48x48" grep -q '/usr/share/icons/hicolor/48x48/apps/com.cmux_lx.terminal.png' "$FILE_LIST_FILE"
check "icon 128x128" grep -q '/usr/share/icons/hicolor/128x128/apps/com.cmux_lx.terminal.png' "$FILE_LIST_FILE"
check "icon 256x256" grep -q '/usr/share/icons/hicolor/256x256/apps/com.cmux_lx.terminal.png' "$FILE_LIST_FILE"
check "bash completion" grep -q '/usr/share/bash-completion/completions/cmux' "$FILE_LIST_FILE"
check "zsh completion" grep -q '/usr/share/zsh/vendor-completions/_cmux' "$FILE_LIST_FILE"
check "fish completion" grep -q '/usr/share/fish/vendor_completions.d/cmux.fish' "$FILE_LIST_FILE"
check "man page" grep -q '/usr/share/man/man1/cmux.1.gz' "$FILE_LIST_FILE"

# --- Skills & CLAUDE.md checks (Phase 12.1) ---
echo ""
echo "Skills:"
check "cmux skill SKILL.md" grep -q '/usr/share/cmux/skills/cmux/SKILL.md' "$FILE_LIST_FILE"
check "cmux-browser skill SKILL.md" grep -q '/usr/share/cmux/skills/cmux-browser/SKILL.md' "$FILE_LIST_FILE"
check "cmux-browser commands.md" grep -q '/usr/share/cmux/skills/cmux-browser/references/commands.md' "$FILE_LIST_FILE"
check "CLAUDE.md" grep -q '/usr/share/cmux/CLAUDE.md' "$FILE_LIST_FILE"
check "no cmux-debug-windows (D-13)" bash -c '! grep -q "cmux-debug-windows" "'"$FILE_LIST_FILE"'"'
check "no release skill (D-13)" bash -c '! grep -q "skills/release" "'"$FILE_LIST_FILE"'"'

# --- Metadata checks (RPM-01, RPM-02) ---
echo ""
echo "Metadata:"
check "Name is cmux" grep -q '^Name.*: cmux$' "$INFO_FILE"
check "Architecture is x86_64" grep -q 'Architecture.*x86_64' "$INFO_FILE"

# --- Dependency checks (RPM-02) ---
echo ""
echo "Dependencies:"
for dep in gtk4 fontconfig freetype oniguruma mesa-libGL harfbuzz glib2 cairo pango; do
    check "Requires $dep" grep -q "$dep" "$REQUIRES_FILE"
done

echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]]
