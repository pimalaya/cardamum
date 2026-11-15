---
cairn: delta
change: the-twin-differences-close
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: An address book listing names its account
(commands) `addressbook list` SHALL print as a pimalaya-cli `listing::Listing`, as calendula's `calendar list` and Himalaya's `mailbox list` print theirs: `Account: <name> (<backend>)`, then a blank line and one line per address book with no column header, `id  default  color  name`, `default` on the one the server names the default, the free-text name last, `-` for an empty cell; the description is left to `--json`. Below the address books SHALL print only the command's errors ("A command's errors print at its end").

#### Scenario: Two address books
- GIVEN a vdir account `perso` holding `personal`, coloured `#ff0000`, and `work`, with no colour
- WHEN `cardamum addressbook list` runs
- THEN it prints `Account:    perso (vdir)`, a blank line, then `personal  -  #ff0000  Personal` and `work` with `-` in its default and colour cells, aligned, and no header row

### Requirement: A CardDAV read carries its vCard beside its encoding
(commands) `carddav get`, `carddav report query` and `carddav report multiget` SHALL carry each vCard as `card read` does ("A card read prints its listing row and its vCard") and as calendula's `caldav get`, `report query` and `report multiget` carry an item: `contents` beside `encoding`, the text when the bytes are valid UTF-8, `encoding` then `utf-8`, and their base64 (RFC 4648 standard alphabet, padded) otherwise, `encoding` then `base64` (pimalaya-cli output.md, "Bytes are never raw under JSON"). A card that is not UTF-8 SHALL read rather than fail, and the `carddav get` text SHALL print it as its bytes, as `card build` hands one over. A REPORT row's card is the `address-data` text io-webdav reads, so its `encoding` is `utf-8`.

#### Scenario: A CardDAV card that is not UTF-8
- GIVEN a CardDAV server answering the GET of `c1.vcf` with a vCard holding the byte `0xE9`
- WHEN `cardamum --json carddav get book c1.vcf` runs
- THEN it exits 0 and prints `id`, `etag`, `contents` holding the bytes in base64 and `"encoding":"base64"`

### Requirement: The collection arguments are named as the twin's
(commands) `vdir create` SHALL take the collection's directory name as `<ID>` and its display name as `-n/--display-name <NAME>`, written to the `displayname` file, the id written there when the flag is omitted; `vdir rename` SHALL take `<OLD-ID> <NEW-ID>` and `vdir delete` `<ID>`, as calendula's `vdir` commands name them. `addressbook update -n/--name` SHALL take a `<NAME>`, as calendula's `calendar update` does.

#### Scenario: A display name apart from the id
- GIVEN a vdir account
- WHEN `cardamum --json vdir create friends -n Friends` runs
- THEN it prints `{"id":"friends"}`, makes the directory `friends` and writes `Friends` to its `displayname` file, where `cardamum vdir create bare` writes `bare`

## MODIFIED Requirements

### Requirement: A former command name stays hidden
(commands) A command renamed SHALL keep its former name as an alias hidden from the help, the manual and the completions, so a script keeps working, and the alias SHALL have no schema of its own, its schema being its command's: `card create` for `card add` ("A card is added"), `carddav create` for `carddav mkcol` (`cardamum-carddav-mkcol`), the singular `gpeople` nouns for the plural ones ("Specific commands mirror their protocol"), as calendula keeps `caldav create` for `caldav mkcalendar`. A short name SHALL stay accepted the same way, hidden, where calendula shows none for the counterpart: `rm` for `addressbook delete` and `card delete`, `new` for `card add`, `folders` for `msgraph contact-folders`.

#### Scenario: The former name of mkcol
- GIVEN a CardDAV account
- WHEN `cardamum --json carddav create book -n Book` runs
- THEN it runs `carddav mkcol book -n Book` and prints `{"id":"book"}`, and `cardamum carddav --help` lists `mkcol` with no `create`

