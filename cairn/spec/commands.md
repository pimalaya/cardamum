---
cairn: spec
capability: commands
status: current
---

# Commands

The command tree splits into three groups. The unified API (`addressbook`, `card`) is the cross-protocol surface, serving the union of what the backends can do, each declaring its level per capability; it behaves the same whatever backend serves the active account where that backend serves a verb. The protocol-specific APIs each expose the full surface of one backend, including operations the unified API cannot model. The meta commands (`account`, `completion`, `manual`, `json-schema`) cover account inspection, shell completions, man pages and the JSON Schemas of the structured output.

### Requirement: The top-level help orients a newcomer
A bare `cardamum` SHALL print the top-level help, which SHALL end with the shared Pimalaya footer (`pimalaya_cli::footer`) pointing at the issue tracker and the sponsoring page, as every other Pimalaya CLI does.

### Requirement: Subcommands carry their own data and chain
Each subcommand SHALL be a clap-derived struct holding its own arguments, with an `execute(self, printer, client)` method. `Command::execute` in [cli.rs](../../src/cli.rs) is the single dispatch point: it resolves the account, builds the appropriate client, and hands it to the subcommand.

### Requirement: The doc comment is the help page
Each command's doc comment SHALL be its help text: the first paragraph (two lines maximum) is the short summary `-h` shows, and the full text `--help` shows SHALL end with the command's JSON output shape. `cardamum <command> --help` is therefore the canonical usage reference for both humans and agents, and the README documents no per-command usage.

### Requirement: Specific commands mirror their protocol
A protocol-specific command set SHALL match the remote protocol's own structure and vocabulary rather than the unified API's, following Himalaya's direction. The shape follows the protocol: CardDAV (WebDAV) is a flat list of WebDAV method names, JMAP, Microsoft Graph and Google People are nested by resource with sub-verbs naming the protocol methods, and vdir is flat collection verbs plus an `item` subcommand. Verb names track the protocol, so JMAP uses `destroy` while Graph and People use `delete`. A resource is named by the API's own plural noun, as Himalaya's `gmail` names its own: `gpeople connections`, `gpeople contact-groups` and `gpeople other-contacts`, the singular names (`connection`, `contact-group`, `other-contact`) accepted as hidden aliases beside the visible `people`, `contacts`, `group`, `groups` and `other`; `gpeople profile` reads the one signed-in user and stays singular.

pimdir is a store rather than a protocol: its `pimdir` family covers `status`, what a reader can tell of the store behind the account, `queue list`, `queue show` and `queue cancel`, the writes cardamum queued, and `search` and `count`, the store's search index queried in the pimdir SEARCH language. The other operator commands (retrying a parked action, the retained items) ship as the separate `pimdir` binary in io-pimdir.

### Requirement: A former command name stays hidden
A command renamed SHALL keep its former name as an alias hidden from the help, the manual and the completions, so a script keeps working, and the alias SHALL have no schema of its own, its schema being its command's: `card create` for `card add` ("A card is added"), `carddav create` for `carddav mkcol` (`cardamum-carddav-mkcol`), the singular `gpeople` nouns for the plural ones ("Specific commands mirror their protocol"), as calendula keeps `caldav create` for `caldav mkcalendar`. A short name SHALL stay accepted the same way, hidden, where calendula shows none for the counterpart: `rm` for `addressbook delete` and `card delete`, `new` for `card add`, `folders` for `msgraph contact-folders`.

#### Scenario: The former name of mkcol
- GIVEN a CardDAV account
- WHEN `cardamum --json carddav create book -n Book` runs
- THEN it runs `carddav mkcol book -n Book` and prints `{"id":"book"}`, and `cardamum carddav --help` lists `mkcol` with no `create`

### Requirement: carddav propfind lists the home set or one address book
`carddav propfind` with no address book SHALL list the address book collections under the home set with a Depth 1 PROPFIND, `{"addressbooks": [{id, displayName, description, color, ctag, syncToken}]}`; with one, the cards it holds by id and entity tag, no body, through an `addressbook-query` REPORT asking `getetag` alone (RFC 6352 §8.6), as io-webdav enumerates a collection and as calendula's `caldav propfind` does, `{"cards": [{id, etag}], "truncated"}`, `truncated` when a 507 row cut the listing. Its help SHALL name the request each form sends.

#### Scenario: An address book enumerated
- GIVEN a CardDAV home set `/dav/` holding the address book `book`
- WHEN `cardamum carddav propfind book` runs
- THEN it sends `REPORT /dav/book/`, an `addressbook-query` asking `getetag` alone and no `address-data`

### Requirement: pimdir status reports the store and its parked actions
`pimdir status`, gated on the `pimdir` feature, SHALL report the account being read, every account the store groups collections under, one row per address book of that account (how many cards it holds, how many of them carry a local body, how many creations are queued for the next sync), and every action the store's owner parked in those address books (`PimdirReader::parked_actions`), by its queue row (ROW), its address book, its kind and the error that parked it. A parked action addresses no card, so its row is the only id shown, under its own name. Another account's address books and parked rows SHALL NOT show. Its JSON output is `{"account", "accounts", "addressbooks": [{"id", "name", "total", "hydrated", "queued"}], "parked": [{"row", "addressbookId", "kind", "error"}]}`, `parked` always present, `kind` the key a queue row names its action kind by in `pimdir queue list` and Himalaya. It mirrors calendula's `pimdir status`, `calendars` named `addressbooks`. It is no alias of `pimdir queue list`: it reports the store (its accounts, each address book's cards and how many are downloaded), which no queue command carries, where the queue commands list and retract the rows themselves.

### Requirement: The pimdir queue is read and retracted
`pimdir queue list`, `pimdir queue show <ROW>` and `pimdir queue cancel <ROW>`, gated on the `pimdir` feature, SHALL read and retract the account's queue as Himalaya's `pimdir queue` does. `list` SHALL print the card creations staged in the account's address books, or in the one `-k` names (`addressbook-not-found` for one the account does not hold), then every row of any kind the owner parked there, each as the family's queue row (pimalaya-cli's output.md, "A queue row keeps one shape") plus `fullName`: `{"cards": [<row>], "parked": [<row>]}`, a row `{"row", "kind", "collection", "linkId", "itemId", "producer", "queuedAt", "attempts", "unconfirmed", "error", "fullName"}`, every key printed, `null` or not. `kind` is the action kind io-pimdir reads from the row, never re-spelled; `linkId` the link id of the card the row creates, the one `card add` reported; `itemId` the id `card list` prints of the card the row addresses, `null` for a creation; `attempts` how many times the sync engine tried the row; `unconfirmed` whether its outcome is unknown; `error` `null` while the row is pending, the store's text once it is parked; `fullName` the full name of the body the row pins, else of the card it addresses, `null` when neither is local. A parked row's card is read from its payload: one that does not decode, and an intent or a `set-performer`, name none. The text prints them under `Account:` and `Addressbook:` with the cancelling command. `show` SHALL print where one row stands, `{"row", "state", "collection", "kind", "attempts", "error", "appliedAt", "seq"}`, `state` one of `pending`, `parked`, `unconfirmed`, `applied`, `unknown` and `reset`, `seq` the card an applied add created, the id `card read` takes; a row anchored on a collection that is not one of the account's address books SHALL read `unknown`. `cancel` SHALL ask for confirmation unless `-y`, refuse a row applied already or unknown to the account before the store is opened for writing, then take the store owner's role for that one write (io-pimdir's `PimdirStore::open_current` and `drop_action`), refused at once coded `store-owned` while a sync holds it, and print `{"row", "kind", "unconfirmed"}`. It is the one write cardamum makes as the store's owner, and the only way back for a queued creation, which has no id a `card delete` could take.

