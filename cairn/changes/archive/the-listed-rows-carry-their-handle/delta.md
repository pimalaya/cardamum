---
cairn: change
id: the-listed-rows-carry-their-handle
status: landed
created: 2026-11-18
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: pimdir status reports the store (commands.md)
Each parked row SHALL carry `handle` beside `row`, `{"row", "handle", "addressbookId", "kind", "error"}`, and the text SHALL print the handle in the ROW column.

### Requirement: pimdir reads and retracts its queue (commands.md)
Each `queue list` row SHALL carry `handle`, `<store-id>.<incarnation>.<id>` (pimdir STORAGE §4.5, the identity read from `PimdirReader::change_cursor`), beside the family's keys, and the text SHALL print each row by its handle. `<ROW>` takes a handle as `queue list` and `pimdir status` print it.

## REMOVED Requirements

None.
