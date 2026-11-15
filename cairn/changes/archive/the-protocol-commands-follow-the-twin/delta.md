---
cairn: delta
change: the-protocol-commands-follow-the-twin
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A former command name stays hidden
(commands) A command renamed SHALL keep its former name as an alias hidden from the help, the manual and the completions, so a script keeps working, and the alias SHALL have no schema of its own, its schema being its command's: `card create` for `card add` ("A card is added"), `carddav create` for `carddav mkcol` (`cardamum-carddav-mkcol`), the singular `gpeople` nouns for the plural ones ("Specific commands mirror their protocol"), as calendula keeps `caldav create` for `caldav mkcalendar`.

#### Scenario: The former name of mkcol
- GIVEN a CardDAV account
- WHEN `cardamum --json carddav create book -n Book` runs
- THEN it runs `carddav mkcol book -n Book` and prints `{"id":"book"}`, and `cardamum carddav --help` lists `mkcol` with no `create`

### Requirement: carddav propfind lists the home set or one address book
(commands) `carddav propfind` with no address book SHALL list the address book collections under the home set with a Depth 1 PROPFIND, `{"addressbooks": [{id, displayName, description, color, ctag, syncToken}]}`; with one, the cards it holds by id and entity tag, no body, through an `addressbook-query` REPORT asking `getetag` alone (RFC 6352 §8.6), as io-webdav enumerates a collection and as calendula's `caldav propfind` does, `{"cards": [{id, etag}], "truncated"}`, `truncated` when a 507 row cut the listing. Its help SHALL name the request each form sends.

#### Scenario: An address book enumerated
- GIVEN a CardDAV home set `/dav/` holding the address book `book`
- WHEN `cardamum carddav propfind book` runs
- THEN it sends `REPORT /dav/book/`, an `addressbook-query` asking `getetag` alone and no `address-data`

## MODIFIED Requirements

### Requirement: Text output is plain text
(commands) A text output SHALL be plain text: no border, no colour, nothing sized to or truncated for the terminal, and it SHALL end with a newline: the `card read` view of a card whose last line lacks one, and the pretty JSON of the `jmap`, `msgraph` and `gpeople` `request` commands, included. A list of short rows prints as aligned columns, a list of wide items as records, a single item as `Label:  value` fields (pimalaya-cli `Columns`, `Records`, `Fields`). `carddav discover` SHALL print its endpoints as fields, `Server`, `Principal` and `Addressbook home-set`, `(unresolved)` for one it could not resolve, as calendula's `caldav discover` prints its own.

#### Scenario: A card read through a REPORT
- GIVEN a pimdir card whose body ends `END:VCARD` with no line break, as a CardDAV `REPORT` stored it
- WHEN `cardamum card read` prints it
- THEN the text ends `END:VCARD` and one newline, while `--json` keeps the bytes

#### Scenario: A principal left unresolved
- GIVEN a CardDAV account naming its home, whose server answers no principal
- WHEN `cardamum carddav discover` runs
- THEN it prints the `Server:`, `Principal:` and `Addressbook home-set:` fields, the values aligned, `Principal:` reading `(unresolved)`, and `--json` prints `"principal":null`

### Requirement: A JSON error carries a stable code
(commands) One row of the code table changes, the rest of the requirement as it stands:

| Code | Raised when |
|---|---|
| `addressbook-not-found` | the address book a command names does not exist, or no address book can have its id (empty, not one vdir directory name, refused by io-webdav on CardDAV); a protocol command naming one Graph contact folder Graph does not hold (`msgraph contact-folders get`, `child-folders`, `rename` and `delete`, `msgraph contacts list`, `create` and `delta` under `-f`), Graph's message kept; nothing was read or written |

### Requirement: A server's refusal carries its code
(backends) One row of the code table changes, the rest of the requirement as it stands:

| Code | When |
|---|---|
| `addressbook-not-found` | an address book named that does not exist (vdir, pimdir); a JMAP AddressBook `notFound`; a CardDAV collection answered 404 or 410, a Graph folder 404 or a malformed folder id, a People group 404, an id JMAP's `AddressBook/get` does not hold ("A network backend names an unknown address book"); on the protocol commands naming one Graph folder as on the unified ones, Graph's message kept there |

## REMOVED Requirements
