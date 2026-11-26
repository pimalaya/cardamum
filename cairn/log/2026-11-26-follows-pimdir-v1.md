---
cairn: log
change: follows-pimdir-v1
landed: 2026-11-26
---

# cardamum follows pimdir v1

Cardamum moves from io-pimdir 0.20.4 (storage-draft-17, search-draft-17) to io-pimdir 1.0.0 (storage-v1, sync-v1, search-v1). Both requirements read `version = "1.0.0"`, from crates.io.

- **Stores**: a store any io-pimdir 0.x wrote is refused `store-version` in io-pimdir's wording, never migrated: its user syncs it with the last 0.x engine until nothing is queued, moves its directory aside and lets a 1.x engine sync a new store. A store a later release wrote, such as `storage-v2`, asks for a later cardamum. An index of an earlier version is the sync engine's to rebuild, as before.
- **API**: `PimdirSearchCursor` and `PimdirSearchSort` are imported from `io_pimdir::search::plan`. `PimdirStore::for_source` returns a `Result`, taken in the tests alone (no production path opens the owner role). No new error code reaches cardamum: it stages no copy nor move (`target-unsynced`) and opens no source (`source-empty`).
- **Tests**: the store and index version tests mark the format version through rusqlite, a new dev-dependency, since `storage-v1` and `search-v1` are shorter than any draft marker and an in-place byte rewrite cannot reach one.
- **Evidence**: fmt, clippy `-D warnings` over the CI feature sets, `cargo test --all-features` and the two single-backend CI test steps pass.

Capabilities moved: backends (store version, `storage-v1`), commands (`index-version` wording, search-v1), packaging (rusqlite as a test dependency).
