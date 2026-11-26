---
cairn: change
id: follows-storage-draft-17
status: landed
created: 2026-11-23
---

# cardamum follows storage-draft-17

> Landed on 2026-11-23 (log 2026-11-23-follows-storage-draft-17); its 0.20.0 release steps K1 to K3 were superseded by the move to io-pimdir 1.0.0 (log 2026-11-26-follows-pimdir-v1).

The shared frame (target, edits, git, CPUs, locks, accounts, decision rules) is pimdir cairn/changes/the-consumers-follow-draft-17/frame.md; this file holds what is cardamum's alone. Every figure cited is in inventory.md.

## Why

cardamum reads the store and index neverest writes and queues card writes neverest drains. neverest and himalaya are moving to io-pimdir 7f16b27 (storage-draft-17, sync-draft-15, search-draft-17); a cardamum left on 0.19.1 (storage-draft-15) refuses every store they write (`store-version`, "upgrade cardamum", inventory I6), and the reverse holds.

## What

1. **The pin and the features.** io-pimdir by the frame's `[patch.crates-io]` git pin at 7f16b27. Since `search` now implies `client` (SQLite), the network backends and vdir take `contact` (the card matcher) in place of `search`, `pimdir` takes `client`, `search` and `contact`, and a dev-dependency takes `indexer` for the tests building an index. The release binary loses ical-rs and mime-rs; no backend but pimdir links SQLite.
2. **The 15 compile breaks** (6 in the binary, 9 in tests, inventory I2): the sort renamed (`Name` to `Title`), `PimdirAction::SetProperty`, the matcher behind `contact`, the index refresh behind `indexer`, `server_object_id` in three test placements.
3. **What users meet**, pinned by tests: a draft-15 store refused with io-pimdir's message and the start-over path; the reserved forms `~`, `/`, `{` refused `query-refused`; `label:` and `property:` answered on pimdir and refused `query-unsupported` elsewhere; `mid:`, instants and `min`, `h`, `mo` accepted where they were refused; `set-property` rows listed in the queue.
4. **Bugs** on the pimdir surface: the doubled `Error:` of a query refusal (TRI-7, reproduced); an undecodable queue row hiding its address book's rows, parked on io-pimdir (needs-io-pimdir.md N1); the other open lines re-checked closed (tasks G).
5. **Performance**: a baseline on 0.19.1 and the same measure after, on 10k and 100k card stores the generator writes at both drafts from one seed, with two cardamum-side fixes kept only if measured.
6. **Docs**: spec fold, log, CHANGELOG, README start-over note.

## Scope

In: every item above, the `cargo deny` red at HEAD (url 2.5.8, RUSTSEC-2026-0592) since it fails the push's Audit job, and the cross-app check with neverest.

Out, with the reason:
- card-search's open lines (the `partial` flip of the four network backends after a live measure, the live cells, answering #25): network and live, not the pimdir surface; that change stays active. Its pimdir half landed 2026-11-18; this change only moves its feature line and its store-term list. The offline vdir matcher measure (H) feeds its ruling.
- Labels and properties as card data (D3).
- The `pimdir` binary's JSON contract: cardamum never runs it.
- Sync items of the frame (`settled_collections`, `LandPlacement`, `server_object_id`, `body_flush`): cardamum runs no engine.
- Engine defects R1, R4, K4: needs-io-pimdir.md, last section.

## Decisions (defaults taken)

- **D1** Features as What 1. Not user-visible.
- **D2** `--sort` on `pimdir search` and `card search` takes SEARCH §8.2's names, `title` and `relevance`, with `name` kept as a hidden alias of `title` so scripts keep working: the owner's ruling of 2026-11-21 (five named orders), as himalaya already does and calendula's plan does (its D1). Owner option: keep `name` as the shown value.
- **D3** Labels and properties: no command and no output key now; the query fields alone, as io-pimdir answers them. Unified outputs carry no backend details (backends.md), and a label command is a family ruling for the owner.
- **D4** `set-property` rows list, show and cancel as any row (forced by the exhaustive match).
- **D5** No new code relayed for `property-invalid`: cardamum stages no property.
- **D6** Existing stores: no migration (frame §2); the start-over note in README and CHANGELOG, with io-pimdir's refusal text as the only message.
- **D7** url 2.5.9 in its own commit, before the migration's.

## Risks

1. **Release order**: a cardamum on another io-pimdir than neverest's refuses the user's store, either way (I6). Every CLI on one revision before any release (tasks K).
2. **The pin in a release**: `cargo publish` drops `[patch]`, so a tag on the pin would publish a cardamum depending on 0.19.1 from crates.io; the release waits on io-pimdir 0.20.0 (K).
3. **Feature regressions hidden by the default build**: a single-backend set may break while the default passes; the 11 clippy sets and the two CI test steps gate every step (all clean on the probe, I3).
4. **Two slower walks**: at 10k cards every read got faster (`pimdir search` 29 to 5.5 ms) but `card list --page 200` (+24%) and `pimdir status` (+34%) regressed, both walking `list_summaries` (I10): io-pimdir's to fix (needs-io-pimdir N3, N4), linear at 100k, which was not built while planning.
