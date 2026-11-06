---
cairn: log
change: pimdir-search
date: 2026-11-06
---

# pimdir search through the store's index

`cardamum pimdir search <query>...` and `cardamum pimdir count <query>...` query the store's search index (pimdir SEARCH, search-draft-08) at the io-pimdir pin 0c70144, opened with `PimdirIndex::open_query`: read-only, never built nor refreshed. The `pimdir` feature now enables `io-pimdir/search` and the optional ariadne 0.6, and Cargo.toml patches ical-rs to 99b9273 as himalaya and calendula do. Cargo.lock gains only what those pull: ariadne and yansi, ical-rs, mime-rs, base64-codec, quoted-printable-codec and simd-cpu at their git revisions, unicode-normalization and tinyvec.

Himalaya's src/pimdir/index.rs is copied into src/pimdir/index.rs, on `PimdirBackend` since the projection lives there: the scope (the query's conjuncts plus `kind:contact`, `account:` when the store groups the account, `collection:` with `-k`, all top-level), a hit read through the index's reader (`get_item`, no queue overlay) and projected by `card_from_item` into a `card list` row with `matched` and `snippet`, the coverage (`complete`, `bodiless`, `unindexed`, `behind`, `filling`, `complete` also requiring `behind` at 0) and its line, the query parse with its refusal drawn by ariadne in plain output and as one line (`Query refused, <code> at <start>..<end>: ...; expected ...`) under `--json`. src/pimdir/search.rs takes `-k`, `--sort name|relevance` (name by default), `--page` and `--page-size` (falling back to `card.list.page-size`), and prints the `card list` columns plus the address book, what matched and the snippet; src/pimdir/count.rs prints `{count, coverage}`. Both are registered as `cardamum-pimdir-search` and `cardamum-pimdir-count`. src/error.rs gains `index-missing`, `index-outdated`, `index-version`, `index-fts5-missing`, `query-refused` and `query-unanswerable`, read off `PimdirError::code`. The `pimdir` dispatch now hands the merged account to the family for the table preset, colors and page size.

Spec: commands (Specific commands mirror their protocol; pimdir searches cards through the store's index; pimdir counts cards from the index; A pimdir card search says what it did not cover; A pimdir index or query failure is coded; the code table).

Deviations from the proposal:

- A `-k` naming no address book of the account fails naming those it holds (`known_collection`, as `card list` does), rather than answering nothing, which is the "no such card" reading the proposal avoided by not applying `addressbook.default`. Folded into the spec.
- A hit names the address book `-k` asked when it lies in several, else its first.
- Decision 5's card "dropped since the refresh" never reaches the projection in practice: the index answers from the store's live rows, so a card the owner removed after the refresh is simply not listed while `behind` says the index lags (test `a_card_removed_since_the_refresh_is_not_listed`). The fallback keeping the index's title stays for the race between the answer and the read.
- The coverage line, when `complete` is false and none of its fields holds (a temporal bound only a `date:` term could raise), says the store is not known to hold every card in scope rather than printing nothing.
- The tests reach `index-outdated` by copying another store's index.db and `index-version` by rewriting the draft in index.db's bytes, so cardamum takes no rusqlite dev-dependency. `query-unanswerable` has no test: io-pimdir's `PimdirSqlError` cannot be built outside it.

Verification: 17 tests in src/pimdir/index.rs on scratch stores whose cards a test owner drains and whose index the test builds (`PimdirIndex::open`, `refresh`), plus the new codes in src/error.rs's mapping test. Each scope test was checked to fail with its scope term removed, then the code restored: without `kind:contact`, `a_search_reads_cards_alone`, `the_addressbook_narrows_the_hits_and_the_coverage` and `a_bodiless_card_matches_on_its_summary` fail; without `account:`, `a_search_reads_the_configured_account_alone`; without `collection:`, `the_addressbook_narrows_the_hits_and_the_coverage` and `a_first_sync_still_listing_is_reported`. `cargo test --all-features` 104 passed, `cargo clippy --all-features --all-targets` clean, and each of `carddav`, `jmap`, `msgraph`, `gpeople`, `vdir`, `pimdir` built alone with `rustls-ring`, and the default build, without a warning.
