---
cairn: log
change: the-graph-top-is-long-only
date: 2026-11-16
---

# The Graph top is long-only

Line L17 of the CLI alignment round, the owner's ruling of 2026-11-16. Capability **commands** moved.

- **src/msgraph/contacts/list.rs, src/msgraph/contact_folders/list.rs, src/msgraph/contact_folders/child_folders.rs**: `--top` shows long-only, as Himalaya's Graph listings; `-s` stays a hidden `short_alias`, so a script giving it keeps working. Not breaking, no MIGRATION.md entry.
- **Evidence**: `the_graph_top_is_long_only` (src/cli.rs) asserts each listing's help names `--top <N>` and no `-s,`, and that `-s 5` parses as `--top 5`; it fails with `short = 's'` put back on `msgraph contacts list`.

CHANGELOG (Changed). Spec: commands ("A former command name stays hidden", modified, one scenario added).
