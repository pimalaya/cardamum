---
cairn: tasks
change: the-pimdir-queue-is-read-and-cancelled
---

# Tasks

- [x] src/pimdir/queue/ (cli, list, show, cancel, show and cancel ported from Himalaya); src/pimdir/backend.rs `queued_cards`, `parked_rows`, `queue_row`, `cancel_queued`; src/pimdir/cli.rs takes the account name for the listing frame.
- [x] Tests: `a_queued_card_lists_in_the_queue_until_cancelled`, `an_applied_creation_names_its_card_and_is_not_cancelled`, `a_parked_row_lists_with_its_kind` (backend), the output tests of list, show and cancel.
- [x] `pimdir status` parked rows on `kind`; its three backend tests.
- [x] The schemas `cardamum-pimdir-queue-list`, `-show`, `-cancel`.
- [x] Fold the delta, log, CHANGELOG, MIGRATION.md, README.
