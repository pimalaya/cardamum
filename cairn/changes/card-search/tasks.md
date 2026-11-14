---
cairn: tasks
change: card-search
---

# Tasks

- [ ] Owner decision on vdir (`partial` through the local matcher, or `none`).
- [ ] io-pimdir: the in-memory card matcher, tested on the SEARCH §11 contact vectors (its own change in io-pimdir's cairn).
- [ ] io-pimdir: release with the matcher; keep it building under `search` without `client` (true on master 2026-11-14), or split a `query` feature if ical-rs and mime-rs weigh on non-pimdir builds.
- [ ] src/capability.rs: `card.search` and `card.search.relevance` rows for every backend, the `card search` leaf.
- [ ] src/shared/card/search.rs: the command, parsing with `PimdirQuery::parse`, refusing store terms with `query-unsupported` before connecting.
- [ ] The listing path for carddav, jmap, msgraph and gpeople (and vdir if decided): list every address book in scope, match each card, sort by name, page.
- [ ] pimdir: delegate to the code behind `pimdir search`, mapping its hits and coverage verdict onto the unified output.
- [ ] `query-unsupported` in src/error.rs and the code table; `query-refused` raised by `card search`.
- [ ] MCP `card_search` tool.
- [ ] Tests: one query set answered over a scratch pimdir store and a scratch vdir (or a mocked listing), equal hit sets; every store term refused on a non-pimdir backend; §8.4 refusals.
- [ ] Live read-only searches on the Fastmail, Google and Microsoft test accounts; measure a search over the largest test address book.
- [ ] Answer #25 once released.
- [ ] Fold the delta into commands.md, backends.md and mcp.md, write the log entry, CHANGELOG, mark `landed`.