#### Scenario: A queued card is cancelled
- GIVEN a pimdir address book `book` a source syncs, and `cardamum card add -k book` queued a card as row 3
- WHEN `cardamum --json pimdir queue list` runs, then `cardamum --json pimdir queue cancel -y 3`
- THEN the listing holds `{"row":3,"kind":"add","collection":"book","linkId":"<UID>","itemId":null,…,"attempts":0,"unconfirmed":false,"error":null,"fullName":"<FN>"}` under `cards`, the cancel prints `{"row":3,"kind":"add","unconfirmed":false}`, and `pimdir queue show 3` then reads `unknown`

#### Scenario: A parked update
- GIVEN the card `7` of the pimdir address book `book`, whose `card update` a sync parked
- WHEN `cardamum --json pimdir queue list` runs
- THEN its row under `parked` reads `"kind":"update"`, `"linkId":null`, `"itemId":"7"`, the store's text as `error` and the full name of the body the update pins as `fullName`

### Requirement: account check fails when a backend fails
`account check` SHALL exercise every backend of the account `--backend` allows and print one verdict per backend, a backend the build leaves out failing coded `backend-not-built` under `auto`, under `--json` `ok`, the stable `code` and the error of a failed one (`code` `null` when the backend passed or its failure carries none), as the failure object of any other command names them. When any fails, it SHALL still print the report and exit 3, the family's partial outcome (`pimalaya_cli::error::PartialOutcome`), so a script reads the outcome from the exit code alone and which backends passed from the report. It mirrors Himalaya's `account check`.

### Requirement: The account outputs are the family's
`account list` SHALL print pimalaya-cli's `account::AccountListOutput` and `account check` its `account::AccountCheckOutput`, each row coded by cardamum's vocabulary, so both read as in Himalaya and Calendula. Their JSON keeps its shape: `{"accounts": [{"name", "default", "backends", "notBuilt"}]}` and `{"account", "backends": [{"backend", "ok", "code", "error"}]}`. The text of `account list` SHALL be one line per account with no header row: the name, its backends in one cell (the built ones, then those the build leaves out, each marked `(not built)`), and `default` on the default account. The text of `account check` SHALL be `Account: <name>`, then one `<backend>: OK` or `<backend>: FAIL (<error>)` line per backend, the name and the error defused.

#### Scenario: Two accounts
- GIVEN the accounts `home` over vdir and `work` over CardDAV, the default
- WHEN `cardamum account list` runs
- THEN it prints `home  vdir     -` then `work  carddav  default`, with no header line

### Requirement: Specific commands are raw-faithful
A specific command SHALL surface the native ids and metadata the shared layer normalizes away (ETags, CTags, sync tokens, hrefs, the Graph `changeKey`, the People `etag` and `resourceName`), and SHALL make the incremental-sync primitive its headline (`report sync`, `changes`, `delta`). Default human output is a friendly table; `--json` SHALL emit the raw native payload, the provider's paging cursor alone lifted into `nextPage` ("Protocol listings page by nextPage"), and create and update SHALL accept the raw native representation. Each backend keeps a raw escape hatch: a raw XML body for CardDAV, raw JSON in and out plus a generic request passthrough for the object backends, whose `--json` prints the response verbatim under `response` (`null` when the server sent no body), so the payload is an object whatever the server answered. Precondition control is offered only where the protocol has it (CardDAV `--if-match` and `--if-none-match`).

### Requirement: Specific commands take their ids positionally
A protocol-specific command SHALL take the ids it operates on as positional arguments, repeatable where the protocol accepts several, and reserve flags for the rest, mirroring Himalaya. A short option SHALL NOT be given to an argument whose letter the global options already use: clap asserts on the collision in a debug build and silently makes the letter ambiguous in a release one, so the command never runs either way.

### Requirement: A command prints what the server returned
A command SHALL render only the fields the server's response actually carries, and SHALL report success in words when the response carries nothing to show, rather than printing empty field slots. This matters most on the write path, where a JMAP `set` response echoes only the properties the server itself decided, and where a REST delete answers 204 with no body at all: text output SHALL then print nothing. `--json` still emits the raw native payload.

A protocol view SHALL print the fields an item holds, no fewer and no empty one: `carddav propfind` an address book's name, description, colour, CTag and sync token, `vdir list` a collection's name, description and colour beside its path, a property the item lacks printing no line; `msgraph contacts` a contact's name, then every email and every phone it holds, the mobile, business and home ones, each kind on one line.

#### Scenario: An address book with a description and no name
- GIVEN a CardDAV address book whose PROPFIND carries a description and a CTag and no display name nor colour
- WHEN `cardamum carddav propfind` runs
- THEN its record prints `Description:` and `CTag:` lines and no `Name:` nor `Color:` line

#### Scenario: A contact with a business phone alone
- GIVEN a Graph contact whose `businessPhones` holds one number and which has no mobile phone nor email
- WHEN `cardamum msgraph contacts get` prints it
- THEN it prints that number on a `Phone:` line and no `Email:` line

### Requirement: An incremental-sync command can resume
A protocol-specific command exposing its backend's incremental sync SHALL accept the resume token that sync hands back, in whatever shape the protocol uses (a Graph `@odata.deltaLink` URL, a People sync token, a JMAP or DAV state string). A command that documents a resume SHALL provide the flag for it.

### Requirement: An empty addressbook id is rejected
The shared `-k/--addressbook` resolver SHALL reject an empty id rather than pass it to a backend, where it silently addresses the collection instead of a member. The refusal SHALL be coded `addressbook-not-found`, an empty id naming no address book.

### Requirement: A collection command takes its address book as a positional
`addressbook delete` SHALL take the address book it deletes as a required positional, `addressbook delete <ADDRESSBOOK>`, and `addressbook update` as an optional one, `addressbook update [ADDRESSBOOK]`, falling back to `addressbook.default` as the card commands do, as Himalaya's `mailbox delete <MAILBOX>` takes its mailbox; `addressbook create <NAME>` takes its name so already. `-k/--addressbook`, the v0.3 spelling, SHALL stay accepted on both as a hidden alias of the positional: a second argument conflicting with it, so a line giving both is refused `usage-invalid`, and clap waives the required positional when that option is present. The help, the manual and the MCP input schema SHALL show the positional alone, the tool property named `addressbook` as the option named it, required on `addressbook_delete`. `--color` keeps its short letter `-C` on `addressbook create` and `addressbook update`.

### Requirement: The collection arguments are named as the twin's
`vdir create` SHALL take the collection's directory name as `<ID>` and its display name as `-n/--display-name <NAME>`, written to the `displayname` file, the id written there when the flag is omitted; `vdir rename` SHALL take `<OLD-ID> <NEW-ID>` and `vdir delete` `<ID>`, as calendula's `vdir` commands name them. `addressbook update -n/--name` SHALL take a `<NAME>`, as calendula's `calendar update` does.

#### Scenario: A display name apart from the id
- GIVEN a vdir account
- WHEN `cardamum --json vdir create friends -n Friends` runs
- THEN it prints `{"id":"friends"}`, makes the directory `friends` and writes `Friends` to its `displayname` file, where `cardamum vdir create bare` writes `bare`

#### Scenario: The v0.3 option still deletes
- GIVEN a vdir account holding the address books `personal` and `work`
- WHEN `cardamum --json addressbook delete work` runs, or `cardamum --json addressbook delete -k work`
- THEN either deletes `work` alone and prints `{"id":"work","queued":false}`; `cardamum --json addressbook delete work -k personal` is refused `usage-invalid`, exit 2, deleting nothing

### Requirement: A card is added
The unified verb creating a card SHALL be `card add`, as Himalaya's `message add`, its MCP tool `card_add` and its schema `cardamum-card-add`. `card create`, the v0.3 name, SHALL stay accepted as a hidden alias, as SHALL `card new`. The capability it reaches is `card.add`, pimdir's `contacts.card.add`.

### Requirement: A partial write says so
When a backend cannot carry out part of a write, and the protocol offers no way to insist, the command SHALL report what was left undone rather than print a plain success. On Google People an unmappable vCard property lives in a `clientData` entry the API will not clear, so an update dropping one keeps it, and the command names it.

