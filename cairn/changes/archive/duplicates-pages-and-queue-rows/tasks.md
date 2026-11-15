---
cairn: tasks
change: duplicates-pages-and-queue-rows
---

# Tasks

- [x] D17: `addressbook-duplicate` (src/error.rs), raised by vdir (src/vdir/client.rs, backend.rs, create.rs), CardDAV (src/carddav/backend.rs), Graph (src/msgraph/client.rs, backend.rs) and People (src/gpeople/client.rs, backend.rs); tests `an_address_book_created_twice_is_a_duplicate` in tests/duplicate.rs (exits 0 and rewrites the display name on the code before, checked by disabling the check), src/carddav/backend.rs (403 read `permission-denied`, 405 uncoded before, checked the same way), src/msgraph/backend.rs and src/gpeople/backend.rs (uncoded before).
- [x] D18: `Paginated` and `--page-token` on the People and Graph listings and `jmap contact-card query` (src/gpeople/, src/msgraph/, src/jmap/contact_card/query.rs, src/json_schema.rs); tests `a_people_listing_pages_by_next_page` (src/gpeople/client.rs), `a_graph_listing_pages_by_next_page` (src/msgraph/client.rs), `a_limited_query_names_its_next_page` (src/jmap/contact_card/query.rs), each refused at parse or reading the old cursor key on the code before.
- [x] D19: `PimdirQueueRow` (src/pimdir/queue/list.rs, src/pimdir/backend.rs); tests `a_listing_keys_its_rows_as_the_family_does`, `a_pending_creation_and_a_parked_update_share_the_row`, and the row assertions of `a_queued_card_lists_in_the_queue_until_cancelled` and `a_parked_row_lists_with_its_kind`, none building on the code before.
- [x] CHANGELOG (two Breaking lines, one Added, the `pimdir queue list` line), MIGRATION.md (JSON output, Commands that now fail earlier, New).
- [x] Fold the delta into backends.md and commands.md, write the log entry, set `landed`, archive.
