---
cairn: change
id: show-and-cancel-print-the-handle
status: landed
created: 2026-11-19
---

# Show and cancel print the handle

## Why

`pimdir queue list` prints each row with its handle beside `row` (cairn/changes/archive/the-listed-rows-carry-their-handle), but `pimdir queue show` and `pimdir queue cancel` printed `row` alone, so a script holding a handle could not match their answer to the row it named. pimalaya-cli's output.md keeps `row` an integer with `handle` beside it wherever a queue row is printed, and Himalaya aligned its own `show` and `cancel` the same day (its b6eed3e8). Owner ruling 2026-11-19.

## What

`show` prints `{"row", "handle", "state", ...}` and `cancel` prints `{"row", "handle", "kind", "unconfirmed"}`: `row` the integer id, `handle` the handle given, or a bare id given named under the store's identity (`PimdirReader::change_cursor`). Their text names the row by its handle. `collection` stays the kit's shared key.
