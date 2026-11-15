---
cairn: log
change: a-carddav-id-round-trips
date: 2026-11-15
---

# A CardDAV id round-trips

cardamum moves to io-webdav 0.10.0, which decodes the ids it reads off an href when the decoding encodes back to the segment: `addressbook create "My Book"` printed `My Book` while its listing printed `My%20Book`, and both now print `My Book`. cardamum's part of line W8 of the CLI alignment round, closing W7. Capability **backends** moved.

- **Cargo.toml, Cargo.lock**: io-webdav 0.10.0, the requirement `0.10.0`, the lock diff that crate alone. No code adapts: cardamum reads the typed listings' owned `id` fields, never `WebdavResponseEntry::id` (now `Cow<str>`), and runs no CalDAV (`CaldavCalendarPatch`).
- **Through io-webdav 0.10.0**: every CardDAV id cardamum prints is decoded so, in `addressbook list`, `card list`, `carddav propfind`, `report query` and `multiget` and their `errors` entries, and in the id `card add` and `carddav put` print; a segment whose decoding would not be sent back as served (`%2F`, `%40`, lowercase hex, an escape left) keeps its served form. `report sync` and `report raw` print hrefs, unchanged.
- **src/carddav/propfind.rs, src/carddav/report/entries.rs**: the id docs, which the schemas carry, say the id is decoded so.
- **src/shared/client.rs**: `AddressbookClient::open_carddav`, test only, as `open_msgraph`.
- **Evidence**: `a_created_addressbook_prints_the_id_its_listing_shows` (src/carddav/backend.rs) runs `addressbook create "My Book"` and `addressbook list` through the unified client over a scripted CardDAV server: the MKCOL goes to `/dav/My%20Book/`, both print `My Book`, a listed `a%2Fb` stays as served. With the lock moved back to io-webdav 0.9.1 (`cargo update -p io-webdav --precise 0.9.1`, Cargo.toml and Cargo.lock restored after) the listing printed `["My%20Book", "a%2Fb"]` and the test failed.

CHANGELOG (Changed; the `carddav put` line no longer says percent-encoded), MIGRATION.md (The libraries; the `carddav put` line). Spec: backends ("A CardDAV id round-trips", added; "Card ids are backend-native and verbatim", modified).
