---
cairn: log
change: the-list-frame-is-shared
date: 2026-11-14
---

# The list frame is shared

Cardamum prints its lists, their item errors and `account capability` through pimalaya-cli (79c0d9e, patched by path in Cargo.toml until its release), following Himalaya (984549dd). Capability **commands** moved.

- **Cargo.toml, Cargo.lock**: `pimalaya-cli = { path = "../cli" }` in `[patch.crates-io]`, where it pinned 0be1b1f; the io-webdav and io-pim-discovery path patches stay.
- **src/shared/card/list.rs**: `card list` builds a `pimalaya_cli::listing::Listing` (`Listing::new` with the backend, `head("Addressbook")`, `page` over `ListingPage::Numbered` or `All`, the errors in `Listing::errors`); its own `header`, `paging` and `write_fields` are gone. Its text is byte for byte what it was: the label column already sat two spaces past `Addressbook:`, which is what the shared frame does for a label longer than `Next page:`.
- **src/shared/addressbook/list.rs, src/error.rs**: `addressbook list` keeps its column table and prints its errors through `pimalaya_cli::error::ItemErrors::write_after`; `error::write_item_errors`, the copy, is gone. `error::with_code` stays (an owner decision is pending on it).
- **src/account/capability.rs, src/capability.rs, src/shared/client.rs, src/json_schema.rs**: `account capability` prints `pimalaya_cli::capability::AccountCapabilityOutput`, `{account, capabilities: [{name, backend, level, detail}]}`, where it printed `{account, backend, capabilities: [{name, level, detail}]}`. `Support::level` is the typed `CapabilityLevel`, the help reading the same. `shared::client::resolve_backend` names the backend `AddressbookClient::new` takes, one priority order for both; when it finds none `--backend` allows, every row reads `backend` `null`, level `none`, where the command failed with exit 1. The text is a list headed by `Account:`, one line per capability, where it was a table under a `CAPABILITY  BACKEND  LEVEL  DETAIL` header row. The `cardamum-account-capability` key is unchanged, its schema the shared one (the levels an enum).
- **Evidence**: tests/capability.rs `the_account_lists_what_its_backend_serves` (the flat rows), `the_capabilities_print_one_line_each_under_the_account` and `a_backend_the_account_lacks_resolves_none`, tests/mcp.rs `account_capability_answers_one_row_per_capability`, each failing on the old code; the byte-exact `card list` and `addressbook list` tests pass unchanged.

`account capability` is unreleased (0.3.0 has none), so the CHANGELOG amends its [Unreleased] Added entry rather than adding a breaking one, as Himalaya did for its unreleased frame. MIGRATION.md names what the command prints. Left open: the path patch of pimalaya-cli becomes a version once it is released.
