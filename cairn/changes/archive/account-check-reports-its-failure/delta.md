---
cairn: delta
change: account-check-reports-its-failure
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: account check fails when a backend fails
(commands) `account check` SHALL exercise every backend of the account `--backend` allows and print one verdict per backend, `ok` and the error of a failed one under `--json`. When any fails, it SHALL still print the report and exit 2, the family's partial outcome (`pimalaya_cli::error::PartialOutcome`), so a script reads the outcome from the exit code alone and which backends passed from the report. It mirrors Himalaya's `account check`.

## REMOVED Requirements

None.
