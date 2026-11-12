---
cairn: log
change: account-check-reports-its-failure
date: 2026-11-12
---

# A failed account check prints its report

`account check` prints its per-backend report whatever the outcome and exits 2 (`PartialOutcome`) when a backend failed, where it exited 1 with one bare failure object. The unit test reads `vdir: FAIL (...)` from the report; the 10 anonymous live tests read `ok: false` and the 401 from it.

Capabilities moved: commands ("account check fails when a backend fails").
