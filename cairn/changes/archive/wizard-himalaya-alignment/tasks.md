---
cairn: tasks
change: wizard-himalaya-alignment
---

# Tasks

- [x] wizard/secret.rs: rewrite over `pimalaya_cli::wizard::keyring` as `configure_password` / `configure_token`, with the argv and shell `Secret` builders and their two tests.
- [x] wizard/search.rs: replace `DiscoveredAuth` with `AuthCaps`, fold methods with `caps_of`, one entry per service via `push_entry`, `compose_all_within` with an 8s deadline, rank without the auth tiebreaker.
- [x] `wizard/{carddav,jmap}.rs`: caps-driven `prompt_auth`, skipped when one scheme qualifies; delete `carddav::configure_manual`.
- [x] `wizard/{msgraph,people}.rs`: take the account name, seed the broker handle from it.
- [x] wizard/local.rs: new, marker-based local backend detection.
- [x] wizard/discover.rs: welcome banner, `save_or_print` + `default_config_path`, `Outcome`, derived account name, `default = false`, `retain_scheme`, `stop_undiscovered`, JSON/non-TTY passthrough.
- [x] Cargo.toml: pimalaya-cli `carddav`, `jmap` and `wizard` features.
- [x] config.sample.toml header and src/main.rs header rewritten to the new flow.
- [x] Tests: account-name derivation (domain label, path component), unknown-scheme rejection, caps folding, one-entry-per-service.
- [x] `cargo build/test/clippy --all-features`, feature matrix, `cargo fmt`.
