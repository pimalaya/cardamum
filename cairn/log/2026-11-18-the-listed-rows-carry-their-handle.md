---
cairn: log
change: the-listed-rows-carry-their-handle
landed: 2026-11-18
---

# The listed rows carry their handle

`pimdir queue list` rows and `pimdir status` parked rows print their handle (`<store-id>.<incarnation>.<id>`, pimdir STORAGE §4.5) beside `row` in the JSON and in its place in the text, so a row cancelled from a listing is checked against a restored copy of the store as a handle given by hand is (cairn/changes/archive/the-listed-rows-carry-their-handle). The twin of calendula's change of the same name (its f02e96e), same shape: `PimdirBackend::store_identity` reads `PimdirReader::change_cursor`, public in io-pimdir 0.17.0, and `handle` formats `PimdirActionHandle`. The 2026-11-18 io-pimdir-0-17-and-card-search entry was wrong to say a reader exposed no store identity; no io-pimdir release is needed.

Tests: `a_pending_creation_and_a_parked_update_share_the_row` parses the listed handle, checks it names this store, follows it to `pending` and cancels the row by it; `status_shows_the_parked_edit_of_the_account_read` parses the status handle back to the row and finds it in the text and the JSON; `a_listing_keys_its_rows_as_the_family_does` prints each row by its handle.

Moved: commands (pimdir status, pimdir queue).

Left: no receipt prints a handle, the unified card writes carrying no pimdir row; `handle` is a key cardamum and calendula add to the family's queue row shape (pimalaya-cli output.md), which Himalaya does not print yet.
