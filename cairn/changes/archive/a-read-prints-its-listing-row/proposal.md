---
cairn: change
id: a-read-prints-its-listing-row
status: landed
created: 2026-11-15
---

# A read prints its listing row

## Why

`card read` printed `{id, etag, contents, …common fields}`, lacking the `addressbookId`, `fnValue`, `email` and `tel` its `card list` row carries, where pimalaya-cli's output.md says a read prints the row its list prints plus what only a read carries, with `id` and `etag` (finding F9, decision D7, task C12). A card whose bytes are not UTF-8 failed the read, uncoded, where output.md says vCard text is a string when UTF-8 and base64 otherwise (finding F14, task C13).

## What

- `card read` prints the `card list` row flattened (`CardRow`), then the vCard. A card that is not UTF-8 projects no common field, the parser reading such bytes lossily.
- The vCard is `contents`, a string: the text when the bytes are UTF-8, `encoding` `utf-8`, their base64 (standard, padded) otherwise, `encoding` `base64`, as calendula prints an item's bytes (its K4), following output.md's "Bytes are never raw under JSON" and its rule that a field never changes type between outputs. `card build` carries its card the same way, and in text prints a card that is not UTF-8 as its bytes.
- base64-codec becomes an unconditional dependency (it was jmap's alone).
- Breaking (new keys, `contents` nullable): MIGRATION.md says so.
