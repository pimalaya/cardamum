---
cairn: tasks
change: the-protocol-commands-follow-the-twin
---

# Tasks

- [x] `carddav create` hidden (src/carddav/mkcol.rs); test src/carddav/cli.rs `create_is_a_hidden_alias_of_mkcol`: the line parses as `mkcol` with its flags, `create` its one alias, none visible.
- [x] `carddav discover` text through `Fields` (src/carddav/discover.rs); test `the_endpoints_print_as_fields`, the text and the JSON.
- [x] `folder_refusal` (src/msgraph/client.rs) on `msgraph contact-folders get`, `child-folders`, `rename`, `delete` and `msgraph contacts list`, `create`, `delta` (src/msgraph/contact_folders/, src/msgraph/contacts/); test `a_specific_command_on_a_missing_folder_is_coded`: each of the seven over a 404 and a 400 `ErrorInvalidIdMalformed` is `addressbook-not-found`, Graph's message kept, a 403 `permission-denied`. With `folder_refusal` neutralised the test fails on the first line (`None`).
- [x] `carddav propfind` help (src/carddav/propfind.rs); test `an_addressbook_is_enumerated_through_a_report`: a PROPFIND of the home set without an address book, an `addressbook-query` REPORT asking `getetag` and no `address-data` with one.
- [x] CHANGELOG (Changed), MIGRATION.md (Text output).
- [x] Fold the delta into commands.md and backends.md, write the log entry, set `landed`, archive.
