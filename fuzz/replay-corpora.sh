#!/usr/bin/env bash

# Replay stored fuzz corpora against the fuzz targets without fuzzing.
#
# Every input in the corpus is executed once (-runs=0), so a run takes
# seconds per target and acts as a regression test for every crash that
# fuzzing has found before. Corpora are maintained in the qa-assets repo.
#
# Usage: replay-corpora.sh CORPORA_DIR [TARGET]
#
# CORPORA_DIR is the fuzz_corpora directory of a qa-assets checkout.
# Targets with no stored corpus are skipped.

set -euo pipefail

REPO_DIR=$(git rev-parse --show-toplevel)

# can't find the file because of the ENV var
# shellcheck source=/dev/null
source "$REPO_DIR/fuzz/fuzz-util.sh"

corporaDir=${1:?usage: replay-corpora.sh CORPORA_DIR [TARGET]}
corporaDir=$(cd -- "$corporaDir" && pwd)
target=${2:-}

if [ -z "$target" ]; then
  targetFiles="$(listTargetFiles)"
else
  targetFiles=fuzz_targets/"$target".rs
fi

# The hashes targets build without the fuzz stubs while everything else
# builds with them. Changing RUSTFLAGS rebuilds the whole dependency tree,
# so replay all non-hashes targets first and the hashes targets last to
# build only twice instead of at every group boundary in sort order.
orderedTargetFiles=$(
  for f in $targetFiles; do
    case "$(targetFileToName "$f")" in hashes_*) ;; *) echo "$f" ;; esac
  done
  for f in $targetFiles; do
    case "$(targetFileToName "$f")" in hashes_*) echo "$f" ;; esac
  done
)

cargo --version
rustc --version

cargo install --force --locked --version 0.12.0 cargo-fuzz

for targetFile in $orderedTargetFiles; do
  targetName=$(targetFileToName "$targetFile")

  corpus="$corporaDir/$targetName"
  if [ ! -d "$corpus" ]; then
    echo "No stored corpus for $targetName, skipping"
    continue
  fi

  # Same fuzz stub selection as fuzz.sh. The corpora were generated under
  # these cfgs and crashes may only reproduce under them.
  fuzz_rustflags=''
  if [[ ! "$targetName" =~ ^hashes_ ]]; then
    fuzz_rustflags='--cfg=hashes_fuzz --cfg=secp256k1_fuzz'
  fi

  echo "Replaying $(find "$corpus" -maxdepth 1 -type f | wc -l) inputs for $targetName"
  RUSTFLAGS="${RUSTFLAGS:-} $fuzz_rustflags" cargo +nightly fuzz run "$targetName" "$corpus" -- -runs=0
done
