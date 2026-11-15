---
cairn: tasks
change: the-twin-differences-close
---

# Tasks

- [x] M5: `encoding` on `carddav get`, `report query`, `report multiget` (src/carddav/get.rs, src/carddav/report/entries.rs, query.rs, multiget.rs); tests `a_card_reads_beside_its_encoding` and `a_card_that_is_not_utf8_reads_as_base64` (src/carddav/get.rs: the first fails on the code before for the missing key, the second on its UTF-8 failure), `a_multiget_reading_every_card_prints_no_error` (src/carddav/report/entries.rs, the key asserted).
- [x] L1: `addressbook list` text through `Listing` (src/shared/addressbook/list.rs); tests `address_books_print_one_line_each_under_the_account` and `the_errors_print_last_after_one_blank_line`, which read the header row on the code before.
- [x] L3 and L8: `vdir create <ID> -n/--display-name <NAME>`, `vdir rename <OLD-ID> <NEW-ID>`, `vdir delete <ID>` (src/vdir/), `addressbook update -n <NAME>` (src/shared/addressbook/update.rs); tests `the_collection_arguments_read_as_the_twin` (src/vdir/cli.rs), `the_display_name_is_its_own_flag` (src/vdir/create.rs), `the_new_name_is_a_name` (src/shared/addressbook/update.rs).
- [x] L5: `rm`, `new`, `folders` hidden (src/shared/addressbook/cli.rs, src/shared/card/cli.rs, src/msgraph/cli.rs); test `the_short_names_are_hidden_aliases` (src/cli.rs).
- [x] L9: `-b` help and possible values (src/backend.rs `parser`, src/cli.rs); tests `the_backend_flag_lists_its_values`, `a_backend_left_out_names_its_feature`, `a_built_backend_is_taken_by_its_name` (src/cli.rs).
- [x] L19: ``Card `<id>` not found in address book `<book>` `` (src/error.rs and its callers, src/pimdir/backend.rs); tests `a_vdir_card_read_of_an_unknown_id_is_a_card_not_found` and `a_pimdir_card_read_of_an_unknown_id_is_a_card_not_found` (tests/not_found.rs), `a_card_id_out_of_its_addressbook_is_not_found` (tests/carddav_id.rs), `a_missing_card_reads_as_a_card_not_found` (src/jmap/backend.rs), each asserting the sentence the code before did not print.
- [x] L23: commands.md "A card is added" names `card create` as the hidden v0.3 name; MIGRATION.md "The libraries" lists the versions of Cargo.toml.
- [x] CHANGELOG (Changed, one Breaking), MIGRATION.md (Text output, JSON output, The libraries).
- [x] Fold the delta into commands.md and backends.md, write the log entry, set `landed`, archive.
