#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ASSETS_DIR="$ROOT_DIR/src-tauri/assets"

mkdir -p "$ASSETS_DIR"

PSTREAM_URL="https://raw.githubusercontent.com/xp-technologies-dev/Userscript/main/p-stream.user.js"
PSTREAM_OUT="$ASSETS_DIR/p-stream.user.js"

echo "Fetching P-Stream userscript..."
curl -fsSL "$PSTREAM_URL" -o "$PSTREAM_OUT"
echo "  -> $PSTREAM_OUT ($(wc -c < "$PSTREAM_OUT" | tr -d ' ') bytes)"

echo "Done. Desktop assets in $ASSETS_DIR"
