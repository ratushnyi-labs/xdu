#!/bin/sh
# Run cargo-audit inside the test container to scan workspace dependencies
# against the RustSec advisory database.
#
# Lawbook §7 + org pattern (per TASK-007-A audit finding line 226): pre-publish
# advisory scan must pass clean or have every surviving advisory documented in
# the audit transcript with rationale and explicit user approval.

set -eu

cd "$(dirname "$0")/.."

printf '==> cargo audit (workspace, locked)\n'
docker compose run --rm -T test sh -c 'cargo audit'
