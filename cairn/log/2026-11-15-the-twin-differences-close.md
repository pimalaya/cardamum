---
cairn: log
change: the-twin-differences-close
date: 2026-11-15
---

# The twin differences close

The X3 rows of the CLI alignment round left to cardamum (M5, L1, L3, L5, L8, L9, L19, L23; line X7). Capabilities **commands** and **backends** moved.

- **src/carddav/get.rs, src/carddav/report/entries.rs**: `carddav get`, `report query` and `report multiget` carry `encoding` beside `contents` through `CardContents`, as `card read` and calendula's `caldav` reads; `carddav get` reads a card that is not UTF-8 as base64 where it failed uncoded, its text printing the bytes. Breaking for a script reading such a card's `contents` (MIGRATION.md, JSON output). A REPORT row stays `utf-8`, io-webdav reading `address-data` as text.
- **src/shared/addressbook/list.rs**: the text prints through `Listing`, `Account:` then `id  default  color  name` lines with no header row, as calendula's `calendar list` and Himalaya's `mailbox list`; the JSON is unchanged.
- **src/vdir/**: `vdir create <ID> -n/--display-name <NAME>` (the id still written as the display name without `-n`), `vdir rename <OLD-ID> <NEW-ID>`, `vdir delete <ID>`; **src/shared/addressbook/update.rs**: `-n <NAME>`. Value names and one flag added, nothing breaks.
- **src/shared/addressbook/cli.rs, src/shared/card/cli.rs, src/msgraph/cli.rs**: `rm`, `new` and `folders` hidden aliases.
- **src/backend.rs, src/cli.rs**: `-b` parsed over its possible values, `auto` then every backend an account may name, built or not, each with Himalaya's line, then through `Backend::from_str`, so `-b nope` lists them and a backend left out still names its feature; the help in Himalaya's words.
- **src/error.rs**: `card_not_found` names the address book, ``Card `<id>` not found in address book `<book>` ``, on every backend and the CardDAV id check; pimdir words the same. Codes unchanged.
- **Docs**: commands.md "A card is added" names `card create` as the hidden v0.3 name; MIGRATION.md "The libraries" lists the versions Cargo.toml requires.
- **Evidence**: `a_card_reads_beside_its_encoding`, `a_card_that_is_not_utf8_reads_as_base64` (src/carddav/get.rs), `a_multiget_reading_every_card_prints_no_error` (src/carddav/report/entries.rs), `address_books_print_one_line_each_under_the_account` (src/shared/addressbook/list.rs), `the_collection_arguments_read_as_the_twin` (src/vdir/cli.rs), `the_display_name_is_its_own_flag` (src/vdir/create.rs), `the_new_name_is_a_name` (src/shared/addressbook/update.rs), `the_short_names_are_hidden_aliases`, `the_backend_flag_lists_its_values`, `a_backend_left_out_names_its_feature` (src/cli.rs), the sentence in tests/not_found.rs, tests/carddav_id.rs and src/jmap/backend.rs; each fails on the code before.

CHANGELOG (Changed, one Breaking), MIGRATION.md (Text output, JSON output, The libraries). Spec: commands ("An address book listing names its account", "A CardDAV read carries its vCard beside its encoding", "The collection arguments are named as the twin's", added; "A former command name stays hidden", "A card is added", "A not-found failure is cardamum's sentence" and the "An unknown card" scenario of "A JSON error carries a stable code", modified), backends ("Backend selection is local-first" and "A CardDAV id addresses its own resource", modified).
