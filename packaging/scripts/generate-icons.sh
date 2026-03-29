#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SVG="$REPO_ROOT/resources/cmux.svg"
ICON_DIR="$REPO_ROOT/packaging/icons/hicolor"
APP_ID="com.cmux_lx.terminal"

if command -v rsvg-convert &>/dev/null; then
    CONVERTER="rsvg-convert"
elif command -v inkscape &>/dev/null; then
    CONVERTER="inkscape"
elif command -v convert &>/dev/null; then
    CONVERTER="convert"
else
    echo "ERROR: No SVG converter found. Install one of: librsvg2-bin, inkscape, imagemagick" >&2
    exit 1
fi

for SIZE in 48 128 256; do
    mkdir -p "$ICON_DIR/${SIZE}x${SIZE}/apps"
    OUT="$ICON_DIR/${SIZE}x${SIZE}/apps/${APP_ID}.png"
    case "$CONVERTER" in
        rsvg-convert)
            rsvg-convert -w "$SIZE" -h "$SIZE" "$SVG" -o "$OUT"
            ;;
        inkscape)
            inkscape --export-type=png --export-filename="$OUT" \
                -w "$SIZE" -h "$SIZE" "$SVG" 2>/dev/null
            ;;
        convert)
            convert -background none -resize "${SIZE}x${SIZE}" "$SVG" "$OUT"
            ;;
    esac
    echo "Generated ${SIZE}x${SIZE} icon (using $CONVERTER)"
done
echo "Icons generated in $ICON_DIR"
