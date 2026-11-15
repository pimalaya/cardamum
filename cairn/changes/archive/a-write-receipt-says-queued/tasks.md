---
cairn: tasks
change: a-write-receipt-says-queued
---

# Tasks

- [x] src/shared/card/create.rs and update.rs serialize `queued`; src/shared/addressbook/create.rs prints `name` and `queued`.
- [x] Tests: the pimdir `a_queued_write_says_it_is_queued` and the Graph update test assert the new JSON (each fails with `queued` skipped), the `addressbook create` receipt test.
- [x] Fold the delta, log, CHANGELOG, MIGRATION.md.
