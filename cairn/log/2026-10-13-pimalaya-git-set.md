---
cairn: log
change: pimalaya-git-set
landed: 2026-10-13
---

# io-pimdir 0.8, io-msgraph 0.6, io-gpeople 0.5, and the unreleased Pimalaya set

io-pimdir moves from the git revision `be03184` to 0.8, io-msgraph from 0.4.3 to 0.6 and io-gpeople from 0.4.2 to 0.5, and `[patch.crates-io]` pins io-http (`e181530`), io-msgraph (`2ae2637`), io-pimdir (`189db40`) and io-webdav (`62fad89`), the pushed revisions the Pimalaya Android client builds, so cardamum opens the same store schema as the neverest, himalaya and calendula built beside it. The patches go once those revisions are released. `Cargo.lock` holds one io-http and one io-pimdir; io-jmap stays at 0.4, which builds against the patched io-http.

What the revisions change for cardamum: io-http bounds a response head at 64 KiB and a body at 256 MiB, failing with a typed error past either, and fails on a connection closed mid-body; io-webdav parses a multistatus without quadratic copies, nesting capped at 64. io-gpeople 0.5 carries Google's error envelope in `GpeopleSendError::Api`, which cardamum only propagates. cardamum's code is unchanged: it reads `PimdirSummary` through a `let ... else` on its contact arm alone, which the new file kind leaves whole, and calls none of the mail reads io-pimdir 0.8 changed.

Verification: `cargo test` with all features, the default features, and `--no-default-features --features pimdir` as MOA builds it (67, 67 and 35 tests); `cargo clippy --all-targets -- -D warnings` on the same three sets; `cargo fmt --check`; `carddav`, `jmap`, `msgraph`, `gpeople` and `vdir` each alone with `rustls-ring`.

Capabilities moved: none.
