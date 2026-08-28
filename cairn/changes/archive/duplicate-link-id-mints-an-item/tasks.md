---
cairn: tasks
change: duplicate-link-id-mints-an-item
---

# Tasks

- [x] Bump io-pimdir (and io-replica through it) to the releases carrying the minted key and no ambiguity surface. None of the three is released, and the `[patch.crates-io]` entries pointed at git revisions predating the change, so they point at the sibling checkouts instead: `io-pimdir.path = "../io-pimdir"`, `io-replica.path = "../io-replica"`, `io-webdav.path = "../io-webdav"`. The whole local suite is testable end to end that way.
- [ ] Point the three `[patch.crates-io]` entries back at git, or drop them for the crates.io versions, once io-pimdir, io-replica and io-webdav are released. Cardamum cannot be published while they resolve to a sibling path.
- [x] Audit src/pimdir/ for a read that assumes one card per identity: none found. `get_card`, `update_card` and `delete_card` all resolve a `seq` through `item`, and `scan_items` pages on `(sort_key, seq)`. Nothing reads `ambiguous_handles` or `ReplicaStatus::Ambiguous`, so io-pimdir's removal is not breaking here.
- [x] `create_card` needs no change: it stages `card::derive`'s bare `UID` and returns it for display only, and a collision parks in the store rather than minting.
- [x] The card preview emits `UID:` from the summary, so a listing may now legitimately show two rows carrying one `UID`. Confirmed nothing downstream dedupes or groups a listing by that value: `CardRow::from` builds one row per card positionally and never reads a `UID`, `vcard_preview` pulls `FN`, `EMAIL` and `TEL` only, and `paginate` slices the vector. The NOTE sits in `preview_vcard`, on the `UID:` line it emits, which is where the value enters the listing and where a reader would reach for a dedup.
- [x] Correct the argument docs that call the shared `id` a "Card UID" in the read, update and delete commands: for the pimdir backend it is the public `seq`, which the spec already says, and a `UID` no longer identifies a card even in principle. All three now read "Card identifier, as `card list` reports it. It is the backend's own id, not the vCard `UID`, which names no card on its own", which stays protocol-agnostic.
- [x] Tests: `two_items_sharing_a_uid_project_two_distinct_cards` in src/pimdir/backend.rs. Two vCards carrying one `UID` derive the one bare link id (which is why the store mints), and the two items the store hands back, one under the bare key and one under a minted `dup:` one, project two distinct previews both stating that `UID`, with no minted key leaking into either. The parking behaviour is the store owner's and is tested in io-pimdir.
- [x] `cargo test` (56 passed), `cargo clippy --all-targets` (clean on the default set and with `pimdir`), `cargo fmt`.
- [x] CHANGELOG `### Fixed`: a duplicated card no longer vanishes from the pimdir backend, plus a `### Changed` line for the reworded card argument.
- [x] Fold `delta.md` into `cairn/spec/backends.md`; append the log entry; mark `landed`.
