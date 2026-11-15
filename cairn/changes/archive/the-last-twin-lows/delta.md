---
cairn: delta
change: the-last-twin-lows
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A vdir item read carries its bytes beside their encoding
(commands) `vdir item get` SHALL carry the item as `carddav get` carries a vCard ("A CardDAV read carries its vCard beside its encoding") and as calendula's `vdir item get` carries one: `{id, kind, contents, encoding}`, `contents` the text when the bytes are valid UTF-8, `encoding` then `utf-8`, and their base64 (RFC 4648 standard alphabet, padded) otherwise, `encoding` then `base64`. An item that is not UTF-8 SHALL read rather than fail, and the text SHALL print it as its bytes.

#### Scenario: A vdir item that is not UTF-8
- GIVEN a vdir address book `book` holding `b.vcf`, whose bytes hold `0xE9`
- WHEN `cardamum --json vdir item get book b` runs
- THEN it exits 0 and prints `"kind":"vcard"`, `contents` holding the bytes in base64 and `"encoding":"base64"`

### Requirement: An address book is created under the id given
(commands) `addressbook create <NAME>` SHALL take an optional `--id <ID>`, as calendula's `calendar create <NAME> [--id <ID>]`: on vdir the directory and on CardDAV the URL segment, `<NAME>` then the display name alone, and the id the command prints. Without it the name SHALL stay the id, as before. JMAP, Microsoft Graph and Google People mint their own ids and pimdir creates no address book, so the four SHALL refuse `--id` before connecting (`addressbook.create.id` at `none`, "An unserved verb is refused before connecting").

#### Scenario: A vdir address book under an id
- GIVEN a vdir account
- WHEN `cardamum --json addressbook create "My Work" --id work` runs
- THEN it prints `{"id":"work","name":"My Work","queued":false}` and the home holds the directory `work`, displayed `My Work`

#### Scenario: A JMAP account
- GIVEN a JMAP account
- WHEN `cardamum --json addressbook create Work --id work` runs
- THEN it exits 1 coded `capability-missing` naming `addressbook.create.id`, nothing connected

## MODIFIED Requirements

### Requirement: Specific commands mirror their protocol
(commands) A protocol-specific command set SHALL match the remote protocol's own structure and vocabulary rather than the unified API's, following Himalaya's direction. The shape follows the protocol: CardDAV (WebDAV) is a flat list of WebDAV method names, JMAP, Microsoft Graph and Google People are nested by resource with sub-verbs naming the protocol methods, and vdir is flat collection verbs plus an `item` subcommand. Verb names track the protocol, so JMAP uses `destroy` while Graph and People use `delete`. A resource is named by the API's own plural noun, as Himalaya's `gmail` names its own: `gpeople connections`, `gpeople contact-groups` and `gpeople other-contacts`, the singular names (`connection`, `contact-group`, `other-contact`) accepted as hidden aliases beside the visible `people`, `contacts`, `group`, `groups` and `other`; `gpeople profile` reads the one signed-in user and stays singular. Each namespace SHALL read as Himalaya's in its summary line, `<API>-specific API`: `CardDAV-specific API`, `JMAP-specific API`, `Microsoft Graph-specific API`, `Google People-specific API`, `pimdir-specific API` and `vdir-specific API`, as calendula's namespaces read.

pimdir is a store rather than a protocol: its `pimdir` family covers `status`, what a reader can tell of the store behind the account, `queue list`, `queue show` and `queue cancel`, the writes cardamum queued, and `search` and `count`, the store's search index queried in the pimdir SEARCH language. The other operator commands (retrying a parked action, the retained items) ship as the separate `pimdir` binary in io-pimdir.

### Requirement: Capability registry
(commands) Each backend SHALL declare, in a table compiled into every build, each capability of the unified API (`addressbook.list`, `addressbook.create`, `addressbook.create.id`, `addressbook.rename`, `addressbook.delete`, `addressbook.description`, `addressbook.color`, `card.list`, `card.read`, `card.add`, `card.update`, `card.remove`, `card.update.if-match`, `card.remove.if-match`) at `full`, `partial`, `server` or `none`, every level but `full` with a detail saying what differs, which server refuses, or what serves it instead. A backend declares a capability only when it gives the result every other declarer gives.

### Requirement: An unserved verb is refused before connecting
(commands) A unified command SHALL check the capabilities it reaches, those its flags reach included (`--if-match`, an addressbook's id, name, description or colour), against the backend the account resolves to, before reading the store or connecting, and SHALL fail with `capability-missing` naming the backend, the capability and the detail.

### Requirement: A collection created twice is refused
(backends) `addressbook create` naming an address book the backend holds already SHALL fail coded `addressbook-duplicate`, ``Cannot create address book `<id>`: it exists already``, `<id>` the name or the id `--id` gave, as Himalaya words a mailbox created twice, exit 1, creating nothing and leaving the existing one as it was, on every backend that can tell, as pimalaya-cli's error.md asks of every collection noun. vdir: a directory of that name under the home, checked before anything is written, `vdir create` refused the same way, where io-vdir would create over it and rewrite its metadata. CardDAV: a `MKCOL` refused 405, the status RFC 4918 §9.3.1 gives a URL already mapped and RFC 5689 §3.2 keeps for the extended `MKCOL`, or 403, which Fastmail answers (testing/carddav-specific.md), on an id the home set then lists, one `PROPFIND` of the home set on that path alone; any other refusal, or a listing that fails or does not hold the id (iCloud refuses every `MKCOL`), keeps the server's. Microsoft Graph: a folder refused `ErrorFolderExists`. Google People: a contact group refused 409, which contactGroups.create answers a name the user holds. The server's answer SHALL be in `sources`. JMAP lets two AddressBooks share a name (RFC 9610 §2 asks no uniqueness), so it holds no duplicate to refuse, and pimdir creates no address book (`capability-missing`, "A cache renames nothing it cannot push"). The protocol commands creating a collection on a server (`carddav mkcol`, `msgraph contact-folders create`, `gpeople contact-groups create`, `jmap address-book create`) keep the server's refusal as it is.

#### Scenario: An address book created twice
- GIVEN a vdir home holding the address book `work`, named `Work` and described
- WHEN `cardamum --json addressbook create work` or `cardamum --json vdir create work -n Other` runs
- THEN each exits 1 coded `addressbook-duplicate`, ``Cannot create address book `work`: it exists already``, and `work` keeps its name and description

### Requirement: A CardDAV id round-trips
(backends) An address book or card id a CardDAV command prints SHALL be the one io-webdav reads off the server's href or the URL a write sent: its last segment, percent-decoded when the decoding encodes back to that segment, else as served (an encoded `/`, an escape the decoding leaves, `%40`, lowercase hex). It holds in every listing (`addressbook list`, `card list`, `carddav propfind`, `report query` and `report multiget`, their `errors` entries included) and in the id a write answers (`card add`, `carddav put`); `report sync` and `report raw` print the server's hrefs as served. An address book created by name SHALL print that name as its id, or the id `--id` gave, the one its listing shows, and every id printed SHALL address its resource when given back, io-webdav encoding it as it sends it.

#### Scenario: An address book named with a space
- GIVEN a CardDAV home set `/dav/`
- WHEN `cardamum --json addressbook create "My Book"` runs, then `cardamum --json addressbook list`
- THEN the create sends `MKCOL /dav/My%20Book/` and prints `"id":"My Book"`, and the listing, whose href is `/dav/My%20Book/`, prints `"id":"My Book"` too

## REMOVED Requirements

None.
