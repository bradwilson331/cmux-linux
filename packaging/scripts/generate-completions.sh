#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building cmux-generate..."
cd "$REPO_ROOT"
cargo build --bin cmux-generate

echo "Running generator..."
cargo run --bin cmux-generate

echo "Generated files:"
ls -la packaging/completions/
ls -la packaging/man/
echo "Done. Completions and man page updated."
