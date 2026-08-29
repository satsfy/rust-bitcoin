#!/usr/bin/env bash

# Manage the shared fuzz corpora stored in the qa-assets repository.
#
# Usage: corpora.sh {seed QA_DIR TARGET | refresh INCOMING_CORPORA INCOMING_CRASHES TARGETS_FILE | replay QA_DIR | push}
#
# Commands:
#   seed     Copy a target's stored corpus into fuzz/corpus.
#   refresh  Replace each stored corpus with the one under INCOMING_CORPORA, add
#            crash inputs under INCOMING_CRASHES to the fuzz_crashes store and
#            drop targets not listed in TARGETS_FILE. The crash store is then trimmed to MAX_STORED_CRASHES.
#   replay   Run every input in the fuzz_crashes store under QA_DIR against
#            its target and fail if any of them still crashes.
#   push     CI only, not meant to be run locally. Commit and push to
#            qa-assets. Reads QA_ASSETS_PUSH_TOKEN, SOURCE, RUN_URL and
#            BRANCH from the environment.

set -euo pipefail

usage="Usage: $0 {seed QA_DIR TARGET | refresh INCOMING_CORPORA INCOMING_CRASHES TARGETS_FILE | replay QA_DIR | push}"

# Upper bound on inputs kept in the crash store, oldest dropped first.
# 40MB is the limit, because libFuzzer default max input size is 4KB.
readonly MAX_STORED_CRASHES=10000

seed() {
  local qa="${1:?$usage}" target="${2:?$usage}"
  local corpus
  corpus="$(dirname "$0")/corpus/$target"
  mkdir -p "$corpus"
  if [ -d "$qa/fuzz_corpora/$target" ]; then
    find "$qa/fuzz_corpora/$target" -maxdepth 1 -type f -exec cp -t "$corpus/" {} +
  fi
}

# Trim the crash store to MAX_STORED_CRASHES, oldest first.
maybe_prune_old_crashes() {
  local total excess file
  total=$(find fuzz_crashes -type f | wc -l)
  excess=$((total - MAX_STORED_CRASHES))
  [ "$excess" -gt 0 ] || return 0
  echo "Crash store holds $total inputs, dropping $excess oldest"
  while IFS= read -r file; do
    [ "$excess" -gt 0 ] || break
    [ -f "$file" ] || continue
    rm -f -- "$file"
    excess=$((excess - 1))
  done < <(git log --reverse --diff-filter=A --format='' --name-only -- fuzz_crashes \
           | awk 'NF && !seen[$0]++')
  find fuzz_crashes -mindepth 1 -type d -empty -delete
}

refresh() {
  local corpora="${1:?$usage}" crashes="${2:?$usage}" targets_file="${3:?$usage}"
  local dir name
  mkdir -p fuzz_corpora fuzz_crashes "$corpora" "$crashes"
  for dir in "$corpora"/corpus-*/; do
    [ -d "$dir" ] || continue # skip on empty corpora, string expansion yields literal pattern
    name=$(basename "$dir")
    name=${name#corpus-}
    rm -rf "fuzz_corpora/$name"
    mkdir -p "fuzz_corpora/$name"
    find "$dir" -maxdepth 1 -type f -exec cp -t "fuzz_corpora/$name/" {} +
  done
  # Crash files are named by libFuzzer after the hash of their contents, -n
  # makes re-adding a known crash a no-op.
  for dir in "$crashes"/crash-*/; do
    [ -d "$dir" ] || continue
    name=$(basename "$dir")
    name=${name#crash-}
    mkdir -p "fuzz_crashes/$name"
    find "$dir" -maxdepth 1 -type f -exec cp -n -t "fuzz_crashes/$name/" {} +
  done
  for dir in fuzz_corpora/*/ fuzz_crashes/*/; do
    [ -d "$dir" ] || continue
    name=$(basename "$dir")
    grep -qx "$name" "$targets_file" || rm -rf "$dir" # drop targets that no longer exist upstream
  done
  maybe_prune_old_crashes
}

replay() {
  local qa="${1:?$usage}"
  local fuzz dir input target rustflags nightly failed=0
  qa="$(cd "$qa" && pwd)"
  fuzz="$(cd "$(dirname "$0")" && pwd)"
  cd "$fuzz"
  nightly="$(cargo rbmt toolchains --nightly)"
  for dir in "$qa"/fuzz_crashes/*/; do
    [ -d "$dir" ] || continue
    target=$(basename "$dir")
    rustflags=''
    if [[ ! "$target" =~ ^hashes_ ]]; then
      rustflags='--cfg=hashes_fuzz --cfg=secp256k1_fuzz'
    fi
    # Inputs go in as files, not as a directory, so libFuzzer runs each one and
    # exits instead of taking the store for a corpus to fuzz.
    while IFS= read -r -d '' input; do
      echo "Replaying $target/$(basename "$input")"
      if ! RUSTFLAGS="${RUSTFLAGS:-} $rustflags" cargo +"$nightly" fuzz run "$target" "$input"; then
        echo "::error title=Stored crash still reproduces in $target::$(basename "$input")"
        failed=1
      fi
    done < <(find "$dir" -maxdepth 1 -type f -print0)
  done
  [ "$failed" = 0 ] || exit 1
}

push() {
  git add -A fuzz_corpora fuzz_crashes
  if git diff --cached --quiet; then
    echo "No corpus changes"
    exit 0
  fi
  if [ -z "${QA_ASSETS_PUSH_TOKEN:-}" ]; then
    echo "::error::QA_ASSETS_PUSH_TOKEN is not set, refusing to drop the corpus updates"
    exit 1
  fi
  git config user.name "github-actions[bot]"
  git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
  git commit -m "Update fuzz corpora" \
    -m "Source: $SOURCE" \
    -m "Run: $RUN_URL"
  # try to push three times, rebasing on the remote branch if we race with it
  for _ in 1 2 3; do
    git push origin "HEAD:$BRANCH" && exit 0
    git pull --rebase origin "$BRANCH"
  done
  exit 1
}

export LC_ALL=C
cmd="${1:?$usage}"
shift
case "$cmd" in
  seed | refresh | replay | push) "$cmd" "$@" ;;
  *)
    echo "$usage" >&2
    exit 2
    ;;
esac
