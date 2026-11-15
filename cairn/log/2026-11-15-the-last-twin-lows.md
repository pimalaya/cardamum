---
cairn: log
change: the-last-twin-lows
date: 2026-11-15
---

# The last twin lows

Line X13 of the CLI alignment round for cardamum: X9c, the `vdir item get` encoding, L11, and the owner's rulings D27 and D28. Capabilities **commands** and **backends** moved.

- **src/error.rs, src/vdir/client.rs**: a duplicate address book reads ``Cannot create address book `<id>`: it exists already``, Himalaya's form, on vdir, `vdir create`, CardDAV, Graph and People; the code and `sources` unchanged. Never shipped, so its [Unreleased] line is reworded.
- **src/vdir/item/get.rs**: `vdir item get` prints `{id, kind, contents, encoding}` through `CardContents`, as `carddav get` and calendula's `vdir item get`; an item that is not UTF-8 reads as base64 where it failed uncoded, its text printing the bytes. Breaking as `carddav get` was (CHANGELOG and MIGRATION.md lines widened).
- **src/pimdir/cli.rs, src/vdir/cli.rs, src/jmap/cli.rs, src/gpeople/cli.rs**: the namespace summaries read `pimdir-specific API`, `vdir-specific API`, `JMAP-specific API`, `Google People-specific API`, as Himalaya's and calendula's.
- **D27**: nothing here. `card list` and `pimdir search` print `Next page:` through pimalaya-cli's `Listing::page`, which pimalaya-cli moves after the items (0.5.3), and the protocol listings through `Paginated`, which prints it last already; the bump comes in a later pass.
- **src/shared/addressbook/create.rs, src/shared/client.rs, src/vdir/backend.rs, src/carddav/backend.rs, src/capability.rs**: `addressbook create <NAME> [--id <ID>]`, the vdir directory or the CardDAV URL segment, the name then the display name alone; the capability `addressbook.create.id`, `none` on JMAP, Graph, People (the server mints the id) and pimdir, which refuse `--id` before connecting. Additive.
- **Evidence**: tests/duplicate.rs and the three backend duplicate tests assert the new sentence; `an_item_prints_its_bytes_as_text_or_base64` (src/vdir/item/get.rs); `the_namespaces_read_as_himalaya` (src/cli.rs); `a_vdir_address_book_takes_the_id_given`, `a_backend_minting_its_ids_refuses_the_id` (tests/addressbook_id.rs), `a_created_addressbook_takes_the_id_given` (src/carddav/backend.rs). Each fails on the code before.

CHANGELOG (Added, Changed, two reworded lines), MIGRATION.md (JSON output, the duplicate sentence). Spec: commands ("A vdir item read carries its bytes beside their encoding", "An address book is created under the id given", added; "Specific commands mirror their protocol", "Capability registry", "An unserved verb is refused before connecting", modified), backends ("A collection created twice is refused", "A CardDAV id round-trips", modified).
