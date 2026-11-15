---
cairn: change
id: an-option-after-a-query-is-an-option
status: landed
created: 2026-11-15
---

# An option after a query is an option

## Why

Live testing of 2026-11-15 (reports/cardamum/part2-pimdir.md, F5, medium): the query of `pimdir search` and `pimdir count` took `allow_hyphen_values` and `trailing_var_arg`, so every word after the first query word was a query word. `pimdir search fictif --page-size 3` searched `fictif --page-size 3` and answered nothing, `pimdir search Voisin --json` printed text, and `-k <book>` after the query searched it, each exit 0. Task C26 of cli-alignment-2026-11-15, family-wide: calendula fixed it in de9ea97 (the same id), Himalaya in H27.

## What

- The query is a plain repeated positional: an option is read wherever it stands, `--json` included.
- A query word starting with `-`, a negated term, goes after `--`, the options before it, or reads as `not`; before `--` it is refused as an unknown option (exit 2), or read as the option it spells. The help says so.
- `pimdir search` and `pimdir count` are new in v0.4, so nothing breaks for a v0.3 user; MIGRATION.md's "New" section shows the `--` form.
