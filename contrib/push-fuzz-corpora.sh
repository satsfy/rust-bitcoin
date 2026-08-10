#!/usr/bin/env bash
#
# Run from a qa-assets checkout. Commit the fuzz_corpora/ changes and push,
# retrying on a racing push. No-op when nothing changed. Refuses to commit
# without QA_TOKEN so a green run means the corpora were persisted.
#
# Usage: ./contrib/push-fuzz-corpora.sh
#
# Reads QA_TOKEN, SOURCE, RUN_URL and BRANCH from the environment.

set -euo pipefail

git add -A fuzz_corpora fuzz_crashes
if git diff --cached --quiet; then
  echo "No corpus changes"
  exit 0
fi
if [ -z "${QA_TOKEN:-}" ]; then
  echo "::error::QA_ASSETS_PUSH_TOKEN is not set, refusing to drop the corpus updates"
  exit 1
fi

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git commit -m "Update fuzz corpora" \
  -m "Source: $SOURCE" \
  -m "Run: $RUN_URL"
for _ in 1 2 3; do
  git push origin "HEAD:$BRANCH" && exit 0
  git pull --rebase origin "$BRANCH"
done
exit 1
