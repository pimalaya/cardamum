---
cairn: delta
change: account-check-names-its-codes
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: account check fails when a backend fails
(commands) `account check` SHALL exercise every backend of the account `--backend` allows and print one verdict per backend, under `--json` `ok`, the stable `code` and the error of a failed one (`code` `null` when the backend passed or its failure carries none), as the failure object of any other command names them. When any fails, it SHALL still print the report and exit 2, the family's partial outcome (`pimalaya_cli::error::PartialOutcome`), so a script reads the outcome from the exit code alone and which backends passed from the report. It mirrors Himalaya's `account check`.

## REMOVED Requirements

None.