### Requirement: A queued write says it is queued
A unified write SHALL say whether it reached the server or was queued for a sync. On pimdir, `card add`, `card update` and `card delete` SHALL say the write is queued and that the next sync carries it, never that the card was created, updated or deleted. Under `--json`, every unified write SHALL print `queued`, the family's receipt key (pimalaya-cli output.md, "A write answers a receipt"), on every backend: `true` when a sync engine applies the write later (pimdir), `false` when it reached the server. `card add` prints `{id, queued}`, `card update` `{id, keptProperties, queued}`, `card delete`, `addressbook update` and `addressbook delete` `{id, queued}`, `addressbook create` `{id, name, queued}`, `name` the display name it was given, as Himalaya's `mailbox create` prints it.

#### Scenario: A card queued on pimdir
- GIVEN a pimdir address book a source syncs
- WHEN `cardamum --json card add -k <book> <vcard>` runs
- THEN it prints `{"id":"<link id>","queued":true}`, and the text form says the next sync creates the card

A queued creation SHALL get no placeholder id (`q<id>`, `0`, `""`): `card add` reports the card's link id, which `card read` does not accept until a sync applies the create and the card lists under its public id.

### Requirement: A displayed count is the server's
A text output SHALL NOT render an absent value as a default that reads as fact. A count the API returns only on request SHALL be requested, or left blank.

### Requirement: An address book says whether it is the default
`addressbook list` SHALL print `default` on each address book, `true` on the one the backend states a card goes to when none is named: JMAP's `isDefault` (RFC 9610 §2), the Graph default Contacts folder (`contacts`), People's myContacts, the pimdir store's role (STORAGE §14); `false` on the others, and on every CardDAV and vdir address book, which state none. It SHALL NOT read `addressbook.default`, which stays the configuration's.

#### Scenario: The default JMAP address book
- GIVEN a JMAP account whose `AddressBook/get` answers `RBk` with `isDefault: true`
- WHEN `cardamum --json addressbook list` runs
- THEN `RBk` carries `"default":true`, as `jmap address-book get` shows `isDefault`

### Requirement: A command resolves its addressbook before acting
A unified command SHALL resolve the addressbook it was given before reading or writing through it, and SHALL fail naming it when it does not exist. Listing an unknown addressbook SHALL NOT read as empty, and writing into one SHALL NOT create it.

### Requirement: A rejected write is legible
A rejected protocol write SHALL surface as prose: the server's own error type and description, plus the properties it named, never a Rust `Debug` rendering of the error value. Where the rejection has a known cause the user can act on, the message SHALL name it, as the CardDAV missing-UID hint and the JMAP `vCardProps` hint do.

### Requirement: Output streams
All output, data and errors alike, SHALL go to stdout through the printer, distinguished by the exit code; `--json` switches every command to JSON. stderr carries logs and interactive prompts only, so redirecting stdout is always safe.

### Requirement: Exit statuses
The binary SHALL exit 0 on success, 1 with the failure object printed, 2 on a command line the grammar refuses (an unknown flag or command, a missing argument, a bad value), and 3 on a partial outcome, a command that printed its result without fully succeeding (`pimalaya_cli::error::PartialOutcome`): `account check` with a failed backend, and a listing, `pimdir search` or a CardDAV `report query` or `report multiget` whose `errors` is not empty. The binary SHALL parse its command line through `pimalaya_cli::clap::parse`: help and version print as clap prints them; a refused command line with a `--json` word before any `--` SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without the `error: ` prefix nor styles, and without `--json` clap's rendering on stderr.

#### Scenario: A refused command line under JSON
- GIVEN any configuration, or none
- WHEN `cardamum --json card list --nope` runs
- THEN it exits 2, prints nothing on stderr and prints `{"code":"usage-invalid","error":"unexpected argument '--nope' found…","sources":[],"backtrace":null}`

### Requirement: A data command answers a named output type
A command returning data SHALL hand the printer a dedicated type named `<Domain><Target><Verb>Output`, deriving `Display`, `Serialize` and `JsonSchema`, with every public field documented. A write with no payload of its own SHALL print a receipt rather than a sentence: a unified write `{id, queued}` (`WriteReceipt`), the id of the item acted on and whether a sync engine applies the write later, `addressbook create` `{id, name, queued}`, `card add` `{id, queued}`, `card update` `{id, keptProperties, queued}`, a write in a protocol namespace `{id}` (`IdReceipt`) or `{ids}` (`IdsReceipt`), the ids as they are after the write (a renamed collection's new one), and a `card build -o` `{path, size}` (pimalaya-cli's `output::SavedOutput`). `keptProperties` stays on `card update`: it names what a People update dropped and the server kept ("A partial write says so"), a result calendula's `event update` and `item update` print no counterpart of, so the twins align on `{id, queued}` and cardamum adds this one key. The sentence is the text form only; data a confirmation used to word (an ETag, member counts) is a key of its own.

Every runnable command SHALL be described in the `json-schema` registry, a command printing one of two types through both (`OneOf`), which a test checks over the clap command tree.

A command whose two shapes are one invocation SHALL answer one type covering both, untagged so each shape serializes exactly as it would alone.

### Requirement: JSON output keys are camelCase
Every output type SHALL carry `serde(rename_all = "camelCase")`, so a `--json` key spelling more than one word reads `addressbookId` rather than `addressbook_id`. That is the convention of the wire formats the backends speak, and unlike a hyphenated key it needs no quoting in a `jq` path. The configuration vocabulary is a separate one and stays kebab-case.

A field renamed onto a provider's own wire key SHALL keep that spelling, a container attribute never reaching a field carrying its own `rename`: `@odata.deltaLink`, `nextSyncToken`, `contactGroups` and the JMAP `list` are the provider quoted back to the caller, not ours to restyle. A provider's paging cursor is the one key not quoted back: it prints as `nextPage` ("Protocol listings page by nextPage").

### Requirement: The JSON output of every data command has a schema
Every output type SHALL be registered in [json_schema.rs](../../src/json_schema.rs) under its CLI invocation path, hyphen-joined and prefixed `cardamum-`, and the `json-schema` command (aliased `json-schemas`) SHALL print one or write one file per entry. A registered key naming no command is a test failure, so the registry cannot drift from the tree; `cardamum-error`, the failure object every command prints, is the one key naming none.

A wire object the protocol crate exposes without a `JsonSchema` of its own SHALL be described as a JSON object of any shape rather than left out: the payload is the provider's to define, and claiming a shape we do not own would be worse than claiming none.

### Requirement: A card is edited through a command, never through a pipe
A composer SHALL be a command the configuration names, spawned on the path of a temporary file holding the vCard, with stdin, stdout and stderr all inherited. Cardamum SHALL capture none of the command's streams: a composer that spawns an editor would otherwise hand it a pipe instead of the terminal, and the editor hangs or writes where nothing reads. The command edits the file in place, and cardamum reads it back once the command exits.

The path SHALL be appended as the command's last argument. For a shell line that means interpolating it into the line, single-quoted, since a shell invoked as `sh -c <line> <path>` binds the path to `$0` rather than passing it on.

### Requirement: The composer's own exit is the decision
When the composer exits, what it left in the file SHALL settle the edit, and nothing SHALL be asked after it. A file that came back changed is the card, and the command writes it. A file the composer emptied, and one handed back byte for byte as it was given, are an edit given up on: nothing is written, and nothing failed. A non-zero exit status is a failure and SHALL be reported as one.

Those three outcomes SHALL be the whole protocol, so a composer owning its own save and discard is not second-guessed by a menu it cannot see, and a plain editor keeps the meaning its own quit already has: a graphical editor says no by emptying the file or by exiting non-zero, and `nvim` says it with `:q!`.

Abandoning SHALL be the same on a create and on an update, and SHALL exit as a success: the card is written or it is not, and which verb asked is already on the command line. `card build` SHALL print nothing when abandoned, the two write verbs SHALL say that nothing was written.

