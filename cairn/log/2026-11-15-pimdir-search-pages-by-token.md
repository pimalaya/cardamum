---
cairn: log
change: pimdir-search-pages-by-token
date: 2026-11-15
---

# pimdir search pages by token

`pimdir search` prints `{hits, nextPage, coverage, errors}`, takes `--page-token`, and lists a card the store cannot read as an `errors` entry, exit 3, as Himalaya's (c7016650, the same id) and `card list` do; its text is pimalaya-cli's `Listing`. Tasks C9 (F12, D9) and C20 of cli-alignment-2026-11-15. Capabilities **commands** and **backends** moved.

- **src/pimdir/search.rs**: `PageTokenArg` (a page number from 1, winning over `--page`, else ``Invalid page token <token>``), `nextPage` for `more`, `errors`, `partial_when`; the text `Account:`, `Addressbook:` (with `-k`), `Query:`, `Page:`, `Next page:`, `Incomplete:`, one line per hit (id, address book, what matched when a hit says, email, phone, name), the `Error:` lines.
- **src/pimdir/index.rs**: `search_cards` makes a hit `card_hit` cannot read an entry, coded as its failure is or `card-unreadable`, its error not naming the card; `card_hit` fails on an item the store cannot read, a body that cannot be read or is gone from the blob tree (`listed_card`), and one that reads as no vCard (`unreadable`), as `card list` does. `PimdirCardCoverage::summary` and `note` put the coverage above the hits; `pimdir count` keeps its line under the count.
- **src/pimdir/backend.rs**: `card_from_item`, which previewed a missing body with a warning, goes; `listed_card` is crate-visible.
- **Evidence**: `a_hit_the_store_cannot_read_is_an_entry` removes one blob of an indexed store: the other four hits list and the entry is `card-unreadable` `Body blob missing from the store`; on the code before the card listed as its preview. `a_page_names_the_next_and_its_errors` checks the keys and the text. Found on the way: io-pimdir's own search fails whole on a blob it cannot open (permission denied), before cardamum reads a hit, so that case stays a failure of the command.

CHANGELOG (the unreleased `pimdir search` entry, the pimdir missing-body line), MIGRATION.md ("Text output", "New"). Spec: commands ("pimdir searches cards through the store's index", "A pimdir card search says what it did not cover", "Exit statuses"), backends ("Each backend names the cards it could not read").
