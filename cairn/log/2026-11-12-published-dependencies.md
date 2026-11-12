---
cairn: log
change: published-dependencies
date: 2026-11-12
---

# Built on the published dependencies

Cargo.toml requires io-jmap 0.7, io-msgraph 0.7.1, io-gpeople 0.6, io-pimdir 0.10 and pimalaya-cli 0.3.2 (build and normal), and `[patch.crates-io]` is gone: every pinned revision was inside the release replacing it. Cargo.lock holds one io-http (0.6.0) and one vcard-rs (0.6.0). No compile break came with the releases; clippy `-D warnings` passes on the default and all-features builds, the 115 tests pass, and the 87 `json-schema` files are identical (jq-sorted) to the 2026-11-08 build. The CHANGELOG names the released versions.

Capabilities moved: packaging (new, "Released dependencies, patched only while unreleased").
