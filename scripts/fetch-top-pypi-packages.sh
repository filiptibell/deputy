#!/usr/bin/env bash

# Fetches the top PyPI packages by download count and writes
# them to the assets file used for autocomplete suggestions.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_FILE="$REPO_DIR/crates/deputy-tools/assets/top-pypi-packages.txt"

SOURCE_URL="https://hugovk.github.io/top-pypi-packages/top-pypi-packages-30-days.json"
MAX_PACKAGES=10000

echo "Fetching top PyPI packages..."
curl -sS "$SOURCE_URL" \
    | jq -r ".rows[:$MAX_PACKAGES][].project" \
    > "$OUTPUT_FILE"

LINE_COUNT=$(wc -l < "$OUTPUT_FILE" | tr -d ' ')
echo "Wrote $LINE_COUNT packages to $(basename "$OUTPUT_FILE")"
