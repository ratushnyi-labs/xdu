#!/bin/sh
# Multi-target release build via docker-compose.
# Lawbook §8.2: all builds run in containers, never on host.
# Lawbook ADR-001 §4: enforce 1.5 MB size ceiling per binary.

set -eu

cd "$(dirname "$0")/.."

TARGETS="${TARGETS:-x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin}"
SIZE_CEILING_BYTES="${SIZE_CEILING_BYTES:-1572864}" # 1.5 MiB

mkdir -p dist

failed=""
oversize=""

for t in $TARGETS; do
    printf '\n==> Building %s\n' "$t"
    if ! TARGET="$t" docker compose build build >/dev/null 2>&1; then
        printf '    [error] image build failed for %s\n' "$t"
        failed="$failed $t"
        continue
    fi
    if ! TARGET="$t" docker compose run --rm build; then
        failed="$failed $t"
        continue
    fi

    case "$t" in
        *windows*) ext=".exe" ;;
        *)         ext="" ;;
    esac

    bin="dist/$t/xdu$ext"
    if [ ! -f "$bin" ]; then
        printf '    [error] expected binary not found: %s\n' "$bin"
        failed="$failed $t"
        continue
    fi

    size=$(wc -c < "$bin" | tr -d ' ')
    printf '    binary: %s\n    size  : %s bytes\n' "$bin" "$size"

    if [ "$size" -gt "$SIZE_CEILING_BYTES" ]; then
        printf '    [error] size %s exceeds ceiling %s\n' "$size" "$SIZE_CEILING_BYTES"
        oversize="$oversize $t"
    fi
done

printf '\n==> Build summary\n'
if [ -n "$failed" ]; then
    printf '    failed targets:%s\n' "$failed"
fi
if [ -n "$oversize" ]; then
    printf '    oversize targets:%s\n' "$oversize"
fi

if [ -n "$failed" ] || [ -n "$oversize" ]; then
    exit 1
fi

printf '    all targets built within size budget.\n'
