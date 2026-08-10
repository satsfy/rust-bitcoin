#!/usr/bin/env bash
#
# Fuzz this shard's slice of the targets. Each target's corpus is seeded
# from CORPORA_DIR, then fuzz.sh fuzzes and minimizes it. Refreshed corpora
# of targets that fuzzed clean are staged in fuzz/outgoing, a failed target
# is left out so it keeps its stored corpus. All targets are fuzzed even
# when one fails, the script exits non-zero at the end listing the failing
# targets.
#
# Usage: ./contrib/fuzz-shard.sh SHARD_ID SHARD_COUNT CORPORA_DIR
#
# CORPORA_DIR is the fuzz_corpora directory of a qa-assets checkout.
# MAX_TOTAL_TIME sets the seconds of fuzzing per target (default 300).

set -euo pipefail

if [ "$#" -ne 3 ] || [ ! -d "$3" ]; then
  echo "Usage: $0 SHARD_ID SHARD_COUNT CORPORA_DIR" >&2
  exit 2
fi
shard_id=$1
shard_count=$2
corpora_dir=$(cd -- "$3" && pwd)

cd "$(git rev-parse --show-toplevel)/fuzz"

shard_targets=$(cargo fuzz list | sort \
  | awk -v shard="$shard_id" -v count="$shard_count" '(NR - 1) % count == shard')

failed=""
mkdir -p outgoing
for target in $shard_targets; do
  mkdir -p "corpus/$target"
  if [ -d "$corpora_dir/$target" ]; then
    find "$corpora_dir/$target" -maxdepth 1 -type f -exec cp -t "corpus/$target/" {} +
  fi
  if ./fuzz.sh "$target" -max_total_time="${MAX_TOTAL_TIME:-300}"; then
    mkdir -p "outgoing/$target"
    find "corpus/$target" -maxdepth 1 -type f -exec cp -t "outgoing/$target/" {} +
  else
    failed="$failed $target"
  fi
done

if [ -n "$failed" ]; then
  echo "Fuzzing failed for:$failed" >&2
  exit 1
fi
