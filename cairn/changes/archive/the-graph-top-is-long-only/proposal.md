---
cairn: change
id: the-graph-top-is-long-only
status: landed
created: 2026-11-16
---

# The Graph top is long-only

## Why

`msgraph contacts list`, `msgraph contact-folders list` and `msgraph contact-folders child-folders` showed `-s, --top <N>`, where Himalaya's Graph listings take `--top` long-only. The owner's ruling of 2026-11-16 (L17 of the CLI alignment round) aligns the twins on Himalaya.

## What

- The three listings show `--top <N>` alone in their help, the manual and the completions.
- `-s` stays accepted as a hidden short alias of `--top` (clap `short_alias`), so no script breaks: not a breaking change, no MIGRATION.md entry.
- The other `-s` letters of the CLI (`--page-size` on `card list`, `pimdir search` and the `gpeople` listings) are no `--top` and stay as they are.
