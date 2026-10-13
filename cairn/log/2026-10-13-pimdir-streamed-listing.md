---
cairn: log
change: pimdir-streamed-listing
landed: 2026-10-13
---

# A pimdir card listing streams the store instead of loading it

`list_cards` in `src/pimdir/backend.rs` walks the address book through `scan`, one keyset batch of rows (500) at a time: the cards before the page are counted off their rows, only the page's cards are built with their bodies, and the walk stops once the page is full. A listing naming no page size still reads every body, one at a time, and holds them all, as its output does. `paginate` is now built only for the backends that cut a whole list.

Equivalence: a test lists 49 page and size pairs of an address book spanning several batches, before and after a staged update and removal, against the whole-read listing; another makes the bodies before and after a page unreadable and lists the page. The `--json` output of 266 commands (`card list` at six sizes and six pages, `card read` of every card) is byte-identical between `d64325a` and this change on a store of 230 cards with a staged update, removal and create.

Capabilities moved: **backends** (a pimdir card listing streams the store, added).
