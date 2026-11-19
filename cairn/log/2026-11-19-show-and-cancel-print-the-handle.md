---
cairn: log
change: show-and-cancel-print-the-handle
landed: 2026-11-19
---

# Show and cancel print the handle

`pimdir queue show` and `pimdir queue cancel` print `handle` beside the integer `row`, as `pimdir queue list` does, aligned with Himalaya's b6eed3e8 and pimalaya-cli's output.md (owner ruling 2026-11-19, cairn/changes/archive/show-and-cancel-print-the-handle). Given a handle, `handle` is the handle given; given a bare id, the id named under the store's identity, read by `PimdirBackend::store_identity` through `PimdirReader::change_cursor`, which the new `PimdirBackend::named` does. The text names the row by its handle (`Row <handle> is pending`, `Queued add cancelled (row <handle>)`).

Tests: `show_and_cancel_print_the_row_with_its_handle` runs both commands on a queued card, by bare id and by handle, and checks `row`, `handle` and the text; it failed first on the old shape (`handle` null). The show and cancel unit tests carry the handle.

Moved: commands (pimdir queue).
