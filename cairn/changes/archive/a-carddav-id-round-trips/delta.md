---
cairn: delta
change: a-carddav-id-round-trips
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A CardDAV id round-trips
(backends) An address book or card id a CardDAV command prints SHALL be the one io-webdav reads off the server's href or the URL a write sent: its last segment, percent-decoded when the decoding encodes back to that segment, else as served (an encoded `/`, an escape the decoding leaves, `%40`, lowercase hex). It holds in every listing (`addressbook list`, `card list`, `carddav propfind`, `report query` and `report multiget`, their `errors` entries included) and in the id a write answers (`card add`, `carddav put`); `report sync` and `report raw` print the server's hrefs as served. An address book created by name SHALL print that name as its id, the one its listing shows, and every id printed SHALL address its resource when given back, io-webdav encoding it as it sends it.

#### Scenario: An address book named with a space
- GIVEN a CardDAV home set `/dav/`
- WHEN `cardamum --json addressbook create "My Book"` runs, then `cardamum --json addressbook list`
- THEN the create sends `MKCOL /dav/My%20Book/` and prints `"id":"My Book"`, and the listing, whose href is `/dav/My%20Book/`, prints `"id":"My Book"` too

## MODIFIED Requirements

### Requirement: Card ids are backend-native and verbatim
(backends) A card's shared `id` SHALL be the backend's own identifier, used verbatim end to end: the last path segment of the resource href, decoded as "A CardDAV id round-trips" says (CardDAV), the file stem (vdir), the `ContactCard` id (JMAP), the contact id (Graph), the person id (People), or the store-assigned public `seq` (pimdir). No adapter SHALL add or strip a file extension when addressing a resource. CardDAV names a new card `<uuid>.vcf` itself and passes the whole name through, since the server owns the resource name from then on. A CardDAV write SHALL report the id io-webdav answers, never the caller's string: `card add` and `carddav put` print the last segment of the `Location` the server names, else of the URL sent, decoded as a listing shows it.

## REMOVED Requirements
