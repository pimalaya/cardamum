---
cairn: log
change: pimdir-follows-storage-draft-10
date: 2026-11-13
---

# The pimdir backend follows storage-draft-10

cardamum builds against io-pimdir 0.11.0 from crates.io (storage-draft-10, sync-draft-10, search-draft-10; spec at pimdir ae8b231), the release neverest, himalaya and calendula move to. Capabilities **backends** and **commands** moved.

## What landed

- **Cargo.toml, Cargo.lock**: `io-pimdir = "0.11"`, from the registry, no `[patch.crates-io]` entry.
- **The three compile breaks** (src/pimdir/backend.rs, src/pimdir/index.rs): `PimdirAction::Remove` carries `if_match` and `resolves`; `enqueue_with_body_checked` is `enqueue_with_body`, which answers the same `PimdirEnqueued`; `PimdirSearchLocal::bodiless` is the per-kind `kinds`. Nothing else broke: cardamum matches on no `PimdirActionStatus` nor `PimdirMutation`, and its query scoping keeps a catch-all arm, so `PimdirQuery::Group` needed none.
- **A delete names the body it read** (STORAGE §15.3): `delete_card` states `if_match: if_match.and(item.object)` on its `remove`, as `update_card` does, and `resolves: None`. Without `--if-match` it states none and removes whatever the card holds, as before.
- **One gate per write** (STORAGE §15.6): `delete_card` dropped its `PimdirProducer::check` and logs the `partial` its `enqueue` answers, as `stage` does for a write carrying a body.
- **A write over a conflicted card parks** (STORAGE §15.2, §15.3): no code, cardamum stating no `resolves`. A tombstone settles no conflict since draft-10, so a `card delete` of a conflicted card, which used to settle it, is parked like a `card update`, and `pimdir status` lists it. The reader's overlay folds neither, so the card reads as it stood and a second write still passes `--if-match`.
- **Coverage** (src/pimdir/index.rs, SEARCH §8.2): `PimdirCardCoverage` gains `missing` and `unparseable`, summed over the kinds whose bodies a term of the query reads (`PimdirSearchLocal::reads`), so each field at rest still means `complete`, and the line under a list names them. `complete` is `PimdirSearchCoverage::is_complete` alone, the lag being part of it since 0.11. `encrypted` and `zoneDiffers` are left out: every cardamum query carries `kind:contact`, which reaches neither an encrypted body nor a calendar item.
- **`lock-replaced`** (src/error.rs): `ErrorCode::LockReplaced`, mapped in `of_pimdir`. Not mapped, no cardamum path raising them: `sync-version` (an owner's write), `store-reset` and `action-not-intent` (cardamum follows no handle and acknowledges no intent), `scope-invalid`, `backup-occupied`.
- **Store version**: a store of storage-draft-09 is refused as `store-version`, an index of search-draft-09 as `index-version`, neither migrated. The reinstall message reads `FORMAT_VERSION`, so it names `storage-draft-10` unchanged.
- **Empty `UID`** (STORAGE §9): a card whose `UID` is empty or blank is keyed `hash:` by io-pimdir's derivation, which `create_card` reports as it is; a composed card always carries a `UID`. No code.
- **Docs**: the spec folds below; README (the parked delete and conflicted writes, the unread bodies of a search); MIGRATION.md (a section on the pimdir store, the search commands and their coverage keys under New); CHANGELOG Unreleased, its pimdir lines being unreleased and so edited in place (io-pimdir 0.11, storage-draft-10, `lock-replaced`, the coverage keys) beside two new ones (the body a queued write names, the conflicted card).

## Spec

- backends, added: "A pimdir write over a conflicted card is parked".
- backends, modified: "A queued pimdir edit names the body it replaced" (the delete, with a scenario), "pimdir writes are staged queue actions" (`enqueue_with_body`, `enqueue` for a `remove`, one gate), "A pimdir body is named by the store's own hash" and "A duplicate pimdir create is refused" (the method's name), "pimdir derivations match the sync engine" (the empty `UID`), "A store refusal carries its code" (`lock-replaced`), "A pimdir store of another version is reinstalled" (`storage-draft-10` at io-pimdir 0.11, its scenario a draft-09 store).
- commands, modified: "A pimdir card search says what it did not cover" (`missing`, `unparseable`, `complete` as io-pimdir judges it, a scenario), "A JSON error carries a stable code" (the `lock-replaced` row), "pimdir searches cards through the store's index" (search-draft-10).

## Tests

- `a_delete_whose_body_a_sync_replaced_is_parked`: a delete staged with the version read, a sync replacing the body, then the drain parking the `remove`, the card keeping the sync's body and `pimdir status` listing the row. It fails with `if_match: None` on the `remove`, the old behaviour restated on the new crate (the drain reports `applied: 1, parked: 0`), the 0.10 code no longer compiling against 0.11.
- `a_write_over_a_conflicted_card_is_parked`: two sources diverging on a card as io-pimdir's own tests/item_conflict_settle.rs builds it (four pulls, the last answering the conflict), then an update and a delete naming the version read, both parked, the card unchanged.
- `an_unread_body_counts_when_a_term_reads_it`: a coverage counting a missing and two unparseable cards is complete, with no line, under `email:@example.org` and incomplete, naming both, under `has:phone`, the reads being `PimdirQuery::reads` of the parsed query. The coverage value is built by hand: no store here holds a body the index refused.
- `coverage_renders_as_camel_case_json` carries the two keys; `a_pimdir_failure_prints_the_code_io_pimdir_names` carries `PimdirError::LockReplaced`; the version fixtures name `storage-draft-09` and `search-draft-09`, the store and the index a draft-09 neverest wrote.
- 99 tests pass with every feature, 73 with `--no-default-features --features rustls-ring,pimdir`; `cargo check --all-features --tests` passes, and `cargo check --all-targets` for carddav, jmap, msgraph, gpeople, vdir and pimdir each alone with `rustls-ring`; clippy with `-D warnings` is clean with every feature and with pimdir alone.

## Not in this entry

- The suites under tests/ reaching a provider, and an end-to-end run on a store a draft-10 neverest synced: the owner's, once the stores are reinstalled.
- The audit lines the CLIs still owe (pimdir cairn/changes/audit-2026-11-12/tasks.md): C2 (`enqueue_foreign_body` for a card the user did not write, which needs the provenance of a write), C4 (the handle of a queued row in receipts and in `pimdir status`, a bare row id staying accepted), C5 (the store rules hoisted into io-pimdir). C6 is answered here for cardamum: one gate per write.
- Refusing a write over a conflicted card at staging, and a command resolving a conflict.
- The commit.
