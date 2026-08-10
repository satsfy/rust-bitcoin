#!/usr/bin/env bash
#
# Report every crash input under fuzz/artifacts to the GitHub step summary,
# hex dump, base64 of the input and repro steps, plus one error annotation
# per crash so failures surface on the run's front page.
#
# Usage: ./contrib/report-fuzz-crashes.sh ARTIFACT_NAME
#
# ARTIFACT_NAME is the uploaded artifact holding the crash inputs, pointed
# to when an input is too large to inline.

set -euo pipefail

artifact_name="${1:?Usage: $0 ARTIFACT_NAME}"
summary="${GITHUB_STEP_SUMMARY:-/dev/stdout}"

cd "$(git rev-parse --show-toplevel)"

found=0
for tdir in fuzz/artifacts/*/; do
  [ -d "$tdir" ] || continue
  target=$(basename "$tdir")
  for f in "$tdir"*; do
    [ -f "$f" ] || continue
    found=1
    name=$(basename "$f")
    size=$(wc -c < "$f")
    {
      echo "## Fuzz crash: $target"
      echo ""
      echo "\`$name\` ($size bytes)"
      echo ""
      echo '```'
      xxd "$f" | head -n 64
      echo '```'
      echo ""
      if [ "$size" -le 65536 ]; then
        echo "Reproduce locally (base64 of the full input):"
        echo ""
        echo '```'
        base64 -w0 "$f"
        echo ""
        echo '```'
        echo ""
      else
        echo "Input too large to inline, grab it from the $artifact_name artifact."
        echo ""
      fi
      echo '```sh'
      if [[ "$target" != hashes_* ]]; then
        echo 'export RUSTFLAGS="--cfg=hashes_fuzz --cfg=secp256k1_fuzz"'
      fi
      echo "base64 -d > crash <<'EOF' # paste the base64 line above"
      echo "EOF"
      echo "cargo +nightly fuzz run $target crash"
      echo '```'
      echo ""
    } >> "$summary"
    echo "::error title=Fuzz crash in $target::$name ($size bytes), hex dump and repro steps in the job summary"
  done
done

if [ "$found" = 0 ]; then
  echo "::error title=Fuzz failure::job failed without producing a crash artifact, see the fuzz step log"
fi
