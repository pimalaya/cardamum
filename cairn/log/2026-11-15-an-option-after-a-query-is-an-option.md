---
cairn: log
change: an-option-after-a-query-is-an-option
date: 2026-11-15
---

# An option after a query is an option

`pimdir search` and `pimdir count` read an option wherever it stands, where every word after the first query word joined the query (`pimdir search fictif --page-size 3` answered nothing, a trailing `--json` printed text). A `-` term goes after `--`. Task C26 of cli-alignment-2026-11-15, live finding F5 (reports/cardamum/part2-pimdir.md); calendula de9ea97 and Himalaya H27 did the same. Capability **commands** moved.

- **src/pimdir/search.rs, src/pimdir/count.rs**: the query positional drops `allow_hyphen_values` and `trailing_var_arg`; its help names `--`.
- **Evidence**: src/cli.rs `an_option_after_a_query_is_an_option` parses `pimdir search alice --page-size 3 [--json]`, `pimdir count alice -k book`, `-s 5 -- alice -zoe`, and refuses `alice -zoe`; on the code before the query read `alice --page-size 3`.

CHANGELOG (the unreleased `pimdir search` entry), MIGRATION.md ("New"). Spec: commands ("pimdir searches cards through the store's index").
