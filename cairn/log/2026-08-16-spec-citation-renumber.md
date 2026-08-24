---
cairn: log
change: spec-citation-renumber
landed: 2026-08-16
---

# Follow the pimdir spec renumbering

pimdir reordered its sections and moved the per-kind `meta` conventions to an annex ([its log entry](https://github.com/pimalaya/pimdir/blob/master/cairn/log/2026-08-16-spec-restructure.md) carries the old-to-new table). The `text/vcard` convention itself did not change, so the only thing owed here was the citations: `src/pimdir/card.rs` and `src/pimdir/backend.rs` named `§13`, which is now Annex A.

Doc comments only. No behaviour moved, and the card summary this crate writes is unchanged.
