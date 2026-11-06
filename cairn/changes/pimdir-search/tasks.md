---
cairn: tasks
change: pimdir-search
---

# Tasks

- [ ] Cargo.toml: `pimdir` enables `io-pimdir/search`; `[patch.crates-io]` ical-rs at 99b9273; ariadne
- [ ] src/error.rs: `index-missing`, `index-outdated`, `index-version`, `index-fts5-missing`, `query-refused`, `query-unanswerable`, with spec/commands.md's code table
- [ ] src/pimdir/index.rs: `open_query`, the scope (`kind:contact`, `account:`, `collection:`), hits read through the index's reader and projected as `card list` projects them, the coverage and its line, the query parse and its refusal (plain and JSON)
- [ ] src/pimdir/search.rs: `pimdir search <query>... [-k] [--sort name|relevance] [--page] [--page-size]`, table and JSON (`hits`, `more`, `coverage`)
- [ ] src/pimdir/count.rs: `pimdir count <query>... [-k]`, JSON (`count`, `coverage`)
- [ ] src/pimdir/cli.rs: the two commands beside `status`, the family's help updated
- [ ] src/json_schema.rs: `cardamum-pimdir-search`, `cardamum-pimdir-count`
- [ ] Tests on a scratch store whose cards a test owner drains and whose index the test builds (`PimdirIndex::open`, `refresh`): a word of a `NOTE` with its snippet; a phone number of a `TEL` by free text; `has:phone`, `category:`, `email:@domain`; a bodiless card matched on its summary and counted in `bodiless`; `-k` narrowing the hits and the coverage; the configured account alone in a store of two; name order; a queued card not listed until drained and refreshed, `behind` reported in between; every §8.4 refusal code in both renderings; a missing and an outdated index
- [ ] Each scope test fails with its scope term removed
- [ ] README pimdir section; CHANGELOG
- [ ] Fold delta.md into spec/commands.md, log, archive