### Requirement: A composed card is checked before it is written
What the composer wrote SHALL be checked against its own version.s RFC contract, through vcard-rs`s validator rather than a look at its first line. Reading a card is liberal and this is the strict half: a 4.0 card missing its required `FN`, a 2.1 one missing its required `N`, a property the version does not define, are all caught here rather than by the server or by nobody.

A card that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes. This is the one thing a composer cannot be trusted with, a plain editor happily handing back a card missing its `FN`, and it is not a menu: the only question is whether to fix it, and declining is an error rather than an abandon.

A card built from flags with no source SHALL be checked the same way and refused, naming the violations, whether it was `card add` or `card build` that built it. A vCard given on the command line SHALL NOT be, going to the backend as it was written: that is the promise the protocol-specific commands already make, and second-guessing a card someone handed over is not cardamum.s to make.

### Requirement: An edit is never lost
When a composed card cannot be written, whether the check was declined or the backend rejected it, the temporary file SHALL be kept and the error SHALL name it: `Cannot edit vCard <path>`, which says what failed and where the work is in one line, the path being the recovery. A person who spent a minute in an editor never loses it to a failure that happened afterwards.

An abandoned edit SHALL drop the file instead: an emptied one holds nothing, and an untouched one holds only what cardamum put there, so neither is anything to lose, and removing them is what keeps the temporary directory clear of every abandoned run.

### Requirement: A composer is not spawned under --json
`--json` SHALL refuse to run a composer rather than spawn one. The child inherits cardamum's own stdout, where it would interleave with the JSON payload, and a consumer parsing that output has no terminal to edit in.

### Requirement: A source, the field flags and the composer stack
`card add` and `card update` SHALL take a vCard source, the field flags and `-i/--interactive` together, applied in that order: the source is the card to start from, each flag sets the property it names on it, and `-i` opens the result in the composer. Every combination therefore falls out of one pipeline, which `card build` stops one step early rather than restating: no verb beside these two writes a card.

The composer SHALL be opt-in through `-i`, never opt-out. A command with no `-i` spawns nothing, which is what keeps both verbs scriptable, and `-i` SHALL bail when no composer is configured rather than fall back to one. `--composer <COMMAND>` SHALL override the configured one for a single invocation, and SHALL require `-i`. The pair SHALL be one shared argument, spelled once for the three commands that take it.

A create given no source SHALL mint the card it starts from rather than open an empty file, so that no composer is asked to invent an identity: it carries a `UID` and a `VERSION` and no more. An update given no source SHALL start from the card the backend holds.

A command given no source, no field flag and no `-i` has nothing to write and SHALL say so, rather than send an unchanged card back to the server.

A card built from flags SHALL be built unchecked. A draft on its way to an editor is not yet a card, and refusing to open one because `FN` is missing is the opposite of what a composer is for.

### Requirement: A card can be built without an account
`card build` SHALL take the same vCard source, the same field flags and the same composer as `card add`, apply them in the same order, and print the resulting vCard rather than sending it anywhere. It SHALL reach no backend.

It SHALL read no configuration, unless `-i` is given without `--composer` and the configured composer is the one thing it needs. A vCard can therefore be formatted on a machine holding no configuration at all, and the account a card subcommand runs against SHALL be resolved by that subcommand rather than ahead of the whole `card` family.

It SHALL check what `card add` would check: a card built from flags with no source is validated and refused, a vCard given as a source passes through untouched. A weaker rule would make `card build ... | card add -` a way past the guard, the piped card arriving at `add` as a source, which `add` does not check.

A build given no source, no field flag and no `-i` has nothing to build and SHALL say so.

### Requirement: A build can be composed, and captured
`card build` SHALL take `-i/--interactive` and `--composer <COMMAND>`, the same pair the two write verbs carry, and SHALL print what the composer wrote rather than sending it anywhere. This is where a card is judged before it is written: the menu that used to offer a look is gone, and a build is the look.

`-o/--output <PATH>` SHALL write the card to that file instead of printing it, answering pimalaya-cli's `output::SavedOutput` rather than the card: `{path, size}` under `--json`, `Saved <size> bytes to <path>` in text. The flag earns its place on this command alone: the composer inherits stdout, so `card build -i > card.vcf` hands the editor the file as its terminal, and with `-i` there is no redirection to fall back on.

An abandoned interactive build SHALL print nothing at all, and exit as a success. The output of this command is a vCard someone pipes onwards, and a line saying no card was built is not one.

### Requirement: A source carries a card or it is refused
A source that holds nothing but whitespace SHALL be refused, naming where it was read from, rather than read as a card. Handing a backend an empty body is never what was meant, and an abandoned `card build -i` prints nothing, which pipes straight into the next command.

A source holding several vCards SHALL be refused when a field flag is set. A flag rewrites the card the parser reads first, so the cards behind it would be dropped without a word. With no flag the source passes through as it was written, several cards included, which is the promise every other source keeps.

### Requirement: A minted card carries an identity
A card minted from nothing SHALL carry a `UID`, minted as a fresh `urn:uuid` when `--uid` names none. The pimdir link id derives from it, so a card created without one is unaddressable in the store it lands in, and a composer handed an empty file mints none: tCard seeds an identity for a blank template but preserves what a file already holds, and a graphical editor may do neither.

### Requirement: The field flags are a convenience, not the surface
The field flags SHALL cover the fields `card list` renders and the few every address book carries, and SHALL NOT grow to match what a composer models. `ADR` is out: seven components is where a flag stops being ergonomic. The documentation SHALL say the composer is the complete surface, so the gap reads as the boundary it is rather than as a missing feature.

### Requirement: A field flag sets its property and touches nothing else
Writing a field flag SHALL drop every instance of the property it names and write the flag's own, so a flag sets that property rather than adding to it, and a repeated flag writes one instance per value. Every other line SHALL keep the card's own bytes, the properties no flag covers included.

`N` is the exception, being one property holding several names: `--given-name` and `--family-name` SHALL merge into the card's existing `N` rather than replace it, so setting one does not silently clear the other. All five components of `N` SHALL have a flag, and each SHALL repeat, a component being a comma-separated list (RFC 6350 section 6.2.2).

The name flags SHALL be named for the role, given and family, rather than for the position, first and last. Which of the two is written first is exactly what varies between cultures, which is why RFC 6350, RFC 9553 and CLDR all name the role.

### Requirement: An update guards on the version it read
`card update` given no vCard source SHALL guard the write with the version of the card it read, an explicit `--if-match` still winning, on a backend declaring `card.update.if-match` at a level above `none` (carddav, gpeople, pimdir, jmap, msgraph). Having read the card to change it, the command knows the version it diverged from, and an edit that took a minute SHALL NOT silently overwrite a write that landed during it. A source given on the command line is a rewrite that was asked for, and carries no such guard of its own.

CardDAV and People send the version with the write (`If-Match`, the People `etag`) and pimdir checks it before queueing. Microsoft Graph and JMAP take no guard on a contact write, so they SHALL compare it themselves, as calendula guards a Graph event, at no request of their own on an update: Graph with the `changeKey` of the contact the update reads as its delta base, JMAP with the BLAKE3 digest of the JSON of the ContactCard the update reads as its patch base (the version `card list` and `card read` report). A version that differs, surrounding double quotes ignored, SHALL fail `precondition-failed` before the write, ``Card `<id>` changed on <server> since it was read``; a write landing between that read and the write is not caught, which their `partial` level says. vdir, declaring `none`, reports no version on a read and SHALL refuse an explicit `--if-match` with `capability-missing` ("An unsupported precondition is refused, not dropped").

#### Scenario: An update on Microsoft Graph without --if-match
- GIVEN a Microsoft Graph account holding the contact `AAMk` in its default Contacts folder, read with the `changeKey` `k1`
- WHEN `cardamum --json card update -k contacts AAMk --full-name "Ada Byron"` runs, with no `--if-match`
- THEN it sends one `PATCH` carrying no `If-Match`, prints `{"id":"AAMk","keptProperties":[],"queued":false}` and exits 0; the same command with `--if-match k0` fails `precondition-failed` with no `PATCH` sent

### Requirement: A delete can be gated on a version
`card delete` SHALL take `--if-match <ETAG>`. CardDAV SHALL send it as `If-Match`, pimdir SHALL check it as an update does, and Graph and JMAP SHALL read the card first, only when `--if-match` is given, and compare its `changeKey` or its hash as an update does, failing `precondition-failed` before the delete. The backends that cannot gate a delete (vdir, gpeople) SHALL refuse it rather than drop it.

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`, and in an `errors` entry beside `id`, `error` and `sources`. The code SHALL be found anywhere in the error chain, so added context does not hide it, or read from the typed error of the protocol that refused ("A server's refusal carries its code"). A failure with no code SHALL print `"code": null`. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. A pimdir store's code is the one io-pimdir names (`PimdirError::code()`), never read off its message. Every code SHALL be declared in every build, whatever the backends compiled in.

