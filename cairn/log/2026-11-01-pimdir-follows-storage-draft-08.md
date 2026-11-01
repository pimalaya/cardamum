---
cairn: log
change: pimdir-follows-storage-draft-08
landed: 2026-11-01
---

# The pimdir backend follows storage-draft-08

cardamum builds against io-pimdir bff5737 (storage-draft-08, sync-draft-07, search-draft-08), still through the `[patch.crates-io]` path to ../io-pimdir. Cargo.lock moves io-pimdir's own tree alone: rusqlite 0.40.2 to 0.41.1, libsqlite3-sys 0.38.2 to 0.39.1, encoding_rs, jiff-tzdb and rustix in, sha2 out of io-pimdir's dependencies. Capabilities **backends** and **commands** moved.

## What landed

- **The compile port**: `PimdirBlobs::path` answers `io::Result<PathBuf>` (the tests unwrap it, src/pimdir/backend.rs); the error-code test builds `Version { found: Option<String> }`, `HashAlgo` without `declared`, `Damaged`, `Exposed` and `Action` (src/error.rs).
- **The code vocabulary** (src/error.rs, `ErrorCode`, the `cardamum-error` schema following it): `store-stale` and `store-newer` out, never released; `store-version`, `store-exposed` and `action-malformed` in. `flag-invalid` stays out: cardamum stages no flag (its `add` carries the empty set), so no cardamum path raises it, as with `collection-queued` and `collection-role` before.
- **A store of another version asks for a reinstall** (src/pimdir/client.rs, `refusal` and `reinstall`): `store-version` carries cardamum's own message, naming the store's version and `FORMAT_VERSION`, and the neverest reinstall (that neverest's sync until nothing is queued, neverest upgraded, the store moved aside, `neverest init`, a sync), or a cardamum upgrade for a later store. io-pimdir's own message says "move it aside and sync a new one", which skips the drain the queued writes need.
- **`pimdir status` carries the code of a queue it cannot read** (src/pimdir/backend.rs, `status`): `count_pending_creates` decodes every pending row of the address book and fails on a payload its schema refuses (`PimdirError::Action`); the failure now goes through `refusal`, so it prints `action-malformed`.
- **A blob directory where a body belongs reads as absent**: io-pimdir now opens a body without following a link and reads anything but a regular file at its path as a stray (pimdir STORAGE §7). `a_page_reads_no_body_but_its_own` poisoned bodies by turning them into directories, which now list as unfetched; it removes every permission from the blob instead (Unix only).
- **Spec**: backends "A store refusal carries its code" and "A pimdir body is named by the store's own hash" (BLAKE3 alone) modified, "A pimdir store of another version is reinstalled" and "A pimdir store others can read is refused" added; the commands code table follows. README and CHANGELOG say the same.

## Tests

- `a_store_of_another_version_asks_for_a_reinstall` rewrites `storage-draft-08` into `storage-draft-07` in the closed pimdir.db (same length, the WAL checkpointed first), then expects `store-version` and the reinstall message. With `refusal` relaying io-pimdir's message it failed on "until nothing is queued".
- `a_store_others_can_read_is_refused` sets the store directory `0755` and expects `store-exposed`; without the mapping it failed with no code.
- `a_malformed_queued_action_carries_its_code` rewrites a queued `add`'s `"v":1` into `"w":1`, then expects `pimdir status` to fail with `action-malformed` and, once the owner drained (and parked) the row, to report it parked; with `count_pending_creates` under a bare `?` it failed with no code.
- Each regression test was checked failing by reverting its production line in place (the message, the status mapping, the `store-exposed` arm), then restoring it.
- 85 tests pass with the default features and with every feature, 52 with `--no-default-features --features rustls-ring,pimdir`; `cargo check --all-targets` passes for each backend alone with `rustls-ring`; clippy is clean with every feature and with pimdir alone.

## Not in this entry

The end-to-end run on a scratch store neverest synced (it waits on neverest-follows-sync-draft-07), the git pin to io-pimdir bff5737, and the commit.

## Supervisor follow-up (2026-11-01)

- Pinned to io-pimdir bff5737 by git rev, the path patch dropped; `cargo test --all-features` green on the pin (85 passed).
- End to end on a scratch store neverest synced from local Stalwart IMAP and Radicale DAV (not a provider test account): card list, read, a create, the `item-duplicate` refusal of a second one, and an update; the writes queued here, then pushed by `neverest sync`, each write found on its server.
