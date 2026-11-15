---
cairn: change
id: a-carddav-id-round-trips
status: landed
created: 2026-11-15
---

# A CardDAV id round-trips

> cardamum's part of line W8 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), the fix cardamum asked for as W7, released in io-webdav 0.10.0.

## Why

io-webdav 0.9.1 percent-encodes a relative id before sending it (W4) but kept the last segment of a listed href raw. `cardamum addressbook create "My Book"` sent `MKCOL /dav/My%20Book/` and printed `My Book`, while `addressbook list` and `carddav propfind` then printed `My%20Book`: both address the same collection, but the id a create prints is never one its listing shows, and a script matching the two finds nothing. The same held for a card whose name holds a byte a path segment cannot carry raw.

io-webdav 0.10.0 decodes every id it reads off an href or a sent URL when the decoding encodes back to the segment, and keeps the served form otherwise (an encoded `/`, an escape the decoding leaves, `%40`, lowercase hex), so an id printed is one the next request sends back unchanged.

## What

- io-webdav 0.10.0 (`cargo update -p io-webdav`, the lock diff that crate alone), the requirement raised to `0.10.0`. Its breaks (`WebdavResponseEntry::id` and `WebdavResponseOf::id` returning `Cow<str>`, `CaldavCalendarPatch`) touch no cardamum code: cardamum reads the typed listings' owned `id` fields and runs no CalDAV.
- Every CardDAV id cardamum prints follows: `addressbook list`, `card list`, `carddav propfind`, `report query` and `multiget` and their `errors` entries, and the id `card add` and `carddav put` print from the write's answer. `report sync` and `report raw` print hrefs, which stay as served.
- The `carddav` output docs (`AddressbookRow`, `CardRefRow`, `EntryRow`) say the id is decoded so, which their schemas carry.
- Spec: backends ("A CardDAV id round-trips", added; "Card ids are backend-native and verbatim", modified).
