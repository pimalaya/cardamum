---
cairn: change
id: pimdir-search-pages-by-token
status: landed
created: 2026-11-15
---

# pimdir search pages by token

## Why

`pimdir search` paged by `more` and `--page`, against pimalaya-cli's cairn/spec/output.md and the family (finding F12, owner ruling D9, task C9 of cli-alignment-2026-11-15), and failed whole on a hit whose body it could not read, having no `errors` list, while a hit whose body was gone from the blob tree listed as its preview with a log warning (task C20). Himalaya landed the shape on 2026-11-15 (c7016650, the same id).

## What

- `--page-token` (pimalaya-cli `PageTokenArg`), a page number from 1 winning over `--page`; `nextPage` replaces `more`. `--page` and `--page-size` stay.
- `errors`: a hit the store cannot read, whose body cannot be read, is gone from the blob tree or reads as no vCard, is an entry alone, coded as its failure is or `card-unreadable`, as `card list` names it; exit 3 when not empty. `card_from_item`, which previewed a missing body, goes.
- Text through pimalaya-cli's `Listing`, one line per hit as `card list` prints, the coverage an `Incomplete:` field above the hits; `pimdir count` keeps its shape.
- `pimdir search` is new in v0.4: no v0.3 user breaks; the CHANGELOG's Added entry and MIGRATION.md's "New" section describe the final shape.
