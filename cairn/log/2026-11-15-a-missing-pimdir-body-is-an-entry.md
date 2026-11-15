---
cairn: log
change: a-missing-pimdir-body-is-an-entry
date: 2026-11-15
---

# A missing pimdir body is an entry

A pimdir card whose blob is gone from the store is a `card-unreadable` entry of `card list` (``Body blob missing from the store``), no row listed, exit 3, as calendula names it `item-unreadable`, where it listed its preview with a log warning. `pimdir search` keeps the preview and the warning until it has `errors` (C20). Task C19 of cli-alignment-2026-11-15. Capabilities **backends** and **commands** moved. Evidence: `a_missing_body_is_an_entry_alone`, and `a_card_whose_blob_is_missing_has_no_version` asserting the entry it asserted absent.
