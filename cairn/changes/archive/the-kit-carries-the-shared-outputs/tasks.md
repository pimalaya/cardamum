---
cairn: tasks
change: the-kit-carries-the-shared-outputs
---

# Tasks

- [x] `cargo update -p pimalaya-cli -p pimalaya-config`, the lock diff kept to the two crates.
- [x] src/account/list.rs and src/account/check.rs on the kit's types; src/json_schema.rs registers them.
- [x] src/repl.rs: the nested-session arm removed; src/cli.rs: the version on `Command`.
- [x] src/mcp.rs: `tool.grammar.usage_error`; tests/mcp.rs asserts the grammar's wording and a missing argument.
- [x] src/shared/card/build.rs: the kit's `SavedOutput`.
- [x] src/config.rs: `removed_key!`; the removed-table test asserts the whole sentence.
- [x] tests/repl.rs: a nested `repl` and `mcp` coded `usage-invalid`, a version line.
- [x] Fold the delta, log, CHANGELOG.
