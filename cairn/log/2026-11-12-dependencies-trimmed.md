---
cairn: log
change: dependencies-trimmed
date: 2026-11-12
---

# Dependencies trimmed

Every dependency names its features with `default-features = false`; clap_complete, clap_mangen, env_logger and percent-encoding left the manifest. base64-codec replaces base64 for the JMAP `Basic` header, guarded by `basic_auth_is_standard_padded_base64` (passed on base64 before the switch); `main` installs the detected CPU token; `idna_adapter = "=1.0.0"` removed the ICU4X crates, guarded by `a_non_ascii_host_is_refused` (failed on ICU4X). Cargo.lock lost 307 lines (29 packages, base64 0.23 and the ICU4X and anstream crates among them). deny.toml lists the release targets and the licences the tree carries; `cargo deny check` passes, warning on getrandom 0.2 (ring) beside 0.4.

89 unit tests and 8 first-run tests pass; clippy `-D warnings` passes on all features and on each backend alone. The build with no backend fails clippy on dead code, every-feature-set-builds-clean's to fix.

Capabilities moved: packaging (added "Dependencies name their features", "Dependency checks pass clean", "Hosts are ASCII").
