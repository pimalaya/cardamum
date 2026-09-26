---
cairn: change
id: carddav-query-filter
status: active
created: 2026-09-26
---

# CardDAV query filters

> Cross-repo change, same id in io-webdav and cardamum (here). Order: **io-webdav** (0.4.0) → **cardamum**.

## Why

#25 asks for a card search. The shared `card` API stays least-common-denominator until the 2027 query work, where a single grammar shared with Himalaya and owned by the pimdir search spec reaches every backend. Until then search is per protocol: JMAP (`contact-card query --text`), Google (`connection search`), Microsoft (`contacts list --filter`) already have one, vdir has no protocol to mirror, and pimdir waits for 2027. CardDAV is the gap: `report query` always sends the match-all filter.

## What

`carddav report query` gains flags mapping onto RFC 6352 §10.5, one prop-filter per flag:

- `--match <PROP> <TYPE> <VALUE>` and `--not-match <PROP> <TYPE> <VALUE>`, repeatable: a text-match, negated for the latter. `TYPE` is `equals`, `contains`, `starts-with` or `ends-with`.
- `--defined <PROP>` and `--not-defined <PROP>`, repeatable.
- `--test allof|anyof`, default `allof`: every flag narrows, as in Himalaya and khard. The test is always sent, so the RFC's `anyof` schema default never applies.
- `--collation <NAME>`, applied to every text-match; omitted, the server's `i;unicode-casemap` applies.
- `--limit <N>`.

A truncated result says so: a `truncated` field in JSON, a line in text. The table gains an FN column read from the vCard, shared with `report multiget`, since ids and ETags alone do not tell a search result apart.

## Scope / non-goals

- **The flags cover the common case.** Param-filters, several text-matches in one prop-filter and a prop-level test stay with `report raw`.
- **No shared search.** `card list` passes the default match-all filter.
- **No client-side matching.** The server decides, and the command prints what it answered.
