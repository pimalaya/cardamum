---
cairn: change
change: carddav-query-filter
---

# Delta

## ADDED Requirements

### Requirement: CardDAV query filters
`carddav report query` SHALL map each of `--match`, `--not-match`, `--defined` and `--not-defined` onto its own RFC 6352 prop-filter, under a top-level test set by `--test` and defaulting to `allof`, and SHALL send `--collation` on every text-match and `--limit` as the query limit. Anything the flags cannot express goes through `report raw`.

### Requirement: A truncated query says so
A query the server truncated SHALL say so, as a `truncated` field in JSON and as a line in text output, rather than read as the whole result.
