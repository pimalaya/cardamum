---
cairn: log
change: the-card-row-prints-each-value-once
date: 2026-11-15
---

# The card row prints each value once

Schema review A15 of the CLI alignment round. Capability **commands** moved.

- **src/shared/card/list.rs**: `CardRow` drops `fnValue`, `email` and `tel`, which repeated `fullName`, `emails` and `phones` as the raw lines wrote them, and the raw line scanner goes with them. `CardRow::cells` gives the text columns from the projection: the first of `emails`, the first of `phones`, `fullName`, an escape decoded and a `tel:` URI as its number.
- **src/shared/card/read.rs, src/pimdir/search.rs**: `card read` (the row plus its vCard) and `pimdir search` (the row plus `matched` and `snippet`) lose the same keys; the search text prints the same cells.
- **Evidence**: `a_row_prints_each_value_once`, `the_cells_read_the_projected_fields` (src/shared/card/list.rs), `a_read_prints_the_row_of_its_listing` (src/shared/card/read.rs), each failing on the code before.

Breaking against v0.3's `card list --json`: CHANGELOG (Removed), MIGRATION.md (JSON output). Spec: commands ("A card projects its common fields", "A card read prints its listing row and its vCard", "A card page names its context and its next page" modified).
