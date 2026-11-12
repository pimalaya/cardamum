---
cairn: change
id: account-check-reports-its-failure
status: landed
created: 2026-11-12
---

# A failed account check prints its report

## Why

The family output contract (pimalaya-cli output.md, every-output-is-data) exits 2 with a printed result whose outcome is not a full success, a failed `account check` being its example, and Himalaya does. Cardamum exited 1 with one bare failure object, so a `--json` caller could not see which backends passed. The live suites found it (live-provider-tests, 2026-11-12); a gap of the-family-standards, landed with it.

## What

`account check` prints its report whatever the outcome and ends with `PartialOutcome` (exit 2) when a backend failed. The unit test and tests/common.rs `json_check_failure` read the report.
