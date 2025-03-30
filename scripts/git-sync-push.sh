#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking Git remote origin..."
git remote -v

if [[ "${1:-}" == "--force" ]]; then
  echo "==> Pushing local commit history to origin/main (--force-with-lease)..."
  git push -u origin main --force-with-lease
else
  if git fetch origin main 2>/dev/null; then
    echo "==> Rebasing local commits cleanly on top of origin/main..."
    git pull --rebase -X theirs origin main || true
  fi
  echo "==> Pushing to origin/main..."
  git push -u origin main
fi
echo "==> Done! Repository is synchronized with GitHub."
