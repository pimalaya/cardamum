---
cairn: log
change: a-write-receipt-says-queued
date: 2026-11-15
---

# A write receipt says queued

`card create` prints `{id, queued}`, `card update` `{id, keptProperties, queued}` and `addressbook create` `{id, name, queued}`, `queued` the family's receipt key (pimalaya-cli output.md), where the first two kept it for the text alone (reverses 2026-10-30-queued-stays-out-of-json, on the family's ruling) and the third printed `{id}`. Task C3 of cli-alignment-2026-11-15, twin of calendula's K3. `keptProperties` stays (P2), the spec saying why; a queued create keeps reporting its link id. Capabilities **commands** and **backends** moved.

Evidence: the pimdir `a_queued_write_says_it_is_queued` and the Graph update test assert the new JSON and fail with `queued` skipped; a receipt test for `addressbook create`. MIGRATION.md and CHANGELOG say it.
