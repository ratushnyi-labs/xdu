#!/bin/sh
# Lawbook §8.7 CI simulation: fmt → clippy → check → test → multi-target release.
# Runs entirely inside Docker (lawbook §8.2).

set -eu

cd "$(dirname "$0")/.."

dev() {
    docker compose run --rm -T dev "$@"
}

printf '\n==> [1/5] cargo fmt --check\n'
dev cargo fmt --all -- --check

printf '\n==> [2/5] cargo clippy -D warnings\n'
dev cargo clippy --all-targets --all-features --locked -- -D warnings

printf '\n==> [3/5] cargo check\n'
dev cargo check --all-targets --locked

printf '\n==> [4/5] cargo test\n'
dev cargo test --locked

printf '\n==> [5/5] release builds (all targets)\n'
sh scripts/build.sh

printf '\n==> CI passed.\n'
