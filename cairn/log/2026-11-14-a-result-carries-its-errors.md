---
cairn: log
change: a-result-carries-its-errors
date: 2026-11-14
---

# A result carries its errors

cardamum follows Himalaya's change of the same name (2026-11-14): `addressbook list` and `card list` print an `errors` list beside their items, one pimalaya-cli item error `{id, code, error, sources}` per card they could not read, and exit 3 when it is not empty; a server's refusal and a missing card or address book carry a code of the family vocabulary. Capabilities **commands**, **backends** and **mcp** moved.

## Phase 1, the libraries (dcbae05)

- **Cargo.toml, Cargo.lock**: pimalaya-stream 0.4, io-http 0.8, io-jmap 0.9, io-msgraph 0.9, dev io-oauth 0.7; `[patch.crates-io]` io-gpeople e4a9cac (stream 0.4 over the year-less `BDAY` fix), io-webdav 185ab63, pimalaya-cli 0be1b1f (`ItemError` and the registry spare) and io-pim-discovery from `../io-pim-discovery` (its stream-0-4 change, 1c3d195, local); `cargo update` took pimalaya-config 0.4.1.
- **src/config.rs**: `TlsConfig::into_tls` fills `server` from `tls.cert`, `client: None`; the key keeps its name.
- **Evidence**: `cargo tree -d --all-features -e normal` lists no family crate twice; tests/google.rs and tests/microsoft.rs build against io-oauth 0.7.

## Phase 2, the codes (431ad9d)

- **src/error.rs**: `permission-denied`, `quota-exceeded`, `rate-limited`, `card-not-found`, `addressbook-not-found` and `card-unreadable`, declared in every build (the pimdir codes ungated too, the schema the same whatever the features); `error::code` walks the chain over one reader per backend, and `error::with_code` codes the failure object once, in `Command::execute`. `addressbook-unreadable` left out: no backend reads an address book it cannot list whole.
- **Readers**: CardDAV (src/carddav/client.rs, a 507 or `quota-not-exceeded`, 429, a 403 naming `need-privileges` or no precondition), Graph (src/msgraph/client.rs), People (src/gpeople/client.rs, a copy of neverest's `google_code`), vdir (a missing item or collection); JMAP SetErrors coded where they are built (src/jmap/error.rs); not-found coded at the card requests of each backend (`card_error`) and on pimdir. A refused `valid-address-data` keeps io-webdav's typed error as the cause of the hint. `account check` rows read their code through `error::code`.
- **Evidence**: `a_carddav_refusal_carries_its_code`, `a_refused_write_prints_its_code_in_its_failure_object`, `a_graph_refusal_carries_its_code`, `a_people_refusal_carries_its_code`, the `a_missing_*_is_a_card_not_found` tests and `a_set_error_is_coded_by_its_type` over scripted streams (src/script.rs, shared by the HTTP backends), tests/not_found.rs running the binary over a temp vdir and a temp pimdir store.

## Phase 3, the listings (d0bd73b)

- **Outputs**: `errors` required in both schemas, the item error registered under `cardamum-item-error`; a non-empty list returns `PartialOutcome::default()` once printed (exit 3). The text prints `card list`'s paging in its header and the `Error:` lines last, after one blank line, through `error::write_item_errors`, a copy of pimalaya-cli 79c0d9e's `ItemErrors` (unpushed) to be swapped for it with no output change.
- **Backends**: a body that is no vCard or not UTF-8 is an entry on every backend (`shared::client::unreadable`, over the cards of the page); vdir reads io-vdir 0.3's `list_readable_items`; a JMAP card that does not project is an entry, the others listed; a pimdir body that cannot be read is an entry, its card listed with its preview, a body not hydrated yet none. Whole-read backends print their entries on the first page only, pimdir on the page holding the card.
- **MCP and REPL**: the instructions name `errors`; a session keeps its client after a partial outcome.
- **Evidence**: tests/listing.rs (an unreadable vdir file, first page only, an unparseable body, a non-UTF-8 body, the byte-exact text), tests/exit.rs's exit 3, tests/mcp.rs `a_listing_with_errors_answers_its_result_beside_is_error`, `a_card_that_does_not_project_is_an_entry_the_others_listed`, `an_unreadable_body_is_an_entry_its_card_listed_with_its_preview`, `a_partial_outcome_keeps_the_client_a_failure_drops_it`.

## Phase 4, CardDAV rows

- **Cargo.toml, Cargo.lock**: io-webdav from `../io-webdav` at its local 4ecb2ab (a-listing-returns-what-it-could-not-read), where it was pinned at 185ab63.
- **src/carddav/backend.rs**: each `WebdavUnreadableEntry` of `CarddavCardListOk::unreadable` is an entry naming the card, coded by `carddav::client::row_code` (the Phase 2 reader's status rule over no body) from its row status or a refused property's, so a 403 is `permission-denied`, else `card-unreadable`; its refused properties (`address-data (403)`) in `sources`. `truncated` (a 507 row) adds an entry with a `null` id and no code. The entries print on the first page only, as CardDAV reads the whole address book. `carddav report multiget` reads `.cards` of io-webdav's `CarddavCardMultigetOk`, its output unchanged.
- **Evidence**: `a_carddav_listing_names_the_rows_it_could_not_read` over a scripted multistatus (a card, a 403 row, a refused `address-data`, a data-less row, a 507 row) fails with the rows dropped as before; `a_carddav_listing_prints_its_entries_on_the_first_page_only`.

## Landing

CHANGELOG (Added the codes and `errors`, **Breaking:** the exit 3 and the vdir and JMAP listings, Changed the not-found codes; the Unreleased text-output entry now says `card list` prints its paging above the cards, which 0.3 never printed), MIGRATION.md (`errors`, exit 3, the codes, the `Error:` lines and the paging above the cards), the README exit statuses, the output doc comments. The delta is folded into spec/commands.md, backends.md and mcp.md, and rewritten to the landed text.

Left open: the io-webdav and io-pim-discovery path patches switch to git rev pins once 4ecb2ab and 1c3d195 are pushed, and pimalaya-cli moves to the commit carrying `ItemErrors`, dropping `error::write_item_errors`. `carddav report multiget` and `report query` still print no row the server answered without data. The live suites (google, microsoft, fastmail) are for the owner to run on the result.
