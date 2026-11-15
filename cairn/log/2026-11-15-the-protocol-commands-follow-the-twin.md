---
cairn: log
change: the-protocol-commands-follow-the-twin
date: 2026-11-15
---

# The protocol commands follow the twin

Four points where cardamum's protocol commands were off the family and calendula, which took cardamum's namespaces in K6, follows it. Line C36 of the CLI alignment round, for X3. Capabilities **commands** and **backends** moved.

- **src/carddav/mkcol.rs**: `create` a hidden alias of `carddav mkcol`, where it was visible, as calendula hides `caldav create`. Not breaking: the line still runs, its schema stays `cardamum-carddav-mkcol`.
- **src/carddav/discover.rs**: the text prints pimalaya-cli's `Fields`, `Server`, `Principal` and `Addressbook home-set`, `(unresolved)` for one not resolved, each value sanitized, where it printed bare `server: …` lines; the JSON is unchanged. Breaking for a script cutting the text (MIGRATION.md, Text output).
- **src/msgraph/client.rs**: `folder_refusal` codes a 404 or a 400 `ErrorInvalidIdMalformed` `addressbook-not-found`, Graph's message kept, as `contact_refusal` codes a contact and calendula's `calendar_refusal` a calendar; applied in src/msgraph/contact_folders/ (`get`, `child-folders`, `rename`, `delete`) and src/msgraph/contacts/ (`list`, `create`, and `delta` opening a round). They failed uncoded.
- **src/carddav/propfind.rs**: the help says an address book is enumerated through the `addressbook-query` REPORT asking `getetag` alone, as io-webdav does it, where it named a PROPFIND, and its JSON output names `truncated`.
- **Evidence**: `create_is_a_hidden_alias_of_mkcol` (src/carddav/cli.rs), `the_endpoints_print_as_fields` (src/carddav/discover.rs), `a_specific_command_on_a_missing_folder_is_coded` (src/msgraph/client.rs; with `folder_refusal` neutralised it fails on `get AAMk`, the code `None`), `an_addressbook_is_enumerated_through_a_report` (src/carddav/propfind.rs, the request each form sends).

CHANGELOG (Changed), MIGRATION.md (Text output). Spec: commands ("A former command name stays hidden" and "carddav propfind lists the home set or one address book", added; "Text output is plain text" and the `addressbook-not-found` row of "A JSON error carries a stable code", modified), backends (the `addressbook-not-found` row of "A server's refusal carries its code", modified).
