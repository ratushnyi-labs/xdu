# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in xdu, please report it responsibly.

**Do NOT open a public issue.** Instead, use one of these methods:

1. **GitHub Security Advisories** (preferred): [Report a vulnerability](https://github.com/ratushnyi-labs/xdu/security/advisories/new)
2. **Email**: Contact the maintainers privately

## Response Timeline

- **Acknowledgment**: within 48 hours
- **Initial assessment**: within 7 days
- **Fix or mitigation**: based on severity, targeting the next release

## Scope

The following are considered security issues:

- Panics or crashes on malformed filesystem input (entries with control bytes, very deep
  trees, very long names, mid-scan deletion, etc.).
- Path traversal in the JSON exporter path argument.
- Bypasses of the read-only invariant (BR-001): any code path that mutates the filesystem
  outside of the explicit `d` (delete) and `f` (force-close handles) flows, or that
  bypasses the confirmation modals.
- Force-close handle path closing more than the targeted file's handles, or closing
  handles in `xdu`'s own process.
- Memory safety violations in `unsafe` blocks (Windows / Linux FFI in
  `src/disk_space.rs`, `src/lock_detect.rs`).
- Resource exhaustion from crafted input (unbounded error vector, unbounded handle
  enumeration).

The following are NOT security issues:

- Performance regressions or scan-throughput complaints (file a feature request).
- The user deleting a file with `d` (this is the documented mutation path).
- Foreign-app data loss after `f` force-close (this is the documented consequence
  surfaced in the force-confirm modal).
