---
cairn: log
change: the-vdir-item-is-added
date: 2026-11-16
---

# The vdir item is added

Line L18 of the CLI alignment round, the owner's ruling D32 of 2026-11-16. Capability **commands** moved.

- **src/vdir/item/add.rs, src/vdir/item/cli.rs, src/vdir/item/mod.rs, src/vdir/item/input.rs**: `vdir item create` is `vdir item add` (`VdirItemAddCommand`, `VdirItemAddOutput`), `new` still its visible alias and `create` a hidden one; the output is unchanged.
- **src/json_schema.rs**: the schema key is `cardamum-vdir-item-add`, where v0.3.0 shipped `cardamum-vdir-item-create`: breaking, with its MIGRATION.md entry. No MCP tool reaches `vdir`, so no tool name moves.
- **Evidence**: `the_vdir_item_is_added` (src/cli.rs) parses `add`, `create` and `new` to the same `VdirItemCommand::Add` and asserts the help lists `add` with `[alias: new]` and no `create`; it fails with the command named `create` again.

CHANGELOG (Changed, breaking), MIGRATION.md (Command names). Spec: commands ("Specific commands mirror their protocol" and "A former command name stays hidden", modified, one scenario added).
