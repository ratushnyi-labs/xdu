# TASK-001-L — JSON export

**Status:** READY  **Type:** TASK  **Parent:** TASK-001
**User Mode:** GUIDED  **Guided Flow Stage:** PLAN_READY
**Created:** 2026-05-06  **Updated:** 2026-05-06

## Refined Description
**Scope:** `e` keybinding writes deterministic JSON snapshot of the focused subtree per UC-006. Hand-rolled streaming encoder (no `serde_json` dep) to keep binary small. Modal prompt with default path `xdu-export.json`.
**Impacted UCs:** UC-006.
**Impacted BR/WF:** BR-001 (export writes only the export file; never to scanned tree).
**Depends on:** TASK-001-F.

## Lifecycle Coverage Map
| Stage | State |
|---|---|
| CLARIFICATION | DONE | ESTIMATION | DONE | DECOMPOSITION | N/A | DESIGN | DONE | BACKEND | PENDING | TESTING | PENDING | POST_AUDIT | PENDING |

## Estimation
**Complexity:** EASY

## Acceptance (pointers)
- AC-1: schema field `"schema": "xdu-export-1"`.
- AC-2: deterministic key ordering (children sorted by name asc).
- AC-3: refuses to write into `/proc /sys /dev` on Linux; refuses null-byte paths.
- AC-4: refuses to overwrite without explicit confirmation key.
- SEC-1: JSON string escaping covers `\b \f \n \r \t \"` and control codes `0x00–0x1F`.
- SEC-2: refuses paths containing `..` segments resolving outside CWD.

## Speck Reference
`docs/specks/SDD-TASK-001-L-json-export.md` (to be authored).
