---
cairn: change
id: account-check-names-its-codes
status: landed
created: 2026-11-12
---

# account check names its codes

## Why

account-check-reports-its-failure made a failed `account check` print its report and exit 2, which leaves the per-backend result as the only place a `--json` caller reads a failure from. Himalaya's result carries `code` beside `error` (`{backend, ok, code, error}`), and so does calendula's since 5564131; cardamum's carried the message alone, so a pimdir store answering `store-busy` or `store-uncreated` lost its code on the one command meant to diagnose it. Raised by the calendula session on 2026-11-12.

## What

- `BackendCheck` gains `code: Option<ErrorCode>`, `pimalaya_cli::error::code_of` of the failure, `null` when the backend passed or its failure carries none. The text report is unchanged.
