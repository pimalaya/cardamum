---
cairn: log
change: pimdir-status-shows-parked-actions
landed: 2026-10-30
---

# pimdir status shows parked actions

cardamum gains a `pimdir` command family (src/pimdir/cli.rs, gated on the `pimdir` feature) with one subcommand, `status` (src/pimdir/status.rs, `PimdirBackend::status`), mirroring calendula's: the account read, the accounts the store groups, one row per address book of that account (`total`, `hydrated`, `queued` creations), and the actions the owner parked in those address books, read from `PimdirReader::parked_actions()` and printed under "Parked, never applied:" as `ROW <row> <action> in <addressbook>: <error>`. Its JSON is `PimdirStatusOutput`, `{"account", "accounts", "addressbooks": [{"id", "name", "total", "hydrated", "queued"}], "parked": [{"row", "addressbookId", "action", "error"}]}`, registered as `cardamum-pimdir-status`; calendula's `calendars` and `calendarId` are `addressbooks` and `addressbookId` here. The command builds its backend through `PimdirBackend::build` from the account's `[pimdir]` block. A guarded `card update` the owner parks (pimdir-follows-storage-draft-05) is no longer visible only through the sync engine.

Test: `status_shows_the_parked_edit_of_the_account_read` queues a guarded edit in each of two accounts' address books, lands another body through a sync write, drains, and checks `work` sees its own `update` row alone, with the owner's "body replaced since it was read" error, in text and JSON. New command, so no old-code run to fail; with the account filter disabled the test fails with both rows (`left: 2, right: 1`).

Capabilities moved: **commands** (pimdir status reports the store and its parked actions, added; specific commands mirror their protocol, modified), **backends** (a queued pimdir edit names the body it replaced, modified).
