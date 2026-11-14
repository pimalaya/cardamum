---
cairn: tasks
change: the-list-frame-is-shared
---

# Tasks

- [x] Cargo.toml: pimalaya-cli patched by path.
- [x] `card list` on `Listing` and `ListingPage`, `addressbook list` on `ItemErrors`, `error::write_item_errors` removed; the byte-exact tests pass unchanged.
- [x] `account capability` on the shared output, `Support::level` typed `CapabilityLevel`, `resolve_backend` shared with `AddressbookClient::new`.
- [x] tests/capability.rs (the flat rows, the text, a `--backend` the account lacks) and tests/mcp.rs (`account_capability` rows), each failing on the old code.
- [x] CHANGELOG [Unreleased], MIGRATION.md.
- [x] Fold the delta into commands.md, write the log entry, set `landed`.
