---
cairn: change
id: pimdir-engine-merge
status: landed
created: 2026-09-03
---

# pimdir: the backend runs on the merged io-pimdir

io-replica is retired. Its sync engine folded into io-pimdir, which now implements both parts of the pimdir standard, the store and the sync, under one `Pimdir*` prefix. Two things the backend leaned on went with the merge.

**The summary is typed.** `items.meta`, the `v: 1` JSON blob, and `ReplicaMeta` are gone. A store keeps one summary table per kind (pimdir STORAGE Annex A), a contact's row carrying its `UID`, `FN`, `KIND`, `ORG` and every `EMAIL` canonical, and a read that lists joins it as `PimdirItem::summary`. The backend was parsing the JSON back into a `PimdirCardMeta` of its own to render the listing preview, and passing a freshly derived one on every `add` and `update`.

**The owner derives.** `PimdirAction::Add` and `Update` carry no summary any more: the store's owner derives the key, the summary and the sort key from the body when it applies the action, through the same `io_pimdir::summary::derive` a sync uses. A producer handing over a summary was one more writer to keep in step, and `enqueue` no longer takes a timestamp either, SQLite stamping the row.

Alongside, `io_pimdir::conventions` is gone, its card derivation living at `io_pimdir::summary::contact`, the client types moved under `io_pimdir::client::{reader, producer, blobs}` with no crate-root re-export, and a store an earlier draft wrote is refused with `PimdirError::Stale` rather than reconciled.

## What changes

- `Cargo.toml`: the `pimdir` feature pulls io-pimdir alone, io-replica and humantime leave, and a `[patch.crates-io]` points io-pimdir at the sibling checkout until it is released.
- The backend imports the `Pimdir*` types from their modules, stages `add` and `update` naming the body alone, and reports the link id `io_pimdir::summary::contact::derive` yields for the bytes it staged.
- The listing scans through `list_summaries`, the read that joins the contact row, and the preview renders `PimdirSummary::Contact` rather than a JSON meta of its own, the unescaped name escaped back into a content line.
- `summary_of` and `now` go, the store parsing and stamping for itself.

## What does not change

The roles (reader with the pending overlay, a producer per write), the store's own hash naming a body, the write guards, the `seq` as the card id, the refusal of every `addressbook` write and the preview-versus-record split.
