---
cairn: tasks
change: the-shared-kit
---

# Tasks

- [x] Cargo.toml: pimalaya-cli 0.4 pinned to 0fbe5dd, pimalaya-config 0.4, crossterm out, clap without `env` and `wrap_help`
- [x] src/config.rs: `sample_url`, `accounts`, no table or colour types, `carddav.alpn`
- [x] src/cli.rs and src/main.rs: `ConfigFlags`, `resolve_account` over `load` and `take_account`, `meet_bare_invocation`, the welcome
- [x] src/shared/client.rs: one `SecretResolver` per account, the account name and backend, `card list` windows read one card past the page
- [x] `card list` header and footer, `nextPage`, `--page-token`; `addressbook list`, `account list` and `account check` plain
- [x] Protocol-specific renderers to `Columns`, `Records` and `Fields`, ariadne without colour
- [x] tests/first_run.rs, the text of `card list` pinned
- [x] CHANGELOG, README configuration paths, the spec folded
- [x] Clippy `-D warnings` (all features and each backend alone), tests, `json-schema` identical but `card list`
