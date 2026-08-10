#!/usr/bin/env bash
#
# Run from a qa-assets checkout. Replace each target's stored corpus in
# fuzz_corpora/ with the staged one found under INCOMING_CORPORA, add any
# crash inputs found under INCOMING_CRASHES to the fuzz_crashes/ store,
# then drop entries of targets that no longer exist in REPO_DIR.
#
# Usage: ./contrib/refresh-fuzz-corpora.sh INCOMING_CORPORA INCOMING_CRASHES REPO_DIR
#
# The incoming directories hold one directory per downloaded artifact, each
# containing one subdirectory of inputs per target. Corpora are replaced,
# crashes only accumulate, they are the regression suite of every crash
# ever found. REPO_DIR is a rust-bitcoin checkout, only used to enumerate
# the fuzz targets.

set -euo pipefail

usage="Usage: $0 INCOMING_CORPORA INCOMING_CRASHES REPO_DIR"
corpora="${1:?$usage}"
crashes="${2:?$usage}"
repo="${3:?$usage}"
export LC_ALL=C

mkdir -p fuzz_corpora fuzz_crashes "$corpora" "$crashes"

for dir in "$corpora"/*/*/; do
  [ -d "$dir" ] || continue
  name=$(basename "$dir")
  rm -rf "fuzz_corpora/$name"
  mkdir -p "fuzz_corpora/$name"
  find "$dir" -maxdepth 1 -type f -exec cp -t "fuzz_corpora/$name/" {} +
done

# Crash files are named by libFuzzer after the hash of their contents, -n
# makes re-adding a known crash a no-op.
for dir in "$crashes"/*/*/; do
  [ -d "$dir" ] || continue
  name=$(basename "$dir")
  mkdir -p "fuzz_crashes/$name"
  find "$dir" -maxdepth 1 -type f -exec cp -n -t "fuzz_crashes/$name/" {} +
done

# Same file-path to target-name transformation as fuzz/generate-bins.sh.
targets=$(cd "$repo/fuzz" && find fuzz_targets/ -type f -name '*.rs' \
  | sed 's/^fuzz_targets\///; s/\.rs$//; s/\//_/g; s/^_//' \
  | sort)
for dir in fuzz_corpora/*/ fuzz_crashes/*/; do
  [ -d "$dir" ] || continue
  name=$(basename "$dir")
  echo "$targets" | grep -qx "$name" || rm -rf "$dir"
done