| Code | Raised when |
|---|---|
| `usage-invalid` | the command line, a REPL line or an MCP tool call's arguments do not match the grammar (pimalaya-cli's `CommonCode::UsageInvalid`); the command line exits 2 |
| `account-missing` | no account has the name `-a` gives, `default` included (pimalaya-config's `AccountError::Missing`) |
| `account-default-missing` | no `-a` names an account and none is marked `default = true` (`AccountError::NoDefault`) |
| `account-default-several` | no `-a` names an account and several are marked `default = true` (`AccountError::SeveralDefaults`) |
| `backend-not-built` | the backend the account resolves to is configured but left out of the build; the error names the cargo feature (see backends.md, "A backend left out of the build is named") |
| `stdin-taken` | a command inside a session (`repl`, `mcp`) would read its input off stdin, which carries the session's commands |
| `capability-missing` | the backend the account resolves to does not serve a capability the command reaches; nothing was read or written |
| `card-not-found` | the card a command names is not in the address book: the failure of `card read`, `card update` or `card delete`, worded by cardamum ("A not-found failure is cardamum's sentence"); an `errors` entry of `carddav report multiget`; a protocol command naming one card (`carddav get` and `delete`, `msgraph contacts get`, `update` and `delete`, `gpeople connections get`, `update` and `delete`), the server's message kept; a card id no CardDAV card can have, before anything connects (backends, "A CardDAV id addresses its own resource") |
| `addressbook-not-found` | the address book a command names does not exist, or no address book can have its id (empty, not one vdir directory name, refused by io-webdav on CardDAV); a protocol command naming one Graph contact folder Graph does not hold (`msgraph contact-folders get`, `child-folders`, `rename` and `delete`, `msgraph contacts list`, `create` and `delta` under `-f`), Graph's message kept; nothing was read or written |
| `addressbook-default-missing` | a command needing an address book got no `-k` and the account sets no `addressbook.default`; nothing was read or written |
| `addressbook-duplicate` | `addressbook create` or `vdir create` names an address book the backend holds already (backends, "A collection created twice is refused"): a vdir directory, a CardDAV collection the home set lists, a Graph contact folder, a People contact group; nothing was created |
| `permission-denied` | the server refused the request to the account (see "A server's refusal carries its code") |
| `quota-exceeded` | the write would exceed the account's quota |
| `rate-limited` | the server throttled the request; cardamum retries nothing |
| `card-unreadable` | the backend answered for a card cardamum could not read: its listing, `report query` or `report multiget` row came without its data (CardDAV), its file could not be read (vdir), its body did not parse as a vCard or is not UTF-8, it did not project (JMAP), its body could not be read or is gone from the blob tree (pimdir); an `errors` entry |
| `card-invalid` | the server refused the card written as no vCard it takes (CardDAV, "A server's refusal carries its code"); nothing was written, and the card is what to fix, not the account's rights |
| `body-pending` | a listed card's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the card no longer has: pimdir's check, a CardDAV server answering 412, or the Graph `changeKey` or JMAP digest read before the write |
| `store-uncreated` | the pimdir root holds no store; its sync engine creates it |
| `store-version` | the pimdir store is of another format version than this cardamum reads; an earlier neverest's store is reinstalled through neverest, its queue drained first, a later one's asks for a cardamum upgrade |
| `store-foreign` | the pimdir root holds a database another program wrote |
| `store-corrupt` | the pimdir store's two version stamps disagree, or its schema differs from the one its version declares |
| `store-not-current` | the pimdir store holds no schema yet and the verb creates none; its sync engine creates it |
| `store-hash-algo` | the pimdir store names its bodies by a hash this cardamum does not know |
| `sync-version` | the pimdir store's sync state was written by another version of its sync engine |
| `store-exposed` | another user can read or write the pimdir store, or a link stands where a lock file belongs; restrict it |
| `store-busy` | another writer held the pimdir store, past the wait on a write and at once on `account check`, which does not wait; retry |
| `store-owned` | a sync owns the pimdir store; retry once it ends |
| `store-staging` | a producer is staging a body in the pimdir store; retry |
| `store-in-flight` | a pimdir store verb is between two of its chunks; retry |
| `store-reset` | the pimdir store is not the one that answered a queue row's handle, a copy restored since; nothing was done with the row |
| `lock-replaced` | the pimdir store's staging lock file was replaced while a write held it; nothing was queued, retry once no other process sets the store up |
| `unsupported` | a source syncing the address book does not support the write (pimdir STORAGE §15.6); nothing was queued |
| `item-duplicate` | a create names a `UID` the address book holds: on pimdir synced or queued, nothing queued; on CardDAV the server's `no-uid-conflict` (RFC 6352 §6.3.2.1) |
| `flag-invalid` | the pimdir store cannot hold a flag; nothing was staged |
| `action-malformed` | a queued pimdir action's payload is malformed (`pimdir status`); the sync engine's drain parks it |
| `conflict-unread` | a pimdir conflict holds a body nobody has read; fetch it first |
| `no-performer` | no source of the pimdir store can perform the intent |
| `ambiguous-performer` | several sources of the pimdir store can perform the intent and none is chosen |
| `index-missing` | the pimdir store has no search index yet (`pimdir search`, `pimdir count`); its sync engine builds it |
| `index-outdated` | the pimdir search index was built otherwise or from another store; its sync engine rebuilds it |
| `index-version` | the pimdir search index is of another draft or shape; its sync engine rebuilds one of an earlier draft and leaves any other |
| `index-zone-unknown` | the zone the pimdir search index reads floating times in is not an IANA zone |
| `index-fts5-missing` | the SQLite cardamum links lacks FTS5 or predates 3.43, which the pimdir search index needs |
| `query-refused` | the pimdir search query is malformed or names an unknown field or value (SEARCH §8.4); nothing was searched |
| `query-unanswerable` | the pimdir search index could not answer an accepted query, an implementation fault |
| `tag-refused` | a pimdir tag names no flag, or adds and removes one flag; nothing was staged |

#### Scenario: A stale version
- GIVEN a pimdir card whose body changed since it was read
- WHEN `cardamum --json card delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

#### Scenario: An unknown card
- GIVEN a vdir address book `book` holding no card `nope`
- WHEN `cardamum --json card read -k book nope` runs
- THEN the command exits 1 and prints a failure object coded `card-not-found`, its `error` ``Card `nope` not found in address book `book` `` and io-vdir's text in `sources`

### Requirement: A not-found failure is cardamum's sentence
A card a command names that the address book does not hold SHALL fail with cardamum's own sentence on every backend, ``Card `<id>` not found in address book `<book>` ``, as Himalaya words a message not found (``Message `<id>` not found in mailbox `<mailbox>` ``), the backend's answer below it in `sources` (io-vdir's, the CardDAV, Graph or People refusal, the JMAP `notFound`; pimdir has no text of its own to add): the backend's wording never stands as the error. An item error SHALL NOT repeat its card's id in `error`, the id being its own key.

### Requirement: A command with no address book is coded
A card command given no `-k/--addressbook`, and `addressbook update` given no address book, over an account setting no `addressbook.default` SHALL fail coded `addressbook-default-missing`, exit 1, the error naming the flag and the key: ``No default addressbook found, name one with `-k <ADDRESSBOOK-ID>` or set `addressbook.default` ``, as calendula refuses a missing default calendar (`calendar-default-missing`) and the family a missing default account (`account-default-missing`).

### Requirement: The JSON contract is described and versioned
`json-schema` SHALL describe the failure object under the key `cardamum-error`, its `code` typed by the code vocabulary and pimalaya-cli's `usage-invalid`, and the item error of an `errors` list under `cardamum-item-error`, its `code` typed by the code vocabulary. Every generated schema SHALL carry an `$id` naming the contract version: the major version of cardamum, or `0.<minor>` before 1.0. Every registered payload SHALL be a JSON object, checked by a test: a command whose shapes are one untagged type answers an empty object for a shape carrying nothing (an abandoned `card add -i`), never `null`. Piped `--json` output SHALL print one value per line.

### Requirement: A card projects its common fields
`card list` rows and the `card read` output SHALL carry the card's common fields, read through vcard-rs's decoded model rather than off raw lines: `uid`, `fullName`, `givenName` and `familyName` (the `N` components, space-joined), `emails` and `phones` (every `EMAIL` and `TEL` in document order, a `tel:` URI as its number), `organization` and `organizationUnits` (the `ORG` components), `title` and `note`. A card that does not parse, or whose bytes are not UTF-8, SHALL project nothing rather than fail the listing, never a lossy decode. The projection is read-only: the vCard stays the record.

The `fnValue`, `email` and `tel` keys, and the name, email and phone columns of the `card list` text, SHALL be the first `FN`, `EMAIL` and `TEL` values as written, escapes kept and a `tel:` URI whole, read off the card's lines once unfolded (RFC 6350 §3.2), a property matched by its whole name, case-insensitive, whatever its group and its parameters (§3.3), its value starting at the first colon outside a quoted parameter value.

#### Scenario: A derived name and a folded address
- GIVEN a card holding `FN;DERIVED=TRUE:Jane Doe` and an `EMAIL;TYPE=home:jane@examp` line folded before `le.org`
- WHEN `cardamum --json card list` lists it
- THEN its `fnValue` is `Jane Doe` and its `email` `jane@example.org`, the text naming her

### Requirement: A card read prints its listing row and its vCard
`card read` SHALL print the row `card list` prints for the card, every key with the same value (`id`, `addressbookId`, `etag`, `fnValue`, `email`, `tel` and the common fields), `etag` `null` where the backend versions no card, plus what only a read carries, the raw vCard, as pimalaya-cli's output.md says a read prints ("Collections are named and present").

The vCard SHALL travel as `contents`, a string: the text when its bytes are valid UTF-8, `encoding` then `utf-8`, and their base64 (RFC 4648 standard alphabet, padded) when they are not, `encoding` then `base64`, so a script reads `encoding` and never takes base64 for a vCard (output.md, "Bytes are never raw under JSON"), as calendula prints an item's bytes. The bytes SHALL NOT be decoded lossily under `--json`, and a card that is not UTF-8 SHALL read rather than fail. The text form prints the card as it does, defused ("Plain output is printable"). `card build` SHALL carry its card the same way, `{contents, encoding}`, and in text SHALL print a card that is not UTF-8 as its bytes, which a pipe takes on unchanged.

#### Scenario: A card read
- GIVEN a vdir card `ada` whose `FN` is `Ada Lovelace`
- WHEN `cardamum --json card read -k personal ada` runs
- THEN it prints the keys its `card list` row prints with the same values, `"etag":null`, `"contents":"BEGIN:VCARD…"` and `"encoding":"utf-8"`

#### Scenario: A card that is not UTF-8
- GIVEN a card whose bytes hold a Latin-1 `é`
- WHEN `cardamum --json card read` reads it
- THEN it exits 0 and prints `"contents"` holding its bytes in base64 and `"encoding":"base64"`, its common fields `null`

### Requirement: A CardDAV read carries its vCard beside its encoding
`carddav get`, `carddav report query` and `carddav report multiget` SHALL carry each vCard as `card read` does ("A card read prints its listing row and its vCard") and as calendula's `caldav get`, `report query` and `report multiget` carry an item: `contents` beside `encoding`, the text when the bytes are valid UTF-8, `encoding` then `utf-8`, and their base64 (RFC 4648 standard alphabet, padded) otherwise, `encoding` then `base64` (pimalaya-cli output.md, "Bytes are never raw under JSON"). A card that is not UTF-8 SHALL read rather than fail, and the `carddav get` text SHALL print it as its bytes, as `card build` hands one over. A REPORT row's card is the `address-data` text io-webdav reads, so its `encoding` is `utf-8`.

#### Scenario: A CardDAV card that is not UTF-8
- GIVEN a CardDAV server answering the GET of `c1.vcf` with a vCard holding the byte `0xE9`
- WHEN `cardamum --json carddav get book c1.vcf` runs
- THEN it exits 0 and prints `id`, `etag`, `contents` holding the bytes in base64 and `"encoding":"base64"`

### Requirement: pimdir searches cards through the store's index
`cardamum pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (search-draft-11 §8) from the store's search index, scoped to the account's cards: the query is read as its conjuncts plus `kind:contact`, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--addressbook` names an address book. The scope is added beside the query, never around it, so a group that is itself a conjunct of the query keeps narrowing the answer and its coverage as its terms do bare (SEARCH §8.2). Without `-k` it SHALL search every address book of the account, `addressbook.default` not applied, and a `-k` naming no address book of the account SHALL fail naming those it holds rather than answer nothing. The words of the query are joined by spaces into one query. An option of `pimdir search` or `pimdir count` SHALL be read wherever it stands, before the query or after it, `--json` included; a query word starting with `-`, a negated term, SHALL pass after `--` alone, a word starting with `-` before it being refused as an unknown option or read as the option it spells, as calendula and Himalaya read theirs. Cardamum SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

Each hit SHALL carry the keys `card list` prints for a card (its `id` the public `seq`, which `card read` takes), its `addressbookId`, what a text term matched (`body`, `summary`, or `null` when only field terms did) and a `snippet`. `--sort name|relevance` orders the hits, `name` by default. The search SHALL page as `card list` does, and as Himalaya's `pimdir search`: `--page` and `--page-size` page the hits, and `nextPage`, the next page number as a string, `null` on the last page, SHALL be taken back through `--page-token` (pimalaya-cli `PageTokenArg`), which wins over `--page`, a token that is no integer from 1 failing ``Invalid page token <token>``. A hit the store cannot read (its item, or its body that cannot be read, is gone from the blob tree or reads as no vCard) SHALL be an `errors` entry alone, as "A listing names the items it could not read" says, its id the hit's, coded as its failure is or `card-unreadable`, the other hits listed, and a non-empty `errors` SHALL make the command exit 3. Under `--json` the page carries `hits`, `nextPage`, `coverage` and `errors`. The text SHALL be pimalaya-cli's `Listing`: `Account:` and the backend, `Addressbook:` when `-k` names one, `Query:`, `Page:`, `Next page:` and `Incomplete:`, then one line per hit, its id, its address book, what matched when a hit of the page says, its email, phone and name, then the `Error:` lines.

#### Scenario: An option after the query
- GIVEN a pimdir account
- WHEN `cardamum pimdir search alice --page-size 3 --json` runs
- THEN it searches `alice`, three hits a page, and prints JSON
- AND `cardamum pimdir search -k book -- alice -bob` searches `alice -bob` in `book`, while `cardamum pimdir search alice -zoe` is refused, exit 2

#### Scenario: A page and a hit the store cannot read
- GIVEN an indexed pimdir address book of five cards, the body of one gone from the blob tree
- WHEN `cardamum --json pimdir search -s 2 kind:contact` runs
- THEN it prints `"nextPage":"2"`, and the page holding that card lists the others and names it in `errors`, coded `card-unreadable`, exit 3; `--page-token 2` prints the second page

#### Scenario: A word of a note
- GIVEN an indexed pimdir address book holding a card whose `NOTE` says `allergic to peanuts`
- WHEN `cardamum pimdir search peanuts` runs
- THEN the card is listed with `matched` `body` and a snippet holding `peanuts`

#### Scenario: A queued card is not listed
- GIVEN a card created in a pimdir address book and still in the queue
- WHEN `cardamum pimdir search` runs a query its body matches
- THEN it is not listed until the owner applies the create and refreshes the index

#### Scenario: An address book named in a group
- GIVEN an indexed pimdir store whose address book `book` holds a card listed and not fetched and whose address book `team` holds none
- WHEN `cardamum pimdir search "(collection:team) offsite"` runs
- THEN the hits and the coverage are those of `collection:team offsite`: `bodiless` 0 and `complete` true

### Requirement: pimdir counts cards from the index
`cardamum pimdir count <query>...` SHALL answer how many cards the scoped query matches, with its coverage, under the same scope as `pimdir search`.

### Requirement: A pimdir card search says what it did not cover
The output of `pimdir search` and `pimdir count` SHALL carry the coverage of the answer (SEARCH §8.2): `bodiless`, the cards in scope whose body is not local, matched on their summary alone, `missing`, those whose body the index could not read, and `unparseable`, those whose body no parser accepted, the three counted only when a term of the query reads card bodies (free text, `has:phone`, `has:address`, `has:birthday`, `has:photo`, `category:`), a query of summary terms alone counting none and being whole over an address book listed and not fetched; `unindexed`, those whose last change the index has not folded in; `behind`, how many changes the store drew above the index, store-wide; `filling`, whether a first round is still listing an address book in scope; and `complete`, io-pimdir's own verdict (`PimdirSearchCoverage::is_complete`, the lag included), true only when none of these holds. Whether a body out of sight counts is io-pimdir's to say (`PimdirSearchLocal::reads`, `PimdirSearchReads::counts_bodiless`), never a rule restated here, so each count at rest agrees with `complete`. The keys are flat and in camelCase, cardamum's own shape over the per-kind counts of SEARCH §8.2, every query being scoped to cards. The plain output SHALL name what did not hold, `Incomplete: <what>`, as a field above the hits of `pimdir search` and as a line under the count of `pimdir count`, and nothing when the answer is complete.

#### Scenario: A card whose body is not local
- GIVEN an indexed pimdir address book holding a card listed and not fetched
- WHEN `cardamum pimdir count has:phone` runs
- THEN the coverage reports `bodiless` 1 and `complete` false, and the plain output names the card with no local body; `cardamum pimdir count email:@example.org` reports `bodiless` 0 and, nothing else missing, `complete` true, and prints no line

#### Scenario: A card the index could not parse
- GIVEN an indexed pimdir address book holding a card whose body no parser accepted
- WHEN `cardamum pimdir count has:phone` runs
- THEN the coverage reports `unparseable` 1 and `complete` false, and the plain output names the card that does not parse; `cardamum pimdir count email:@example.org` reports `unparseable` 0 and, nothing else missing, `complete` true

#### Scenario: An index behind the store
- GIVEN a card updated in the store since the owner's last refresh
- WHEN `cardamum pimdir count kind:contact` runs
- THEN the coverage reports `behind` above 0, `complete` false, and the plain output says the sync engine refreshes the index

### Requirement: A pimdir index or query failure is coded
A store with no search index SHALL fail both commands with the code `index-missing`, an index another version built with `index-outdated`, one of another draft or shape with `index-version`, and a SQLite lacking FTS5 with `index-fts5-missing`, each saying the store's sync engine builds or rebuilds the index and none advising to delete anything. A query the language refuses SHALL fail with `query-refused`: in plain output the SEARCH §8.4 fault drawn over the query, under `--json` the §8.4 code, span and expectations in `error`. An accepted query the index cannot answer SHALL fail with `query-unanswerable`.

### Requirement: Text output is plain text
A text output SHALL be plain text: no border, no colour, nothing sized to or truncated for the terminal, and it SHALL end with a newline: the `card read` view of a card whose last line lacks one, and the pretty JSON of the `jmap`, `msgraph` and `gpeople` `request` commands, included. A list of short rows prints as aligned columns, a list of wide items as records, a single item as `Label:  value` fields (pimalaya-cli `Columns`, `Records`, `Fields`). `carddav discover` SHALL print its endpoints as fields, `Server`, `Principal` and `Addressbook home-set`, `(unresolved)` for one it could not resolve, as calendula's `caldav discover` prints its own.

#### Scenario: A card read through a REPORT
- GIVEN a pimdir card whose body ends `END:VCARD` with no line break, as a CardDAV `REPORT` stored it
- WHEN `cardamum card read` prints it
- THEN the text ends `END:VCARD` and one newline, while `--json` keeps the bytes

#### Scenario: A principal left unresolved
- GIVEN a CardDAV account naming its home, whose server answers no principal
- WHEN `cardamum carddav discover` runs
- THEN it prints the `Server:`, `Principal:` and `Addressbook home-set:` fields, the values aligned, `Principal:` reading `(unresolved)`, and `--json` prints `"principal":null`

### Requirement: Help is plain text
`--help` and `-h` SHALL print without colour, each paragraph on one line, never wrapped to the terminal width.

### Requirement: A query error is drawn without colour
The report `pimdir search` and `pimdir count` draw over a query they refuse SHALL carry no colour escape, on a terminal as in the `--json` error string.

### Requirement: Plain output is printable
A field taken from a card or a server (a name, an address, an id, an error a server wrote) SHALL have its control characters and bidi controls replaced before it is printed as text (pimalaya-cli `sanitize::text`), the write confirmations included. The `card read` view SHALL replace them in the card too, keeping tabs and line feeds and reading CRLF as LF. JSON strings and the raw bytes a command is asked for SHALL remain unchanged.

### Requirement: A card page names its context and its next page
`card list` SHALL print as a pimalaya-cli `listing::Listing`: above the cards `Account: <name> (<backend>)`, `Addressbook: <id>`, `Page: <n>, cards <a> to <b>` (`card <a>` for one, `empty` for none, `all` when unpaged) followed by `, the last` when no card follows, and `Next page: --page <n>` while one does, the values aligned in one column; then a blank line and one line per card with no column header (`id  email  phone  name`, the free-text name last, `-` for an empty cell). Below the cards SHALL print only the command's errors ("A command's errors print at its end").

### Requirement: An address book listing names its account
`addressbook list` SHALL print as a pimalaya-cli `listing::Listing`, as calendula's `calendar list` and Himalaya's `mailbox list` print theirs: `Account: <name> (<backend>)`, then a blank line and one line per address book with no column header, `id  default  color  name`, `default` on the one the server names the default, the free-text name last, `-` for an empty cell; the description is left to `--json`. Below the address books SHALL print only the command's errors ("A command's errors print at its end").

#### Scenario: Two address books
- GIVEN a vdir account `perso` holding `personal`, coloured `#ff0000`, and `work`, with no colour
- WHEN `cardamum addressbook list` runs
- THEN it prints `Account:    perso (vdir)`, a blank line, then `personal  -  #ff0000  Personal` and `work` with `-` in its default and colour cells, aligned, and no header row

### Requirement: Listings resume from their cursor
`card list` SHALL print `nextPage` beside `cards`: the next page number, as a string, while cards follow the page, `null` otherwise, and SHALL take it back through `--page-token`, which wins over `--page`. Whether cards follow is exact on every backend: the unified client reads one card past the page.

### Requirement: Protocol listings page by nextPage
A protocol listing that can stop before the end SHALL print `nextPage`, the provider's cursor as a string, `null` once the provider gives none, and SHALL take it back through `--page-token` (pimalaya-cli's `PageTokenArg`), as pimalaya-cli's output.md "Paging resumes" asks. `gpeople connections list`, `gpeople other-contacts list` and `gpeople contact-groups list` print People's `nextPageToken`, sent back as the `pageToken`, the other flags given again. `msgraph contacts list`, `msgraph contact-folders list`, `msgraph contact-folders child-folders` and `msgraph contacts delta` print Graph's `@odata.nextLink`, which `--page-token` gets as it is: the link carries the folder and the query, so the flags shaping them conflict with it and `child-folders` then takes no folder id, and a link off the Graph origin is refused before the token leaves (io-msgraph). `jmap contact-card query` prints the position the next page starts at, which `--page-token` sends as the query's `position`, the other flags given again: `null` once that position reaches the query's `total`, or, the server giving none, after a page shorter than `--limit`, and a token that is no integer from 0 fails ``Invalid page token <token>``.

