---
cairn: change
id: account-check-fails-on-a-failed-backend
status: landed
created: 2026-10-30
---

# account check fails when a backend fails

## Why

`account check` printed `<backend>: FAIL (...)` and exited 0 (src/account/check.rs, V9 in [vdir-local](../../../spec/testing/vdir-local.md), open since 2026-08-09). The output contract makes the exit code the only success signal on stdout, so a script or CI step running `cardamum account check` never saw the failure. The pimdir stable-release plan of 2026-10-30 (Apps, Phase A) made calendula exit non-zero on FAIL (calendula/cairn/log/2026-10-30-account-check-fails-on-a-failed-backend.md); cardamum follows it.

## What

- `account check` prints its report when every checked backend passed, and otherwise fails with one error naming each failed backend and why, printed (plain or JSON) as any failure is, with a non-zero exit. One value on stdout either way, as calendula does.
- The wizard, which runs its own fail-fast check on the account it builds, is unchanged.
