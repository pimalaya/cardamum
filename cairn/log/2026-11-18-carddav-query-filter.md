---
cairn: log
date: 2026-11-18
change: carddav-query-filter
---

# CardDAV query filters, folded

The change landed in code before v0.3's successor (`--match`, `--not-match`, `--defined`, `--not-defined`, `--test`, `--collation`, `--limit` in src/carddav/report/query.rs; `truncated` in src/carddav/report/entries.rs), its tasks ticked, but its delta was never folded. Checked against the code on 2026-11-18 and folded: commands ("CardDAV query filters" added; its "A truncated query says so" was already stated by "A CardDAV report names the cards it could not read"). Archived.
