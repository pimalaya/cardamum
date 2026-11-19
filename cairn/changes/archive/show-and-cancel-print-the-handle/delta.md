---
cairn: change
id: show-and-cancel-print-the-handle
status: landed
created: 2026-11-19
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: pimdir reads and retracts its queue (commands.md)
`show` SHALL print `{"row", "handle", "state", "collection", "kind", "attempts", "error", "appliedAt", "seq", "refusals"}` and `cancel` `{"row", "handle", "kind", "unconfirmed"}`: `row` the integer id, `handle` a string, the handle given, or a bare id given named under the store's identity (`PimdirReader::change_cursor`), as `queue list` prints it. Their text SHALL name the row by its handle.

## REMOVED Requirements

None.
