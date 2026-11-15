---
cairn: log
change: the-addressbook-is-a-positional
date: 2026-11-15
---

# The address book is a positional

`addressbook delete <ADDRESSBOOK>` and `addressbook update [ADDRESSBOOK]` take the address book as Himalaya's `mailbox delete <MAILBOX>` takes its mailbox, where both took `-k/--addressbook`. Task C4 of cli-alignment-2026-11-15 (finding F22), twin of calendula's K5. Capabilities **commands** and **backends** (a scenario) moved.

- **src/shared/addressbook/delete.rs, update.rs**: the positional (`value_name = "ADDRESSBOOK"`, required on delete) beside a hidden `-k/--addressbook` declared `conflicts_with` it. clap's validator waives a required argument when a conflicting one is present (`Validator::is_missing_required_ok`), so `-k work` alone parses, the usage and the MCP schema still show `<ADDRESSBOOK>` as required, and both together fail `ArgumentConflict`, `usage-invalid` under `--json`. The value name keeps the MCP property `addressbook`, unchanged.
- **`--color`**: `-C` kept on create and update; calendula has none, `-C` proposed for both twins.
- **Evidence**: the parse and usage unit tests of both commands (the positional fails to parse on the old grammar), tests/addressbook_not_found.rs on the positional with one `-k` case each, tests/mcp.rs `an_addressbook_is_deleted_by_its_property` and the schema assertions of `writes_are_listed_under_allow_write`.

CHANGELOG (Changed), MIGRATION.md ("Command names"). Spec: commands ("A collection command takes its address book as a positional" added, "A command with no address book is coded" modified), backends (the "An id reaching out of the home" scenario).
