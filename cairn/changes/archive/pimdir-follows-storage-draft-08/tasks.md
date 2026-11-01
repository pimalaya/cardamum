---
cairn: tasks
change: pimdir-follows-storage-draft-08
---

# Tasks

- [x] The 2026-10-30 work committed first (user, 2026-11-01)
- [x] The compile port; Cargo.lock moves io-pimdir alone; `cargo test` green; `--no-default-features --features pimdir,rustls-ring` builds alone (AGENTS.md feature matrix)
- [x] Error vocabulary and messages (`store-stale`, `store-newer` out; `store-version`, `store-exposed`, `flag-invalid`, `action-malformed` in), tested where cardamum can build the store (`store-version`, `store-exposed`, `action-malformed` on `pimdir status`); `flag-invalid` left out, no cardamum path stages a flag
- [x] End to end on a scratch store neverest synced from a test account: card list, read, create, update, delete queued, drained by neverest (2026-11-01, on local Stalwart IMAP and Radicale DAV rather than a provider test account: every queued write reached its server after `neverest sync`)
- [x] Pin to io-pimdir bff5737 by git rev, path patch dropped
- [x] Fold into spec/backends.md and spec/commands.md, README, CHANGELOG; log
- [x] Land
