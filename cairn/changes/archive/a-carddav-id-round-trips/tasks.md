---
cairn: tasks
change: a-carddav-id-round-trips
---

# Tasks

- [x] io-webdav 0.10.0 in Cargo.lock (that crate alone) and Cargo.toml (`0.10.0`); the build needs no adaptation.
- [x] The id docs of src/carddav/propfind.rs (`AddressbookRow`, `CardRefRow`) and src/carddav/report/entries.rs (`EntryRow`).
- [x] Test: src/carddav/backend.rs `a_created_addressbook_prints_the_id_its_listing_shows` runs `addressbook create "My Book"` then `addressbook list` through the unified client over a scripted CardDAV server (`AddressbookClient::open_carddav`, test only): the MKCOL goes to `/dav/My%20Book/`, both print `My Book`, and a listed `a%2Fb` keeps its served form. On io-webdav 0.9.1 the listing prints `My%20Book` and the test fails.
- [x] CHANGELOG (Changed, the `carddav put` line amended), MIGRATION.md (The libraries, the `carddav put` line amended).
- [x] Fold the delta into backends.md, write the log entry, set `landed`, archive.
