---
cairn: change
id: pimdir-streamed-listing
status: landed
created: 2026-10-13
---

# A pimdir card listing streams the store instead of loading it

## Why

`card list` on pimdir gathered every row of the address book, read every card's body, then cut the page out of the whole list: a page of 25 held every body of the address book at once. That is memory growing with the store rather than with the answer, the failure the Pimalaya Android client met, and reads a page never needed.

## What

- `list_cards` walks the address book in keyset batches, holding one batch of rows and the page it returns.
- The cards before the page are counted off their rows, with no body read; a body is read only for a card the page returns.
- The walk stops once the page is full.

The output is unchanged: same cards, same order, same bodies, previews and versions, the pending queue folded in as before.
