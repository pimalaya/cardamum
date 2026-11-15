---
cairn: log
change: pimdir-follows-storage-draft-13
date: 2026-11-15
---

# The pimdir backend follows storage-draft-13

cardamum builds against io-pimdir 0.14.0 from crates.io (storage-, sync- and search-draft-13, pimdir 1c6442e), d184622 plus its release commit; the git pin at d184622 that served while it was unreleased is gone, no `[patch.crates-io]` remaining. Task N31 of cli-alignment-2026-11-15. Capabilities **commands** and **backends** moved.

## What landed

- **The refusals** (pimdir STORAGE §15.4): `pimdir queue show` prints, on an applied row, the refusals standing on the card it acted on, `refusals: [{collection, source, reason, refusedAt}]` and a line `Refused by <source> in <collection> at <refusedAt>: <reason>` each, the state staying `applied`. `pimdir status` lists every refusal standing in the account's address books, `refused: [{addressbookId, itemId, source, reason, refusedAt}]`, and in text `CARD  ADDRESSBOOK  SOURCE  REASON` then `Refused by the server, held until the card is edited or removed`. The binding's handle and the refused change's key stay out: the engine's, not the user's.
- **A receipt names its card for every kind** (STORAGE §15.2): `seq` of an applied update or remove names its card too, so the text says `on card <seq>`, not `creating card <seq>`.
- **Write receipts unchanged**: they print at enqueue time, before any push.
- **No other compile break**: cardamum matches no `PimdirPushOutcome` (now `#[non_exhaustive]`), writes no `PimdirWriteOp` outside its tests, and reads no sync report.
- **Store version**: a draft-11 store is refused `store-version`, the message naming `storage-draft-11`, `storage-draft-13`, the drain, the store moved aside and `neverest init`.

## Tests

- `a_refused_push_shows_on_its_row_and_in_the_status`, new: a drained update, then `PimdirWriteOp::RefusePlacement` written as a sync engine would; `queue show 1` reads applied on the card with the refusal, `status` lists it. Seen failing with the status read taken out (`refused` empty).
- `an_applied_row_prints_the_refusals_on_its_card`, new: the JSON and the text of `queue show`.
- Moved to draft-11: the store- and index-version fixtures (`a_store_of_another_version_asks_for_a_reinstall`, now asserting the rebuild, `an_index_of_another_draft_is_refused_as_index_version`, the code tests in src/error.rs).
