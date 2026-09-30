#!/bin/bash
# Gate G4: No git spawn outside crates/core
# Fails the build on any Command::new("git"), process.spawn(, or exec("git outside crates/core

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE_PATH="$ROOT/crates/core"
ERRORS=()

# Check Rust files outside crates/core
while IFS= read -r -d '' file; do
    # Skip core crate
    if [[ "$file" == *"crates/core"* ]]; then
        continue
    fi
    
    line_num=0
    while IFS= read -r line; do
        ((line_num++))
        if [[ "$line" =~ Command::new[[:space:]]*\([[:space:]]*[\"'"'"']git[\"'"'"'] ]]; then
            ERRORS+=("RUST: $file:$line_num: git spawn via Command::new")
        fi
        if [[ "$line" =~ process\.spawn\( ]]; then
            ERRORS+=("RUST: $file:$line_num: process.spawn")
        fi
        if [[ "$line" =~ exec[[:space:]]*\([[:space:]]*[\"'"'"']git[\"'"'"'] ]]; then
            ERRORS+=("RUST: $file:$line_num: git exec")
        fi
    done < "$file"
done < <(find "$ROOT" -type f -name "*.rs" -not -path "*/crates/core/*" -not -path "*/target/*" -print0)

# Check JS/TS files
while IFS= read -r -d '' file; do
    line_num=0
    while IFS= read -r line; do
        ((line_num++))
        if [[ "$line" =~ spawn[[:space:]]*\([[:space:]]*[\"'"'"']git[\"'"'"'] ]]; then
            ERRORS+=("JS: $file:$line_num: git spawn")
        fi
        if [[ "$line" =~ exec[[:space:]]*\([[:space:]]*[\"'"'"']git[\"'"'"'] ]]; then
            ERRORS+=("JS: $file:$line_num: git exec")
        fi
    done < "$file"
done < <(find "$ROOT" -type f \( -name "*.js" -o -name "*.ts" -o -name "*.mjs" \) -not -path "*/crates/core/*" -not -path "*/node_modules/*" -not -path "*/spike/*" -not -path "*/target/*" -print0)

if [[ ${#ERRORS[@]} -gt 0 ]]; then
    echo "Gate G4 FAILED - Git spawns found outside crates/core:" >&2
    printf '%s\n' "${ERRORS[@]}" >&2
    exit 1
else
    echo "Gate G4 PASSED - No git spawns outside crates/core"
    exit 0
fi