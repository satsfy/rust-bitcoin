#!/usr/bin/env bash
#
# Install the pinned cargo-fuzz version. Single source of truth for the pin
# across the fuzz workflows.
#
# Build with stable. cargo-fuzz's locked rustix dependency enables
# rustc-internal attributes when it detects a nightly compiler and no longer
# builds there.

set -euo pipefail

version="0.13.2"

if cargo fuzz --version 2>/dev/null | grep -qF "$version"; then
  exit 0
fi

rustup toolchain install stable --profile minimal --no-self-update
cargo +stable install --force --locked --version "$version" cargo-fuzz
