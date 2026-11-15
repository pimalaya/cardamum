---
cairn: change
id: the-addressbook-is-a-positional
status: landed
created: 2026-11-15
---

# The address book is a positional

## Why

`addressbook delete` took its address book as a required `-k/--addressbook` and `addressbook update` as an optional one, where Himalaya's collection commands take theirs as a positional (`mailbox delete <MAILBOX>`). Finding F22, task C4 of cli-alignment-2026-11-15, twin of calendula's K5.

## What

- `addressbook delete <ADDRESSBOOK>`, required; `addressbook update [ADDRESSBOOK]`, falling back to `addressbook.default` as before.
- `-k/--addressbook` stays accepted as a hidden alias: a second, hidden argument conflicting with the positional, which clap's validator waives when that conflicting option is present (`is_missing_required_ok`), so the usage still reads `<ADDRESSBOOK>` and the MCP schema still marks it required. Both together are refused `usage-invalid`.
- The value name `ADDRESSBOOK` keeps the MCP property `addressbook` the option gave it, so no tool input changes.
- `--color` keeps `-C`, the letter proposed to calendula for the twins.
- Breaking for the help text only: MIGRATION.md says it.
