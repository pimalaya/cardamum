---
cairn: change
id: a-missing-pimdir-body-is-pending
status: landed
created: 2026-11-18
---

# A missing pimdir body is pending

## Why

Audit line TRI-3 of the CLI family analysis (2026-11-15) and the owner ruling of 2026-11-18. A card whose item names a body the blob tree no longer holds (a `pimdir item erase`, a quarantine, a purge) was a `card-unreadable` entry, no row listed, exit 3, and its `card read` failed uncoded ("Body blob missing for ..."). After an erase every listing of the address book exited 3 until a resync. pimdir's own guide says such a body "is not hydrated and not an error" (pimdir GUIDE §14), and Himalaya lists the row as usual and answers `body-pending` on read. The ruling: the guide and Himalaya win over the twin specs.

## What

- `card list` and `pimdir search` list such a card as usual, as an unfetched one: its summary preview, `etag` `null`, no `errors` entry, no exit 3.
- `card read` fails coded `body-pending` with the sentence of an unfetched card, ``Card `<id>` in `<addressbook>` is not downloaded yet (body not fetched); run a sync to hydrate it``, exit 1, the failure object Himalaya prints for a message in the same state.
- A body that cannot be read (an I/O error) stays a `card-unreadable` entry.
- calendula takes the same change (its `a-missing-pimdir-body-is-pending`).