The provider's cursor key SHALL NOT print beside `nextPage`; the rest of the payload stays raw, People's `nextSyncToken` and the `@odata.deltaLink` ending a Graph delta round included, the latter still given back through `--delta-link`. The output is pimalaya-cli's `output::Paginated` around the listing: in text the listing, then a `Next page: <cursor>` line, one blank line apart when the listing printed anything. A search the provider answers with no cursor (`gpeople connections search`, `gpeople other-contacts search`) prints none.

#### Scenario: A Graph listing resumed
- GIVEN a Graph contact folder holding more contacts than `--top 2`
- WHEN `cardamum --json msgraph contacts list --top 2` runs, then `cardamum --json msgraph contacts list --page-token <nextPage>`
- THEN the first prints `{"contacts":[…],"nextPage":"https://graph.microsoft.com/v1.0/…"}` with no `@odata.nextLink`, and the second gets that link and prints the next page, `nextPage` `null` on the last one

### Requirement: A listing names the items it could not read
`addressbook list` and `card list` SHALL print an `errors` list beside their items, under `--json` and in text: one pimalaya-cli item error `{id, code, error, sources}` per address book or card the command could not read, `[]` when there is none, in the order met, and one entry with a `null` id and no code when the server truncated the listing (CardDAV). `id` SHALL be the item's id as the command prints ids, `null` when the backend named none; `code` a code of the vocabulary or `null`; `error` the message; `sources` the backend's own reasons, outermost first. An item in `errors` SHALL NOT print among the items, under `--json` nor in text, whatever made it unreadable (a card whose body does not parse or is not UTF-8, a file or a body that cannot be read, a row without its data): it is an entry alone, as calendula's `event list` prints an item it cannot read. It SHALL keep its place in the paging, the page holding it naming it and the next page starting after it. A non-empty `errors` SHALL make the command exit 3 once it printed its result. A listing that reads the whole address book before cutting its page SHALL print its entries on the first page only.