### Requirement: A card is added
(commands) The unified verb creating a card SHALL be `card add`, as Himalaya's `message add`, its MCP tool `card_add` and its schema `cardamum-card-add`. `card create`, the v0.3 name, SHALL stay accepted as a hidden alias, as SHALL `card new`. The capability it reaches is `card.add`, pimdir's `contacts.card.add`.

### Requirement: A not-found failure is cardamum's sentence
(commands) A card a command names that the address book does not hold SHALL fail with cardamum's own sentence on every backend, ``Card `<id>` not found in address book `<book>` ``, as Himalaya words a message not found (``Message `<id>` not found in mailbox `<mailbox>` ``), the backend's answer below it in `sources` (io-vdir's, the CardDAV, Graph or People refusal, the JMAP `notFound`; pimdir has no text of its own to add): the backend's wording never stands as the error. An item error SHALL NOT repeat its card's id in `error`, the id being its own key.

### Requirement: A JSON error carries a stable code
(commands) Its scenario "An unknown card" changes, the rest of the requirement as it stands:

#### Scenario: An unknown card
- GIVEN a vdir address book `book` holding no card `nope`
- WHEN `cardamum --json card read -k book nope` runs
- THEN the command exits 1 and prints a failure object coded `card-not-found`, its `error` ``Card `nope` not found in address book `book` `` and io-vdir's text in `sources`

### Requirement: Backend selection is local-first
(backends) The unified commands run over an `AddressbookClient` owning one `BackendClient` variant per compiled-in backend. The global `--backend` flag picks it: `auto` (the default) SHALL take the first configured-and-allowed backend in priority order, which puts the local stores (vdir, then pimdir) before the network ones so an offline store wins over a round-trip. A named value pins that backend and bails when the account has no matching config block. The order SHALL walk every block the account declares, the backends the build leaves out included, so the backend `auto` resolves to is the same in every build. The protocol-specific commands build their own client and ignore `--backend` entirely. The flag's help SHALL read as Himalaya's and list its possible values, `auto` then every backend an account may name in that order, built or not, each with Himalaya's line (``Pin the command to the account's <protocol> backend``); a value outside them SHALL be refused as a command line (`usage-invalid`, exit 2), the possible values listed.

#### Scenario: An unknown backend
- GIVEN any build
- WHEN `cardamum -b nope card list` runs
- THEN it exits 2 and the refusal lists `[possible values: auto, vdir, pimdir, carddav, jmap, msgraph, gpeople]`

### Requirement: A CardDAV id addresses its own resource
(backends) An address book or card id a CardDAV command names SHALL be refused, before the account's secret is resolved or anything connects, when io-webdav refuses it (`rfc4918::check_collection_id`, `check_resource_id`: empty, or holding an empty, `.` or `..` segment once split on `/` and `\` and percent-decoded, an address book id's trailing slashes and an absolute id's leading one aside), such an id addressing the home set, its parent or the server root: an address book id coded `addressbook-not-found`, ``Addressbook `<id>` not found``, a card id coded `card-not-found`, ``Card `<id>` not found in address book `<book>` ``, io-webdav's reason in `sources`, as vdir refuses an id that is not one directory name. Every unified command naming one SHALL refuse it so (`addressbook update` and `delete`, `card list`, `read`, `add`, `update` and `delete`), and every `carddav` command (`propfind`, `proppatch`, `get`, `put`, `delete`, `report query`, `multiget`, `sync` and `raw`). The name of an address book to create, its id on CardDAV, SHALL be refused the same way, uncoded as vdir refuses a new name: `addressbook create` and `carddav mkcol` with ``Addressbook name `<name>` names no CardDAV collection``, or ``Addressbook name cannot be empty``.

io-webdav refuses such an id itself before sending anything, but as an I/O error no code reads, and only once the account is open.

#### Scenario: An id climbing out of the home set
- GIVEN a CardDAV account whose password command would leave a file behind
- WHEN `cardamum --json addressbook delete ..` or `cardamum --json card read -k book ..` runs
- THEN it exits 1 coded `addressbook-not-found` (``Addressbook `..` not found``) or `card-not-found` (``Card `..` not found in address book `book` ``), the password command never ran and nothing connected

## REMOVED Requirements
