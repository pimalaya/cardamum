---
cairn: change
id: pimdir-follows-storage-draft-10
status: landed
created: 2026-11-13
---

# The pimdir backend follows storage-draft-10

## Why

io-pimdir 0.11.0 implements pimdir storage-draft-10, sync-draft-10 and search-draft-10 (spec at pimdir ae8b231), and neverest, himalaya and calendula move to it together: a store opens under its own draft alone, so a cardamum left on io-pimdir 0.10 is refused by every store a draft-10 neverest writes. Three of its breaking changes stop cardamum compiling, and four more change what a cardamum write or search means without the compiler saying so.

## What

- **Build**: io-pimdir 0.11 from crates.io, no patch. `PimdirAction::Remove` gains `if_match` and `resolves`, `enqueue_with_body_checked` is gone (`enqueue_with_body` answers the same), and `PimdirSearchLocal::bodiless` is the per-kind `kinds`.
- **A delete names the body it read** (STORAGE §15.3): `delete_card` states `if_match` on the `remove` it queues, the version `--if-match` matched, as `update_card` does, so the owner parks a delete decided over a body a sync replaced.
- **One gate per write** (STORAGE §15.6): `delete_card` no longer runs `PimdirProducer::check` before `enqueue`, which now answers the partial supports its own gate computed.
- **A write over a conflicted card parks** (STORAGE §15.2, §15.3): a tombstone no longer settles a conflict, and cardamum states no `resolves`, having no command deciding a conflict. Nothing to code: the spec, the README and the migration guide say it, and a test holds it.
- **Coverage** (SEARCH §8.2): `missing` and `unparseable` join the flat coverage of `pimdir search` and `pimdir count`, counted when a term of the query reads card bodies, so an answer they leave incomplete names its reason; `complete` is io-pimdir's `is_complete`, which now includes the lag.
- **`lock-replaced`**: the code of a producer whose staging lock file was replaced before its enqueue (STORAGE §8), raised on a cardamum write.
- **Wording**: the spec and the comments name `enqueue_with_body`, the store version `storage-draft-10`, the search language `search-draft-10`, and the `hash:` key of a card whose `UID` is empty.

## Out of scope

The audit lines the CLIs still owe (pimdir cairn/changes/audit-2026-11-12/tasks.md): C2, stripping `X-PIMDIR-` properties from a resource the user did not write, which needs the provenance of a write; C4, the handle of a queued row in receipts and in `pimdir status`; C5, hoisting the store rules the three CLIs restate into io-pimdir. Refusing a write over a conflicted card at staging, and a command resolving a conflict, are their own changes.