#### Scenario: A vdir card that cannot be read
- GIVEN a vdir address book holding three cards, one of whose files cannot be read
- WHEN `cardamum --json card list` runs
- THEN it prints the two other cards and `"errors":[{"id":"<its id>","code":"card-unreadable",…}]`, and the exit status is 3

#### Scenario: A card that does not parse
- GIVEN a card whose body is no vCard
- WHEN `cardamum --json card list` runs
- THEN `cards` leaves it out, an entry coded `card-unreadable` names it, the text prints no row for it, and the exit status is 3

### Requirement: A command's errors print at its end
In the text output, each `errors` entry SHALL print as one `Error:` line, `<id>  <code>  <error>` with `-` for a null id or code, last in the command's output: after the items and one blank line, the `Error:` lines aligned among themselves (pimalaya-cli `error::ItemErrors`). Nothing SHALL print, the blank line included, when `errors` is empty.

### Requirement: A CardDAV report names the cards it could not read
`carddav report query` and `carddav report multiget` SHALL print an `errors` list beside `cards` and `truncated`, under `--json` and in text, as "A listing names the items it could not read" says: one pimalaya-cli item error per row the server answered without the card's data, coded as a CardDAV listing row is ("Each backend names the cards it could not read"), and on `report multiget` a row answered 404 or 410, an id the address book does not hold, coded `card-not-found`; `[]` when there is none. A non-empty `errors` SHALL make the command exit 3 once it printed its result. A result the server truncated SHALL stay `truncated`, its text line kept, and SHALL add no entry: a query's `--limit` asks for the cut (RFC 6352 §8.6.1), so a truncation is no partial outcome there. In the text the `Error:` lines print last ("A command's errors print at its end"), after the table and the truncation line, with no blank line leading an output that holds nothing else.

