---
cairn: change
id: pimdir-follows-storage-draft-08
status: landed
created: 2026-11-01
---

# The pimdir backend follows storage-draft-08

## Why

io-pimdir bff5737 (pushed, master, still 0.8.0) follows pimdir d671bbd: storage-draft-08, sync-draft-07, search-draft-08 (io-pimdir cairn/changes/the-crate-follows-storage-draft-08). cardamum reads and queues behind neverest.

Built on 2026-11-01 against ../io-pimdir at bff5737 (`cargo check --all-targets`, over the uncommitted work of 2026-10-30), 7 errors:

- `PimdirError::Stale` is gone, `HashAlgo` has no `declared`, and the code mapping expects `Option<String>` (src/error.rs:102, 104, 111).
- `PimdirBlobs::path` answers `io::Result<PathBuf>` (src/pimdir/backend.rs:1053, 1054, 1120, 1136).

## What the compiler does not see

- **Error codes**: io-pimdir no longer raises `store-stale` or `store-newer`; it adds `store-version`, `store-exposed`, `flag-invalid`, `action-malformed`. A store an earlier neverest wrote is refused as `store-version`: the message points at neverest's reinstall, never at deleting the store.
- **Modes**: a store another user can read is refused as `store-exposed`.
- `PimdirError` is `#[non_exhaustive]`; cardamum already keeps a wildcard arm (pimdir-adopts-the-io-pimdir-release-api).

## Pin

Built against `path = "../io-pimdir"` while porting, then `[patch.crates-io]` by git rev bff5737; the requirement stays `0.8` until the user releases io-pimdir. This also closes the release line of cairn/changes/pimdir-release-api-followups once released.
