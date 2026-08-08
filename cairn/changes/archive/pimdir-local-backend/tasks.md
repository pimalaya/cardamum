---
cairn: tasks
change: pimdir-local-backend
---

# Tasks

- [x] Cargo `pimdir` feature plus io-pimdir (client) and io-replica git patches; added to `default`.
- [x] `Backend::Pimdir` plus `allows_pimdir`, `FromStr` and `Display` arms.
- [x] `config`: `PimdirConfig { root, source, account }` and `AccountConfig.pimdir`.
- [x] src/pimdir/hash.rs: the 128-bit FNV-1a content hash, matching Neverest and Himalaya.
- [x] src/pimdir/card.rs: link id, `v: 1` summary and sort key, matching Neverest's `text/vcard` kind, with its scanner tests plus a sort-key test.
- [x] src/pimdir/client.rs: open the store as a source (auto-detected), shell-expand the root, optional account scoping, blob reader.
- [x] src/pimdir/backend.rs: the nine shared operations, keyset scan, preview vCard for unhydrated cards, staged mutations behind a synced-source guard.
- [x] shared/client.rs: `BackendClient::Pimdir` and all nine dispatch arms, constructed local-before-network.
- [x] account/check.rs: `connect_pimdir` opening the store and listing collections.
- [x] wizard/local.rs: `pimdir.db` and vdir-tree markers, prompt only when inconclusive.
- [x] config.sample.toml `[accounts.pimdir-example]` block; src/main.rs header.
- [x] `cargo build/test/clippy --all-features`, feature matrix including `--features rustls-ring,pimdir` alone, `cargo fmt`.
