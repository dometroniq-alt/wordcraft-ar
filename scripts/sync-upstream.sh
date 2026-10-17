#!/usr/bin/env bash
# Hybrid update: merge upstream WordCraft into the Arabic fork under supervision.
# Usage: scripts/sync-upstream.sh [upstream-ref]   (default upstream/main)
set -euo pipefail
cd "$(dirname "$0")/.."
REF="${1:-upstream/main}"
git remote get-url upstream >/dev/null 2>&1 || git remote add upstream https://github.com/storytold/wordcraft.git
git fetch upstream
echo "== merging $REF into $(git rev-parse --abbrev-ref HEAD)"
if ! git merge --no-edit "$REF"; then
  echo "== conflicts; files owned by the Arabic fork keep our side:"
  while read -r owned; do
    [ -z "$owned" ] && continue
    for f in $(git diff --name-only --diff-filter=U | grep -F "$owned" || true); do
      git checkout --ours -- "$f" && git add "$f" && echo "   ours: $f"
    done
  done < scripts/ar-owned.txt
  rest=$(git diff --name-only --diff-filter=U || true)
  if [ -n "$rest" ]; then
    echo "== resolve by hand, then: git commit"; echo "$rest"; exit 1
  fi
  git commit --no-edit
fi
echo "== tests"
cargo test -p wordcraft-doc -p wordcraft-layout -p wordcraft-docx -p wordcraft-engine --quiet
echo "== done"
