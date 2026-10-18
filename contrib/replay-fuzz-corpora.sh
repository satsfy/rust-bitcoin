#!/usr/bin/env bash
#
# Replay the stored fuzz corpora and crash inputs of a qa-assets checkout,
# running every input once without mutation (-runs=0). The corpora hold
# coverage-interesting inputs that previously ran clean, so they catch new
# crashes on known coverage. The crash store holds every crashing input
# ever found, so a fixed crash that gets reintroduced fails here. Targets
# with no stored inputs yet are skipped. All targets are replayed even when
# one fails, the script exits non-zero at the end listing the failing
# targets.
#
# Usage: ./contrib/replay-fuzz-corpora.sh QA_ASSETS_DIR
#
# QA_ASSETS_DIR is a qa-assets checkout, its fuzz_corpora/ and fuzz_crashes/
# directories hold one subdirectory of inputs per cargo-fuzz target.

set -euo pipefail

if [ "$#" -ne 1 ] || [ ! -d "$1" ]; then
  echo "Usage: $0 QA_ASSETS_DIR (a qa-assets checkout)" >&2
  exit 2
fi
qa_dir=$(cd -- "$1" && pwd)

if ! command -v cargo-fuzz &> /dev/null; then
  echo "ERROR: cargo-fuzz is required but not installed (cargo install --locked cargo-fuzz)"
  exit 127
fi

REPO_DIR=$(git rev-parse --show-toplevel)
cd "$REPO_DIR/fuzz"

failed=""
for target in $(cargo fuzz list); do
  dirs=()
  for store in fuzz_corpora fuzz_crashes; do
    [ -d "$qa_dir/$store/$target" ] && dirs+=("$qa_dir/$store/$target")
  done
  if [ "${#dirs[@]}" -eq 0 ]; then
    echo "No stored inputs for $target yet, skipping"
    continue
  fi
  # Same cfg selection as fuzz.sh, the hashes targets fuzz the real
  # implementations, everything else uses the fuzz stubs.
  fuzz_rustflags=''
  if [[ ! "$target" =~ ^hashes_ ]]; then
    fuzz_rustflags='--cfg=hashes_fuzz --cfg=secp256k1_fuzz'
  fi
  echo "Replaying $(find "${dirs[@]}" -maxdepth 1 -type f | wc -l) inputs for $target"
  RUSTFLAGS="${RUSTFLAGS:-} ${fuzz_rustflags}" cargo fuzz run "$target" "${dirs[@]}" -- -runs=0 \
    || failed="$failed $target"
done

if [ -n "$failed" ]; then
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    echo "::error title=Corpus replay failed::failing targets:$failed"
  fi
  echo "Corpus replay failed for:$failed" >&2
  exit 1
fi