#### Scenario: An id the server does not hold
- GIVEN an address book holding `c1.vcf` and no `c9.vcf`
- WHEN `cardamum --json carddav report multiget <book> c1.vcf c9.vcf` runs
- THEN it prints `c1.vcf` in `cards` and `"errors":[{"id":"c9.vcf","code":"card-not-found",…}]`, and the exit status is 3

### Requirement: A raw CardDAV report addresses and counts as io-webdav does
`carddav report raw` SHALL send its REPORT to the path io-webdav builds for the address book (`rfc4918::collection_path`): an absolute id as given, a relative one joined to the home set and percent-encoded, a slash closing it; the id is refused first, as every CardDAV command refuses one ("A CardDAV id addresses its own resource" in backends). It SHALL run it through io-webdav's client ("A closed CardDAV connection is reopened" in backends), a refusal carrying the code any CardDAV refusal carries ("A server's refusal carries its code" in backends). Its `dataBytes` SHALL be the size of the row's whole `address-data` text, the final CRLF included, the size of the body `carddav get` returns.

#### Scenario: A card whose CRLF the server escapes
- GIVEN a home set `/dav/` whose server answers a REPORT with one 54-byte card ending in `END:VCARD&#13;` and a line feed
- WHEN `cardamum --json carddav report raw "My Book" '<x/>'` runs
- THEN the request line is `REPORT /dav/My%20Book/ HTTP/1.1` and the row's `dataBytes` is 54

### Requirement: Capability registry
Each backend SHALL declare, in a table compiled into every build, each capability of the unified API (`addressbook.list`, `addressbook.create`, `addressbook.rename`, `addressbook.delete`, `addressbook.description`, `addressbook.color`, `card.list`, `card.read`, `card.add`, `card.update`, `card.remove`, `card.update.if-match`, `card.remove.if-match`) at `full`, `partial`, `server` or `none`, every level but `full` with a detail saying what differs, which server refuses, or what serves it instead. A backend declares a capability only when it gives the result every other declarer gives.

### Requirement: An unserved verb is refused before connecting
A unified command SHALL check the capabilities it reaches, those its flags reach included (`--if-match`, an addressbook's name, description or colour), against the backend the account resolves to, before reading the store or connecting, and SHALL fail with `capability-missing` naming the backend, the capability and the detail.

### Requirement: An account lists what it serves
`account capability` SHALL print pimalaya-cli's `capability::AccountCapabilityOutput`, connecting nowhere: `{"account", "capabilities": [{"name", "backend", "level", "detail"}]}`, one row per capability, `backend` the backend the unified commands resolve to (`--backend` honoured), `level` one of `full`, `partial`, `server` and `none`, `detail` what the level leaves out or `null`. When the account configures no backend `--backend` allows, every row SHALL read `backend` `null` and level `none`, and the command SHALL succeed. When it resolves to a backend the build leaves out, the command SHALL fail coded `backend-not-built`, that backend serving nothing. The text SHALL be a list headed by `Account: <name>`, then one line per capability: the name, the backend, the level and the detail.

#### Scenario: A backend the account lacks
- GIVEN an account configuring vdir alone
- WHEN `cardamum --json --backend pimdir account capability` runs
- THEN every row has `"backend": null` and `"level": "none"`, and the exit status is 0

### Requirement: The help names what each backend serves
The `--help` of each unified leaf SHALL end with a `Backends:` footer generated from the tables: per capability the leaf reaches, the backends grouped by level, then each detail with the backends it applies to.
