---
cairn: change
change: json-error-code
---

# Delta

## ADDED Requirements

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`. The code SHALL be found anywhere in the error chain, so added context does not hide it. A failure with no code SHALL print no `code` field. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording.

| Code | Raised when |
|---|---|
| `body-pending` | a listed card's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the card no longer has (pimdir) |

#### Scenario: A stale version
- GIVEN a pimdir card whose body changed since it was read
- WHEN `cardamum --json card delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`
