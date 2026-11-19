---
cairn: tasks
change: card-search
---

# Tasks

- [x] Owner decision on vdir: `partial` through the local matcher (relayed by the coordinator session, 2026-11-18).
- [x] io-pimdir: the in-memory card matcher, tested on the SEARCH §11 contact vectors (`PimdirQuery::store_terms`, `PimdirCard::read`, `match_cards`, under `search`, no SQLite).
- [x] io-pimdir: released in 0.17.0; it builds under `search` without `client`, so the `vdir` feature takes io-pimdir `search` alone, no rusqlite (Cargo.toml: `client` moved onto the `pimdir` feature). ical-rs and mime-rs come with it on a vdir build; no `query` feature split for now.
- [x] src/capability.rs: `card.search` (pimdir `full`, vdir `partial`, the network backends `none`) and `card.search.relevance` (pimdir alone), the `card search` leaf.
- [x] src/shared/card/search.rs: the command; the query parsed by src/shared/query.rs `parse_query` (moved from src/pimdir/index.rs), store terms refused with `query-unsupported` before anything is read.
- [x] The matching path for vdir (src/shared/card/search.rs `matched_page`): every address book in scope listed, every card matched at once (`person:` reads them all), sorted by the folded full name, paged by number.
- [x] The matching path for carddav, jmap, msgraph and gpeople: `card search`, its MCP tool and schema built with any backend, io-pimdir `search` and ariadne pulled by each network feature (Cargo.toml), the same `matched_page` (test `a_graph_search_matches_every_folder_depth`, over a scripted Graph session, a card two folders deep)
- [ ] Declare `card.search` `partial` on carddav, jmap, msgraph and gpeople (src/capability.rs `NO_SEARCH` rows) once a search over the largest test address book is measured; `none` until then. Owner ruling 2026-11-19: flip to `partial` once the measurement is acceptable; the extra ical-rs and mime-rs in a network-only build (through io-pimdir `search`) are accepted, no `query` feature split for now
- [x] pimdir: delegates to `pimdir search`'s page walk (src/pimdir/search.rs `search_page`), its hits, cursor and coverage verdict mapped onto the unified output.
- [x] `query-unsupported` in src/error.rs and the code table; `query-refused` raised by `card search`.
- [x] MCP `card_search` tool (src/mcp.rs; tests/mcp.rs calls it).
- [x] Tests: one query set answered by the matcher and by the index over the same bodies, equal hits in equal order (src/pimdir/index.rs `the_matcher_answers_the_hits_of_the_index`); tests/search.rs over a vdir: folding, scope, paging, `query-refused`, `query-unsupported`, relevance refused, an unknown `-k`.
- [ ] Live read-only searches on the Fastmail, Google and Microsoft test accounts; measure a search over the largest test address book (with the network backends).
- [ ] Answer #25 once released.
- [x] Fold the delta into commands.md and mcp.md, write the log entry, CHANGELOG (vdir and pimdir, 2026-11-18). The change stays active for the network backends.
