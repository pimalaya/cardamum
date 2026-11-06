---
cairn: change
id: pimdir-search
status: landed
created: 2026-11-06
---

# pimdir search through the store's index

## Why

A pimdir store carries a full-text search index (pimdir SEARCH, io-pimdir `PimdirIndex`) with its own query language, built and refreshed by the store's owner (neverest, since its log 2026-10-30-neverest-builds-the-search-index). Himalaya queries it under `himalaya pimdir search` (himalaya cairn/log/2026-10-30-pimdir-search.md). Cardamum cannot: `card list` pages one address book through the reader and filters nothing, so a card is found by name only, and never by its notes, phones, postal address or organization, by `has:phone`, `has:birthday` or `category:`, nor across address books.

The query language arrives as a pimdir-specific command first. The unified API, the shared `card list` taking one query on every backend, is later work (see Later).

## Decisions

1. Two commands, siblings of `pimdir status`: `cardamum pimdir search <query>...` and `cardamum pimdir count <query>...`. No `thread` (SEARCH §9 threads mail alone) and no `tag` (SEARCH §10 tags are placement flags, which no contacts source syncs).
2. Query client only. Cardamum opens the index with `PimdirIndex::open_query`, read-only, and never creates, rebuilds nor refreshes it. An answer is the store as of the owner's last refresh: a queued card write is not listed nor folded in, and the coverage says how far the index lags.
3. Scope. The query is read as its conjuncts plus `kind:contact`, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--addressbook` names one. The scope stays as top-level conjuncts because io-pimdir computes the coverage from the top-level terms alone (io-pimdir src/client/search.rs, `coverage`). Without `-k` the search spans every address book of the account: `addressbook.default` is not applied, since a search that silently narrows to one address book reads as "no such card".
4. Order. `--sort name|relevance`, `name` by default: a card has no date (SEARCH §8.1), so the default `date` order would sort by `sort_key` alone. `--page` and `--page-size` page the hits, as in himalaya.
5. A hit. Its card is read through the index's own reader (`index.reader().get_item`, no queue overlay, matching the answer) and projected as `card list` projects it, so the row has the `card list` columns, then the address book, `matched` (`body`, `summary` or none) and the snippet. Its id is the `seq`, which `card read` takes with `-k`. A card the store dropped since the refresh keeps the title the index answered, as himalaya's hits do.
6. Coverage. `complete`, `bodiless` (cards whose body is not local, matched on their summary alone), `unindexed`, `behind` and `filling`; `since` and `until` are left out, a card having no date. `complete` also requires `behind` at 0, as in himalaya. The plain output prints one line under the table when the answer is incomplete, and nothing when it is complete.
7. Codes, spelled as himalaya spells them: `index-missing`, `index-outdated`, `index-version`, `index-fts5-missing`, `query-refused`, `query-unanswerable`. The missing and outdated messages say the sync engine builds or rebuilds the index, never to delete anything.
8. Refusals. Plain output draws the SEARCH §8.4 span with ariadne, as himalaya does (one new dependency). Under `--json` the error is cardamum's error object: the code `query-refused`, with the §8.4 code, span and expectations in `error`, himalaya's deviation, until pimalaya-cli's error report carries structured details.
9. Feature. `pimdir` enables `io-pimdir/search`, with no feature of its own. Cardamum then compiles ical-rs and mime-rs through io-pimdir and needs the `[patch.crates-io]` ical-rs pin io-pimdir's search builds against (99b9273), which cardamum lacks today; the binary grows by both.
10. Code shape. Himalaya's src/pimdir/index.rs is copied into cardamum's src/pimdir/index.rs (scope, hit, coverage, refusal rendering) with src/pimdir/search.rs and count.rs beside it, the shared-client pattern of copying over sharing. The scope builder and the coverage completeness rule are candidates to move down into io-pimdir once three apps carry them (calendula's own pimdir-search is the third).

## Depends on

- io-pimdir 0c70144 (search-draft-08), the pin cardamum, neverest and himalaya already share, so cardamum reads the index version neverest writes. Nothing new is needed from io-pimdir.
- neverest syncing the account's address books with its `search` feature (on by default).

## Later

- The unified API: the shared `card list` takes a query on every backend, CardDAV, JMAP, Graph and People translating it, pimdir answering it natively, as himalaya plans for its v3.
- io-pimdir's search planner (pimdir cairn/changes/performance-audit-2026-11-05, phase 1) pages by a keyset cursor rather than an offset; cardamum moves with himalaya when it lands.
- search-draft-09, when the CLIs move to storage-draft-09 together.
- A contact's `ORG` is searched by free text alone, SEARCH §8 having no `org:` field; pimdir's to add if asked for.
