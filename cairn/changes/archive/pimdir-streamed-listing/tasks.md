---
cairn: tasks
change: pimdir-streamed-listing
---

# Tasks

- [x] pimdir `scan` visiting keyset batches until its visitor breaks, in place of `scan_items` collecting them
- [x] `list_cards` counting the cards before the page off their rows, building the page's cards only, breaking once full
- [x] `paginate` built only for the backends that cut a whole list
- [x] Tests: every page and size against the whole-read listing on an address book spanning several batches, before and after a staged update and removal; bodies made unreadable before and after a page are never read
- [x] Fold into [backends](../../../spec/backends.md); log entry; changelog
