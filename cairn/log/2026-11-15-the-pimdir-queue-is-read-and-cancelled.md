---
cairn: log
change: the-pimdir-queue-is-read-and-cancelled
date: 2026-11-15
---

# The pimdir queue is read and cancelled

`pimdir queue list`, `pimdir queue show <ROW>` and `pimdir queue cancel <ROW>` read and retract the account's queue as Himalaya's do: the card creations no listing shows before a sync, the parked rows, where one row stands, and the cancellation, the one write cardamum makes in the store owner's role. Task C5 of cli-alignment-2026-11-15 (finding F24), twin of calendula's K7. Capabilities **commands** and **backends** moved.

- **src/pimdir/queue/**: `list` (`-k` narrowing to one address book, every one by default as `pimdir count` and `search`), `show` and `cancel` ported from himalaya/src/pimdir/queue/, keys `row`, `kind`, kebab-case states; a queued card carries `collection`, `kind`, `linkId` and `fullName` where Himalaya's message carries its envelope, as calendula's twin listing carries `summary`.
- **src/pimdir/backend.rs**: `queued_cards` (io-pimdir's `pending_creates`, the full name derived from the pinned body), `parked_rows`, `queue_row` (a row outside the account's address books reads `unknown`), `cancel_queued` (`PimdirStore::open_current` then `drop_action`, a sync holding the store refused `store-owned` at once).
- **`pimdir status`**: stays its own command, not a hidden alias of `queue list`, its output differing: it reports the accounts and each address book's cards and downloads, which no queue command carries. Its parked rows key their kind `kind`, where they read `action` (new since v0.3, nothing breaks).
- **Evidence**: `a_queued_card_lists_in_the_queue_until_cancelled`, `an_applied_creation_names_its_card_and_is_not_cancelled`, `a_parked_row_lists_with_its_kind` over a scratch store; the list, show and cancel output tests.

CHANGELOG (Added), MIGRATION.md ("New", "The pimdir store"), README. Spec: commands ("The pimdir queue is read and retracted" added, "Specific commands mirror their protocol" and "pimdir status reports the store and its parked actions" modified), backends ("pimdir takes the reader and producer roles, the owner's for one cancellation" replacing "…, never the owner").
