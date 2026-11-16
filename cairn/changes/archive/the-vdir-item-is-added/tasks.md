---
cairn: tasks
change: the-vdir-item-is-added
---

# Tasks

- [x] src/vdir/item/create.rs moved to src/vdir/item/add.rs, `VdirItemAddCommand` and `VdirItemAddOutput`; src/vdir/item/cli.rs `Add` with `visible_alias = "new"` and the hidden `alias = "create"`; src/vdir/item/input.rs doc comments.
- [x] src/json_schema.rs: key `cardamum-vdir-item-add`.
- [x] Test `the_vdir_item_is_added` (src/cli.rs), seen failing with the command named `create` again (`name = "create"`, `add` a hidden alias).
- [x] Spec (commands), CHANGELOG (Changed, breaking), MIGRATION.md (Command names).
