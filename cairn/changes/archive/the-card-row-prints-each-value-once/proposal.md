---
cairn: change
id: the-card-row-prints-each-value-once
status: landed
created: 2026-11-15
---

# The card row prints each value once

> Schema review A15 of the family's CLI alignment round, X10 (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15, reports/schema-review-status-2026-11-15.md).

## Why

`CardRow` (src/shared/card/list.rs) printed `fnValue`, `email` and `tel`, the first `FN`, `EMAIL` and `TEL` read off the raw lines, beside `fullName`, `emails` and `phones`, the same values read through vcard-rs's decoded model. Two spellings of one value, differing on an escape (`Doe\, Jane`) or a `tel:` URI, and a second line scanner to keep right. `card read` and `pimdir search` reuse the row, so the repetition reached three outputs.

## What

- src/shared/card/list.rs: `CardRow` loses `fn_value`, `email` and `tel`, and the raw scanner (`vcard_preview`, `unfold`, `property`) goes. `CardRow::cells` gives the text columns, the first of `emails`, the first of `phones` and `fullName`, which `card list` and `pimdir search` (src/pimdir/search.rs) print.
- `card read` keeps printing the row plus its vCard, the row losing the same keys (src/shared/card/read.rs doc).
- The text layout keeps its four columns, `id  email  phone  name`; the values are the decoded ones, an escape decoded and a `tel:` URI as its number.
- Breaking against v0.3's `card list --json`: CHANGELOG Breaking line, MIGRATION.md (JSON output) with the jq before and after. `card read` and `pimdir search` carried the keys only since v0.3, so their lines are reworded.
