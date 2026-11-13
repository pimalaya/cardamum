---
cairn: tasks
change: audited-dependencies
---

# Tasks

- [x] Cargo.toml: io-http 0.7, io-oauth 0.6 (dev), io-jmap 0.8, io-msgraph 0.8, io-gpeople 0.7, io-pim-discovery 0.12, io-vdir 0.3, io-webdav 0.8; Cargo.lock holding one io-http and one io-oauth
- [x] Google People: `merge_unprojected` in `update_card`, own `clientData` merge removed, a warning for the dropped properties in `create_card` and `update_card`
- [x] Scripted tests over `GpeopleClientStd::new`: the update keeps a second organization, a sub-field and a foreign `clientData` entry (fails without the merge); the create warns of a `PHOTO` over 8 KiB
- [x] Microsoft Graph: next pages through `page_get`
- [x] JMAP: a cleared address book description sent as `null`; the `Other` set error named
- [x] CardDAV: `carddav put` prints the id the write answered
- [x] `cargo fmt`, `cargo test --all-features`, clippy `--all-targets -D warnings` over the eleven feature sets of CI, `cargo test --no-default-features --features rustls-ring,vdir`
- [x] CHANGELOG; spec (backends, projection); log; land; archive
