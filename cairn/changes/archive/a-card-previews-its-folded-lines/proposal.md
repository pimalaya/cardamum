---
cairn: change
id: a-card-previews-its-folded-lines
status: landed
created: 2026-11-15
---

# A card previews its folded lines

## Why

Live testing of 2026-11-15 (reports/cardamum/part2-pimdir.md F1, medium; jmap-fastmail.md, carddav-fastmail.md): `vcard_preview` in src/shared/card/list.rs scanned raw lines, matched `FN:` literally and never unfolded. A Fastmail card's `FN;DERIVED=TRUE:Jane Marie Doe` listed nameless (`fnValue` null) on every backend it travels through, and Fastmail's JMAP projection, folding every line its `PROP-ID` makes longer than 75 octets, listed `alan@examp` as an email. Task C27 of cli-alignment-2026-11-15.

## What

- Unfold the lines first (RFC 6350 §3.2), then read a property by its whole name, case-insensitive, whatever its group (`item1.EMAIL`) and parameters, its value from the first colon outside a quoted parameter value (§3.3). `TELEX` no longer reads as `TEL`.
- Values stay as written, escapes kept and a `tel:` URI whole, as the legacy keys always were; the decoded values are `fullName`, `emails` and `phones`.
