## Summary

<!-- Brief description of what this PR does -->

## Checklist

- [ ] Tests pass (`docker compose run --rm dev sh -c "cargo fmt --all -- --check && cargo clippy --all-targets --all-features --locked -- -D warnings && cargo test --locked"`)
- [ ] Multi-target release build succeeds (`sh scripts/build.sh`)
- [ ] No `unsafe` code without a `// SAFETY:` comment
- [ ] BR-001 read-only invariant preserved (mutating paths: only `d` delete + `f` force-close, both gated by their confirmation modals)
- [ ] Doc comments on new public functions
- [ ] Conventional commit messages with backlog item ID (e.g. `TASK-###:` or `BUG-###:`)
- [ ] No dead code, TODOs, or commented-out logic
- [ ] Line width ≤ 120 characters
- [ ] Backlog item updated (`docs/backlog/`), Speck updated (`docs/specks/`) if applicable

## Test Plan

<!-- How was this tested? Include any new tests added and any manual verification steps. -->
