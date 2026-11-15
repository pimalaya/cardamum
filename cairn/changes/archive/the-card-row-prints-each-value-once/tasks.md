---
cairn: tasks
change: the-card-row-prints-each-value-once
---

# Tasks

- [x] src/shared/card/list.rs: drop `fn_value`, `email`, `tel` and the raw scanner; `CardRow::cells` for the text columns; src/pimdir/search.rs prints them.
- [x] Tests: `a_row_prints_each_value_once` and `the_cells_read_the_projected_fields` (src/shared/card/list.rs), `a_read_prints_the_row_of_its_listing` (src/shared/card/read.rs) asserting no `fnValue`; each fails on the code before (the keys printed, the text kept `tel:+33-6-0000` and `Doe\, Jane Marie`). src/pimdir/index.rs tests read `fields.full_name`; tests/common.rs `assert_cards` refuses the removed keys.
- [x] CHANGELOG (Breaking line under Removed, the `card read` and Fixed lines reworded), MIGRATION.md (JSON output).
- [x] Fold the delta into commands.md, write the log entry, set `landed`, archive.
