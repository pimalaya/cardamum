---
cairn: tasks
change: pimdir-search-pages-by-token
---

# Tasks

- [x] src/pimdir/search.rs: `PageTokenArg`, `{hits, nextPage, coverage, errors}`, the `Listing` text, exit 3 on entries.
- [x] src/pimdir/index.rs: `search_cards` collects entries; `card_hit` fails on a body that cannot be read, is missing or does not parse; `PimdirCardCoverage::summary` and `note`.
- [x] src/pimdir/backend.rs: `listed_card` reachable, `card_from_item` removed.
- [x] Tests: `a_hit_the_store_cannot_read_is_an_entry` (failing on the code before, the missing body previewed), `a_hit_prints_on_one_line`, `a_page_names_the_next_and_its_errors`.
- [x] CHANGELOG, MIGRATION; fold the delta; log.
