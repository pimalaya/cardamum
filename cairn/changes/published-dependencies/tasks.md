---
cairn: tasks
change: published-dependencies
---

# Tasks

- [x] Check each pinned revision is inside the release it is replaced by (io-http, io-msgraph, io-pimdir, pimalaya-cli)
- [ ] Cargo.toml: io-jmap 0.7, io-msgraph 0.7.1, io-gpeople 0.6, io-pimdir 0.10, pimalaya-cli 0.3.2 (build and normal), `[patch.crates-io]` gone; Cargo.lock holding one io-http and one vcard-rs
- [ ] Fix any compile break the releases bring, without moving a behaviour of cardamum's own
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -D warnings` (default and all features), `cargo test --all-features`, `json-schema -d` structurally identical
- [ ] CHANGELOG: released versions in place of the pin clauses
- [ ] Add cairn/spec/packaging.md with "Released dependencies, patched only while unreleased"
- [ ] Log; land; archive
