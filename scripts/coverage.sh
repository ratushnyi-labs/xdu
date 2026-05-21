#!/bin/sh
# Run cargo-llvm-cov inside the test container and emit:
#   - a human-readable per-file coverage table on stdout
#   - coverage.lcov for downstream tooling (Codecov etc.)
#
# Lawbook §7.1: 100% lines + branches + functions on the library crate is the
# target; this script is the local gate that surfaces regressions.

set -eu

cd "$(dirname "$0")/.."

printf '==> cargo llvm-cov (workspace, locked)\n'
docker compose run --rm -T test sh -c '
    set -eu
    cargo llvm-cov --workspace --locked --lcov --output-path coverage.lcov
    cargo llvm-cov --workspace --locked --summary-only
'

printf '\n==> coverage.lcov size\n'
wc -c coverage.lcov 2>/dev/null || true

printf '\n==> done. Open coverage.lcov in a viewer or upload to Codecov.\n'
