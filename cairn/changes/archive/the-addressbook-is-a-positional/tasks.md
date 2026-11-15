---
cairn: tasks
change: the-addressbook-is-a-positional
---

# Tasks

- [x] src/shared/addressbook/delete.rs and update.rs: the positional and the hidden `-k`; unit tests on the parse (positional, `-k`, `--addressbook`, both refused, none) and the usage, failing on the old grammar.
- [x] tests/addressbook_not_found.rs on the positional, `-k` kept in one case each; tests/mcp.rs `an_addressbook_is_deleted_by_its_property` and the schema of `addressbook_delete` and `addressbook_update`.
- [x] Fold the delta, log, CHANGELOG, MIGRATION.md.
