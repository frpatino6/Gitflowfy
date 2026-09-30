#!/bin/bash
# Gate G5: No embedded Git implementation
# cargo tree --all-features must contain no libgit2, git2, gix, or jgit

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

OUTPUT=$(cargo tree --all-features --prefix=none 2>&1)
if [[ $? -ne 0 ]]; then
    echo "cargo tree failed: $OUTPUT" >&2
    exit 1
fi

FORBIDDEN=("libgit2" "git2" "gix" "jgit")
FOUND=()

while IFS= read -r line; do
    for f in "${FORBIDDEN[@]}"; do
        if [[ "$line" == *"$f"* ]]; then
            FOUND+=("Found forbidden crate: $line")
        fi
    done
done <<< "$OUTPUT"

if [[ ${#FOUND[@]} -gt 0 ]]; then
    echo "Gate G5 FAILED - Forbidden Git crates found:" >&2
    printf '%s\n' "${FOUND[@]}" >&2
    exit 1
else
    echo "Gate G5 PASSED - No embedded Git crates"
    exit 0
fi