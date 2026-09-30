#!/bin/bash
# Gate G1: git is usable and identified

set -euo pipefail

if ! git version >/dev/null 2>&1; then
    echo "Gate G1 FAILED - git version failed" >&2
    exit 1
fi

VERSION=$(git version)
echo "Git version: $VERSION"

# Try to create a temp repo
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

cd "$TEMP_DIR"
if ! git init >/dev/null 2>&1; then
    echo "Gate G1 FAILED - git init failed" >&2
    exit 1
fi

echo "Gate G1 PASSED - Git is usable and identified"
exit 0