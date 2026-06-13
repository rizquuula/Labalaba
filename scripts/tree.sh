#!/usr/bin/env bash
# Print the project tree annotated with each file's line count.
# Build artifacts and vendored dirs are pruned to keep the output meaningful.
set -euo pipefail

# Resolve repo root (parent of this script's dir) so it works from anywhere.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Directories we never want to walk into or count.
PRUNE_DIRS=(.git node_modules target .svelte-kit build dist)

# Build a `find` prune expression from PRUNE_DIRS.
prune_expr=()
for d in "${PRUNE_DIRS[@]}"; do
	prune_expr+=(-name "$d" -o)
done
unset 'prune_expr[${#prune_expr[@]}-1]'  # drop trailing -o

total_files=0
total_lines=0

indent_for() {  # echo a tree indent for the given depth
	local depth=$1 i out=""
	for ((i = 0; i < depth; i++)); do out+="│   "; done
	printf '%s' "$out"
}

echo "."

# Walk dirs and files together, sorted, skipping pruned subtrees.
while IFS= read -r -d '' path; do
	rel="${path#./}"
	[ -z "$rel" ] && continue
	depth=$(awk -F/ '{print NF-1}' <<< "$rel")
	name="${rel##*/}"
	if [ -d "$path" ]; then
		printf "%s├── %s/\n" "$(indent_for "$depth")" "$name"
	else
		lines=$(wc -l < "$path" 2>/dev/null || echo 0)
		total_files=$((total_files + 1))
		total_lines=$((total_lines + lines))
		printf "%s├── %s  (%s lines)\n" "$(indent_for "$depth")" "$name" "$lines"
	fi
done < <(
	find . -mindepth 1 \( "${prune_expr[@]}" \) -prune -o -print0 | sort -z
)

printf "\n%s files, %s lines total\n" "$total_files" "$total_lines"
