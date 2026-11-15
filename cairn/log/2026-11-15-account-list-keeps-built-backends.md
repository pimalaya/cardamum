---
cairn: log
change: a-backend-left-out-is-named
date: 2026-11-15
---

# account list keeps the built backends apart

A follow-up to a-backend-left-out-is-named, aligned with Calendula (owner's main session, 2026-11-15): `account list --json` keeps `backends` to the declared backends the build carries, each a value `--backend` takes as in v0.3, and names the ones it leaves out in `notBuilt` alone, where 41b1a12 listed them in both. Both follow the `auto` order; the text prints them in one cell in that order, a left-out one marked `(not built)`. Capability **backends** moved.

- **src/account/list.rs**: `AccountRow::from_account` partitions `AccountConfig::backends` by `is_built`; the text merges the two lists back into the `NAMES` order.
- **Evidence**: tests/account.rs `account_list_names_a_backend_left_out_apart` (vdir build, an account declaring vdir and gpeople): `backends` `["vdir"]`, `notBuilt` `["gpeople"]`, text `vdir,gpeople (not built)`; it fails on 41b1a12, whose `backends` read `["vdir","gpeople"]`.

CHANGELOG and MIGRATION.md amended. Spec: backends ("A backend left out of the build is named", its `account list` sentence and a scenario).
