---
cairn: log
change: a-card-previews-its-folded-lines
date: 2026-11-15
---

# A card previews its folded lines

The `card list` columns and the `fnValue`, `email` and `tel` keys read a card's lines unfolded and a property whatever its group and parameters: Fastmail's `FN;DERIVED=TRUE` cards list their name, a folded JMAP `EMAIL` reads whole, `TELEX` is no `TEL`. Task C27 of cli-alignment-2026-11-15, live finding F1 (medium). Capability **commands** moved.

- **src/shared/card/list.rs**: `vcard_preview` over `unfold` (RFC 6350 §3.2) and `property` (§3.3: optional group, name, parameters, the first colon outside quotes); values as written.
- **Evidence**: `the_preview_reads_parameters_groups_and_folds` (`FN;DERIVED=TRUE:`, `EMAIL;PROP-ID=e1;TYPE=home:alan@examp` folded, `item1.TEL` with a quoted `:` and a tab fold, `fn:`, `TELEX` before `TEL`); on the code before `fnValue` was null and the email `alan@examp`.

CHANGELOG (Fixed). Spec: commands ("A card projects its common fields").
