---
cairn: tasks
change: the-contract-audit-rulings
---

# Tasks

- [x] Cargo.toml: `[patch.crates-io] pimalaya-cli = { path = "../cli" }`, to become a release pin before commit.
- [x] src/config.rs: `TlsConfig::client`, `TlsClientConfig`, `TlsConfig::inherit`, the account `tls` and `inherit_tls`; config.sample.toml documents them.
- [x] src/account/check.rs: `AccountCheckOutput { accounts }`, `tls` on network rows, `null` over `http`.
- [x] src/main.rs, src/cli.rs: bare invocation through `pimalaya_cli::clap::meet_bare_invocation`; `protocol_exit` on the protocol namespaces; `accounts` alias.
- [x] src/capability.rs: names after the unified commands.
- [x] src/pimdir/backend.rs, queue/list.rs, queue/show.rs, status.rs: `pending_rows`, `listed_row`, `{actions, parked, errors}`, the show row, `collection`.
- [x] src/pimdir/search.rs: `--page-token` text cursor.
- [x] src/shared/card/list.rs, src/jmap/contact_card/query.rs, src/error.rs: `page-token-invalid`.
- [x] src/repl.rs, src/shared/card/composer.rs, src/shared/card/vcard.rs: `stdin-taken`, `card-invalid`.
- [x] src/shared/client.rs, src/shared/addressbook/create.rs: `check_new_addressbook`.
- [x] Tests, CHANGELOG, spec fold, log.
