---
cairn: log
change: duplicates-pages-and-queue-rows
date: 2026-11-15
---

# Duplicates, pages and queue rows follow the family

Rulings D17, D18 and D19 with D21 of the CLI alignment round, pimalaya-cli's contract at 595504e (error.md, output.md). Capabilities **backends** and **commands** moved.

- **src/error.rs, src/vdir/, src/carddav/backend.rs, src/msgraph/, src/gpeople/**: `addressbook-duplicate`, ``Addressbook `<name>` already exists``. vdir refuses a name the home holds in `addressbook create` and `vdir create` before writing, where io-vdir created over it and rewrote its metadata. CardDAV lists the home set after a `MKCOL` refused 403 (Fastmail) or 405 (RFC 4918 §9.3.1, kept by RFC 5689 §3.2) and codes it when the id is listed. Graph codes `ErrorFolderExists`, People the 409 contactGroups.create answers a held name. JMAP cannot tell (RFC 9610 §2 asks no unique name); pimdir creates no address book. Breaking (MIGRATION.md, Commands that now fail earlier).
- **src/gpeople/, src/msgraph/, src/jmap/contact_card/query.rs, src/json_schema.rs**: the People and Graph listings and the delta print pimalaya-cli's `Paginated` around their raw payload, the provider's cursor lifted into `nextPage`, and take `--page-token`: People's `pageToken`, Graph's link got as is (io-msgraph refusing one off its origin), the flags the link carries conflicting with it, `child-folders` taking no folder id with it. The delta's `@odata.deltaLink` stays raw, `--delta-link` taking it. `jmap contact-card query` prints the next position and sends it back as `position`. The People searches print no cursor. Breaking (MIGRATION.md, JSON output).
- **src/pimdir/queue/list.rs, src/pimdir/backend.rs**: one `PimdirQueueRow`, pending and parked, the family's ten keys plus `fullName`; a parked row's `linkId`, `itemId` and full name read from its payload. `pimdir queue list` is new since v0.3, so nothing breaks there.
- **Evidence**: `an_address_book_created_twice_is_a_duplicate` (tests/duplicate.rs, src/carddav/backend.rs, src/msgraph/backend.rs, src/gpeople/backend.rs), `a_people_listing_pages_by_next_page` (src/gpeople/client.rs), `a_graph_listing_pages_by_next_page` (src/msgraph/client.rs), `a_limited_query_names_its_next_page` (src/jmap/contact_card/query.rs), `a_listing_keys_its_rows_as_the_family_does` (src/pimdir/queue/list.rs), `a_pending_creation_and_a_parked_update_share_the_row` (src/pimdir/backend.rs); each fails on the code before, the vdir and CardDAV ones checked by disabling the new check.

CHANGELOG (two Breaking lines, one Added, the `pimdir queue list` line), MIGRATION.md (JSON output, Commands that now fail earlier, New). Spec: backends ("A collection created twice is refused" added, "A server's refusal carries its code" modified), commands ("Protocol listings page by nextPage" added; "The pimdir queue is read and retracted", "Specific commands are raw-faithful", "JSON output keys are camelCase" and "A JSON error carries a stable code" modified).
