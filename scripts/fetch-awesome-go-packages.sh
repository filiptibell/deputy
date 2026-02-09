#!/usr/bin/env bash

# Fetches Go packages from the awesome-go curated list and writes
# them to the assets file used for autocomplete suggestions.
#
# Parses the README as structured markdown — tracking section headers
# to determine the category of each entry — then filters out non-library
# categories (tutorials, conferences, e-books, etc.) and non-code-host
# URLs, keeping only importable Go module paths.
#
# Output format: path:name:"description"
#
# Requires: curl, python3

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_FILE="$REPO_DIR/crates/deputy-tools/assets/top-go-packages.txt"

SOURCE_URL="https://raw.githubusercontent.com/avelino/awesome-go/main/README.md"

echo "Fetching awesome-go package list..."

curl -sS --fail "$SOURCE_URL" \
    | python3 -c "
import re
import sys

# Categories that are not importable Go libraries
EXCLUDE_CATEGORIES = {
    'Benchmarks',
    'Conferences',
    'Contents',
    'Contribution',
    'E-books for purchase',
    'Free e-books',
    'Gophers',
    'Guided Learning',
    'License',
    'Meetups',
    'Social Media',
    'Style Guides',
    'Tutorials',
    'Websites',
}

# Code hosting domains from which Go module paths can be derived
CODE_HOSTS = {
    'github.com':    3,  # github.com/owner/repo
    'gitlab.com':    0,  # gitlab.com/group/.../repo (variable depth)
    'codeberg.org':  3,  # codeberg.org/owner/repo
    'bitbucket.org': 3,  # bitbucket.org/owner/repo
    'git.sr.ht':     3,  # git.sr.ht/~user/repo
}

ENTRY_RE = re.compile(
    r'^\s*-\s+\[([^\]]+)\]\(https?://([^)]+)\)\s*-\s*(.+?)\s*$'
)

category = ''
seen_paths = set()
entries = []

for line in sys.stdin:
    line = line.rstrip()

    # Track current category from section headers
    m = re.match(r'^##\s+(.+)', line)
    if m:
        category = m.group(1).strip()
        continue
    m = re.match(r'^###\s+(.+)', line)
    if m:
        category = m.group(1).strip()
        continue

    # Skip excluded categories
    if category in EXCLUDE_CATEGORIES:
        continue

    # Match package entries
    m = ENTRY_RE.match(line)
    if not m:
        continue

    name, url, desc = m.group(1), m.group(2), m.group(3)
    desc = desc.rstrip('. ')

    # Extract module path from code hosting URL
    path = None
    for host, segment_count in CODE_HOSTS.items():
        if not url.startswith(host + '/'):
            continue
        if segment_count == 0:
            # Variable depth (GitLab) — use full path
            path = url.rstrip('/')
        else:
            # Fixed depth — take exactly N segments
            parts = url.split('/')
            if len(parts) >= segment_count:
                path = '/'.join(parts[:segment_count])
        break

    if path is None:
        continue

    # Truncate deep paths (e.g. /tree/main/pkg/...) to owner/repo
    path = re.sub(r'/(tree|blob|src)/.*', '', path)
    path = path.rstrip('/')

    # Deduplicate by module path (keep first occurrence)
    if path in seen_paths:
        continue
    seen_paths.add(path)

    entries.append((path, name, desc))

# Sort by module path for stable, diff-friendly output
entries.sort(key=lambda e: e[0].lower())

for path, name, desc in entries:
    print(f'{path}:{name}:\"{desc}\"')
" \
    > "$OUTPUT_FILE"

LINE_COUNT=$(wc -l < "$OUTPUT_FILE" | tr -d ' ')
echo "Wrote $LINE_COUNT packages to $(basename "$OUTPUT_FILE")"
