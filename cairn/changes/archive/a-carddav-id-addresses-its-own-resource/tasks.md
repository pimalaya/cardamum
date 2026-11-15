---
cairn: tasks
change: a-carddav-id-addresses-its-own-resource
---

# Tasks

- [x] io-webdav 0.9.1 in Cargo.lock (that crate alone) and Cargo.toml (`0.9.1`).
- [x] `check_addressbook_id`, `check_card_id` and `check_addressbook_name` in src/carddav/client.rs; the unified client checks before opening a CardDAV backend (src/shared/client.rs), the `carddav` commands before building their client (src/carddav/cli.rs, src/carddav/report/cli.rs, src/cli.rs); `CarddavBackend::create_addressbook` leaves its empty-name refusal to the check.
- [x] `carddav report raw`: id checked, path through `collection_path`, `dataBytes` from `prop(ADDRESS_DATA)`'s whole text.
- [x] Tests: tests/carddav_id.rs runs the binary over a CardDAV account whose password command leaves a file: refused ids on every unified and `carddav` command exit 1 coded, the file never created; a clean id goes on to open the account. src/carddav/report/raw.rs: the whole card counted on an encoded path, an id out of the home refused with nothing sent. src/carddav/backend.rs: a listed card keeps its final CRLF. With the checks neutralised and `text` read again, the three binary tests and both raw tests fail; the listing test pins io-webdav 0.9.1's W3 (verified failing on a trimmed read by io-webdav's own test).
- [x] CHANGELOG (Changed, Fixed).
- [x] Fold the delta into backends.md and commands.md, write the log entry, set `landed`, archive.
