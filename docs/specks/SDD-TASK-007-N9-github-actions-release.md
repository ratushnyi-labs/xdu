# SDD-TASK-007-N9 — .github/workflows/release.yml (on v* tag → GH Release)

**Backlog item:** TASK-007-N9
**Impacted UCs:** none directly (release scaffolding)

## Scope
- New file `.github/workflows/release.yml`. Trigger: `on: push: tags: ["v*"]`.
- Job `test`:
  - Identical to ci.yml's `test` job. Gates the release.
- Job `build`:
  - `needs: test`.
  - Identical matrix to ci.yml's `build` job (6 entries).
  - Same steps + `upload-artifact@v4` with `name: xdu-${{ matrix.target }}`.
- Job `release`:
  - `needs: [test, build]`.
  - `runs-on: ubuntu-latest`.
  - `permissions: contents: write` (required for GH Release upload).
  - Steps:
    1. `actions/download-artifact@v4` with `path: artifacts/`.
    2. Package step: for each downloaded `xdu-<TARGET>/` directory:
       - If TARGET contains `windows` → `zip -j xdu-<TARGET>.zip xdu-<TARGET>/xdu.exe`.
       - Else → `tar -C xdu-<TARGET> -czf xdu-<TARGET>.tar.gz xdu`.
    3. `softprops/action-gh-release@v2`:
       - `files: xdu-*.tar.gz xdu-*.zip`
       - `generate_release_notes: true`
       - `draft: false`
       - `prerelease: false`
- Workflow `name: release`.

## ACs
- AC-1: Pushing a tag `v0.1.0` triggers the workflow.
- AC-2: All 6 build artefacts are downloaded and packaged.
- AC-3: GitHub Release named `v0.1.0` is created with 6 asset files attached (`xdu-x86_64-unknown-linux-musl.tar.gz`, `xdu-aarch64-unknown-linux-musl.tar.gz`, `xdu-x86_64-apple-darwin.tar.gz`, `xdu-aarch64-apple-darwin.tar.gz`, `xdu-x86_64-pc-windows-msvc.zip`, `xdu-aarch64-pc-windows-msvc.zip`).
- AC-4: Release notes are auto-generated from commits since the previous tag (or from repo creation if first tag).
- AC-5: A failing `test` or any failing `build` matrix entry prevents the `release` job from running.

## SEC ACs
- SEC-1: All actions pinned to major versions; only well-known publishers (`actions/*`, `dtolnay/*`, `softprops/*`).
- SEC-2: `permissions:` minimised — `contents: read` on `test`/`build`, `contents: write` only on `release`.
- SEC-3: Tag-triggered only — cannot be invoked by an arbitrary PR.
- SEC-4: §8.2 — `test` job continues to use Docker for parity with local gate.

## §9.5 cross-agent attack review
Skipped per session-wide rule (no second-agent vendor reachable). See parent TASK-007.
