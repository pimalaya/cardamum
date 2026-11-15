---
cairn: log
change: a-backend-left-out-is-named
date: 2026-11-15
---

# A backend left out of the build is named

The block of a backend the build leaves out still loads unread, and a command reaching that backend now fails coded `backend-not-built`, naming the cargo feature (owner ruling 2026-11-15, round 2 item 16). Capabilities **backends**, **config** and **commands** moved.

- **src/backend.rs**: `NAMES` (the resolution order, each name its cargo feature), `is_built` and `not_built`, the coded failure "This build leaves out the `<name>` backend, rebuild with `--features <name>`". `Backend::from_str` returns it for a known name left out, so `--backend gpeople` in a build without it is refused as a command line (`usage-invalid`, exit 2) naming the feature, where it read "Invalid backend".
- **src/config.rs**: `AccountConfig::backends`, every declared block in the resolution order, read off the typed or `IgnoredAny` fields alike.
- **src/shared/client.rs**: `resolve_backend` walks `backends()` and returns `Result<Option<Backend>>`, failing `backend-not-built` under `auto` when the first declared block is a left-out backend rather than taking the next one; `AddressbookClient::new` (so the unified commands, the REPL and MCP) and `account capability` propagate it.
- **src/account/check.rs**: under `auto`, each declared left-out backend is a failed row coded `backend-not-built`, sorted into the resolution order.
- **src/account/list.rs**: `backends` names every declared block, built or not, in the resolution order (it was carddav, jmap, msgraph, gpeople, vdir, pimdir, compiled-in ones only), and the new `notBuilt` those left out; the text marks them `(not built)`.
- **Decided**: `account capability` fails rather than print the capabilities at `none`, as a left-out backend serves nothing and a script reads one code; `account list` keeps succeeding and says which backends this build lacks. `proxy` in a build with no network backend stays `IgnoredAny`, no command reaching it on its own. A key inside a left-out block is not checked by that build.
- **Evidence**: tests/account.rs on `--no-default-features --features rustls-ring,vdir` (the CI test build): `a_unified_command_over_a_backend_left_out_is_backend_not_built`, `account_capability_over_a_backend_left_out_is_backend_not_built`, `account_check_reports_a_backend_left_out_as_failed`, `account_list_names_a_backend_left_out`, `a_backend_flag_naming_a_backend_left_out_names_its_feature`; on `rustls-ring,carddav`: `auto_never_skips_a_backend_left_out`. On the old code the first two printed an uncoded "No backend matching" failure, the capability one succeeded with every row at `none`, check bailed with no row, list printed no backend, the flag read "Invalid backend gpeople", and the last took CardDAV.

CHANGELOG, MIGRATION.md. Spec: backends ("Backend selection is local-first" modified, "A backend left out of the build is named" added), config ("An unknown key is refused"), commands ("account check fails when a backend fails", "An account lists what it serves", the code table).
