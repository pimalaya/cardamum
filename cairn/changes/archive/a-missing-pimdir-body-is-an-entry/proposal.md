---
cairn: change
id: a-missing-pimdir-body-is-an-entry
status: landed
created: 2026-11-15
---

# A missing pimdir body is an entry

## Why

A pimdir card whose item names a body its blob tree no longer holds (an inconsistent store) listed as its summary preview with a warning in the log, exit 0, where calendula names the same case an `item-unreadable` entry in `errors` (task C19 of cli-alignment-2026-11-15, found by C2's agent; 2026-11-15-an-unreadable-card-is-an-entry-alone left it as it was).

## What

- `card list` names such a card in `errors`, coded `card-unreadable`, ``Body blob missing from the store`` (the id in its own key, not repeated), no row listed, its place kept in the paging; the command exits 3. A body never fetched stays no entry.
- `pimdir search` keeps previewing such a card with the warning: it has no `errors` until it moves to `nextPage` (C20).
