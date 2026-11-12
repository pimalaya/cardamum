---
cairn: change
id: dependencies-trimmed
status: landed
created: 2026-11-12
---

# Dependencies trimmed

## Why

Fifth change of the-family-standards: Cardamum's dependencies take Himalaya's and Ortie's shape (himalaya 2026-11-11-dependencies-trimmed), each naming the features it needs, the tree checked by `cargo deny`.

## What

- Every dependency, dev ones included, takes `default-features = false` and names its features: anyhow `std`, schemars `derive` and `std`, serde `derive` and `std`, serde_json `std`, url `serde` (`carddav.home` holds a `Url`), toml `parse` and `serde`; ariadne, getrandom, log, secrecy and tempfile take none. clap_complete, clap_mangen, env_logger and percent-encoding, which cardamum's own code never names (pimalaya-cli carries the first three), leave the manifest.
- base64 0.23 gives way to base64-codec 0.1 for the JMAP `Basic` header; `basic_auth_is_standard_padded_base64` (src/jmap/backend.rs) passed on base64 before the switch.
- `main` calls `SimdCpu::install_detected()` first (simd-cpu 0.1), so vcard-rs's codecs run the detected backend.
- `idna_adapter = "=1.0.0"` drops the ICU4X crates: an `http` or `https` server with a non-ASCII host is refused when parsed, its Punycode form working; `a_non_ascii_host_is_refused` (src/config.rs) failed on the ICU4X backend.
- deny.toml lists the release targets under `[graph] targets`; the allowed licences are the ones the tree carries (Apache-2.0, BSD-3-Clause, ISC, MIT, MPL-2.0, Unicode-3.0, Zlib), `Apache-2.0 WITH LLVM-exception` and CDLA-Permissive-2.0 leaving.

## Scope / non-goals

The build with no backend at all, which fails clippy on dead code, is every-feature-set-builds-clean's.
