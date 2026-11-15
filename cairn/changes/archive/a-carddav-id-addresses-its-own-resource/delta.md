---
cairn: delta
change: a-carddav-id-addresses-its-own-resource
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A CardDAV id addresses its own resource
(backends) An address book or card id a CardDAV command names SHALL be refused, before the account's secret is resolved or anything connects, when io-webdav refuses it (`rfc4918::check_collection_id`, `check_resource_id`: empty, or holding an empty, `.` or `..` segment once split on `/` and `\` and percent-decoded, an address book id's trailing slashes and an absolute id's leading one aside), such an id addressing the home set, its parent or the server root: an address book id coded `addressbook-not-found`, ``Addressbook `<id>` not found``, a card id coded `card-not-found`, ``Card `<id>` not found``, io-webdav's reason in `sources`, as vdir refuses an id that is not one directory name. Every unified command naming one SHALL refuse it so (`addressbook update` and `delete`, `card list`, `read`, `add`, `update` and `delete`), and every `carddav` command (`propfind`, `proppatch`, `get`, `put`, `delete`, `report query`, `multiget`, `sync` and `raw`). The name of an address book to create, its id on CardDAV, SHALL be refused the same way, uncoded as vdir refuses a new name: `addressbook create` and `carddav mkcol` with ``Addressbook name `<name>` names no CardDAV collection``, or ``Addressbook name cannot be empty``.

io-webdav refuses such an id itself before sending anything, but as an I/O error no code reads, and only once the account is open.

#### Scenario: An id climbing out of the home set
- GIVEN a CardDAV account whose password command would leave a file behind
- WHEN `cardamum --json addressbook delete ..` or `cardamum --json card read -k book ..` runs
- THEN it exits 1 coded `addressbook-not-found` (``Addressbook `..` not found``) or `card-not-found` (``Card `..` not found``), the password command never ran and nothing connected

### Requirement: A CardDAV card reads whole
(backends) A card a CardDAV listing (`card list`, `carddav report query`) or `carddav report multiget` reads SHALL carry the whole `address-data` text, the final CRLF a server escapes as `&#13;` included, byte-identical to the body `card read` and `carddav get` return.

### Requirement: A raw CardDAV report addresses and counts as io-webdav does
(commands) `carddav report raw` SHALL send its REPORT to the path io-webdav builds for the address book (`rfc4918::collection_path`): an absolute id as given, a relative one joined to the home set and percent-encoded, a slash closing it; the id is refused first, as every CardDAV command refuses one ("A CardDAV id addresses its own resource" in backends). Its `dataBytes` SHALL be the size of the row's whole `address-data` text, the final CRLF included, the size of the body `carddav get` returns.

#### Scenario: A card whose CRLF the server escapes
- GIVEN a home set `/dav/` whose server answers a REPORT with one 54-byte card ending in `END:VCARD&#13;` and a line feed
- WHEN `cardamum --json carddav report raw "My Book" '<x/>'` runs
- THEN the request line is `REPORT /dav/My%20Book/ HTTP/1.1` and the row's `dataBytes` is 54

## MODIFIED Requirements

### Requirement: A JSON error carries a stable code
(commands) Two rows of the code table change, the rest of the requirement as it stands:

| Code | Raised when |
|---|---|
| `card-not-found` | the card a command names is not in the address book: the failure of `card read`, `card update` or `card delete`, worded by cardamum ("A not-found failure is cardamum's sentence"); an `errors` entry of `carddav report multiget`; a protocol command naming one card (`carddav get` and `delete`, `msgraph contacts get`, `update` and `delete`, `gpeople connections get`, `update` and `delete`), the server's message kept; a card id no CardDAV card can have, before anything connects (backends, "A CardDAV id addresses its own resource") |
| `addressbook-not-found` | the address book a command names does not exist, or no address book can have its id (empty, not one vdir directory name, refused by io-webdav on CardDAV); nothing was read or written |

## REMOVED Requirements
