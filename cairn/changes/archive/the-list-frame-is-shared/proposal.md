---
cairn: change
id: the-list-frame-is-shared
status: landed
created: 2026-11-14
---

# The list frame is shared

> Cross-repo change, same id in pimalaya-cli (79c0d9e) and Himalaya (984549dd). Order: pimalaya-cli, Himalaya, then Cardamum (this side).

## Why

The owner decided on 2026-11-14 that the list frame, the item errors and the `account capability` output belong to pimalaya-cli, so every binary prints a list, its item errors and an account's capabilities the same way. Cardamum carried copies: `error::write_item_errors` (a copy of `ItemErrors`, waiting for the pinned commit to carry it) and `card list`'s own header, paging and label column. Its `account capability` printed `{account, backend, capabilities: [{name, level, detail}]}`, the resolved backend once beside rows that did not name it, and failed when `--backend` allowed no configured backend.

## What

- Cargo.toml patches pimalaya-cli by path (`../cli`) until its release, where it pinned 0be1b1f.
- `card list` prints through `pimalaya_cli::listing::{Listing, ListingPage}`; `addressbook list` keeps its column table and prints its errors through `pimalaya_cli::error::ItemErrors`; `error::write_item_errors` is gone. The text of both is byte for byte what it was.
- `account capability` prints `pimalaya_cli::capability::AccountCapabilityOutput`: `{account, capabilities: [{name, backend, level, detail}]}`, `backend` the backend each capability resolves to, `null` with level `none` when the account configures no backend `--backend` allows, where the command failed. `capability::Support::level` is the typed `CapabilityLevel`. The text is a list headed by `Account:`, one line per capability (name, backend, level, detail) with no column header. `shared::client::resolve_backend` names the backend `AddressbookClient::new` takes, so both read one priority order.
- Spec: commands (the card page, the errors, the capability output).
