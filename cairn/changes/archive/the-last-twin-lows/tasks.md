---
cairn: tasks
change: the-last-twin-lows
---

# Tasks

- [x] X9c: the duplicate sentence (src/error.rs, src/vdir/client.rs); the sentence asserted in tests/duplicate.rs and the `an_address_book_created_twice_is_a_duplicate` tests of src/carddav/backend.rs, src/msgraph/backend.rs, src/gpeople/backend.rs, each failing on the code before.
- [x] vdir item get: `encoding` through `CardContents` (src/vdir/item/get.rs); test `an_item_prints_its_bytes_as_text_or_base64`, which fails on the code before (no `encoding`, then the UTF-8 error).
- [x] L11: the four summaries (src/pimdir/cli.rs, src/vdir/cli.rs, src/jmap/cli.rs, src/gpeople/cli.rs); test `the_namespaces_read_as_himalaya` (src/cli.rs).
- [x] D27: nothing in cardamum; `Listing::page` moves in pimalaya-cli 0.5.3, the bump left to a later pass.
- [x] D28: `addressbook create --id` (src/shared/addressbook/create.rs, src/shared/client.rs, src/vdir/backend.rs, src/carddav/backend.rs, src/capability.rs); tests `a_vdir_address_book_takes_the_id_given`, `a_backend_minting_its_ids_refuses_the_id` (tests/addressbook_id.rs), `a_created_addressbook_takes_the_id_given` (src/carddav/backend.rs), which fail to parse `--id` on the code before.
- [x] CHANGELOG (Added, Changed, two reworded lines), MIGRATION.md (JSON output, the duplicate sentence).
- [x] Fold the delta into commands.md and backends.md, write the log entry, set `landed`, archive.
