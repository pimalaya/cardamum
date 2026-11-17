# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `default` to every address book in `addressbook list --json` (pimdir draft-04): `true` on the one the server names the default, read on the pimdir backend from the role the sync engine recorded (`collections.role`), JMAP's `isDefault`, the default Contacts folder on Microsoft Graph and myContacts on Google People, `false` on CardDAV and vdir, which name none.
- Added the common fields of a card to the `card list` and `card read` JSON: `uid`, `fullName`, `givenName`, `familyName`, every address in `emails` and number in `phones`, `organization`, `organizationUnits`, `title` and `note`, read through the decoded vCard.
- Added `--if-match` to `card delete`: CardDAV sends it as `If-Match`, pimdir checks it against the card's version, Microsoft Graph and JMAP compare it with the version they read first, and the backends that cannot gate a delete (vdir, People) refuse it.
- Added `--if-match` on Microsoft Graph and JMAP to `card update` and `card delete`, where it was refused: compared with the contact's `changeKey` or the card's hash read just before the write, a card changed since failing `precondition-failed` before anything is sent, as calendula guards a Graph event. `account capability` says `partial`, a write landing between that read and the write not being caught. `card update` with no vCard source now guards with the version it read there too, at no extra request.
- Added the `--json` error codes `store-not-current`, `sync-version`, `store-owned`, `store-staging`, `store-in-flight`, `store-reset`, `flag-invalid`, `conflict-unread`, `no-performer`, `ambiguous-performer`, `index-zone-unknown` and `tag-refused`: cardamum relays every pimdir code Himalaya relays.
- Added the `--json` error code `addressbook-default-missing`: a card or address book command given no `-k` over an account setting no `addressbook.default` fails with it, naming the flag and the key, where it failed uncoded, as calendula codes `calendar-default-missing`.
- Added `-V` and `--version` to `repl` lines, printing the version as the command line does.
- Added a stable `code` to the `--json` error output for failures a caller acts on: `body-pending`, `precondition-failed`, and on pimdir the code io-pimdir names for a store or a write it refuses: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-exposed`, `store-busy`, `lock-replaced`, `unsupported`, `item-duplicate`, `action-malformed`.
- Added `pimdir status`, reporting the local pimdir store behind the account: its address books, how many of their cards are downloaded and how many creations are queued, every write the sync engine parked rather than applied, with its queue row, its kind and the reason, and every card a server refused for good (pimdir storage-draft-13), with its source and the server's reason, `refused: [{addressbookId, itemId, source, reason, refusedAt}]` under `--json`: the store applied the write, the server refused the change it led to, and the sync engine holds that change until the card is edited or removed.
- Added `pimdir search`, searching the account's cards in the pimdir query language through the store's search index.

  The sync engine builds and refreshes the index, cardamum only reads it. A hit carries the `card list` fields, its address book, what matched and a snippet; `-k` narrows to one address book, `--sort name|relevance` orders. It pages as every listing does, as in Himalaya: `nextPage` under `--json`, the next page number or `null` on the last page, taken back by `--page-token` (`--page` and `--page-size` stay), and a card the store cannot read, or whose body is gone from its blob tree or reads as no vCard, is an `errors` entry alone, coded `card-unreadable` unless the store names its own code, the command then exiting 3: `{hits, nextPage, coverage, errors}`. The text is one line per card under `Account:`, `Query:`, `Page:` and `Next page:`, as `card list` prints, then the `Error:` lines. The answer carries its coverage, `{complete, bodiless, missing, unparseable, unindexed, behind, filling}` under `--json` and an `Incomplete:` line above the cards otherwise: the cards whose body is not local or whose body the index could not read or parse, each counted when a term of the query reads card bodies, and the index lagging the store. An option is read wherever it stands, after the query words too; a word starting with `-`, a negated term, goes after `--`.
- Added `pimdir count`, counting the cards a pimdir query matches, with the same coverage and the same query.
- Added `pimdir queue list`, `pimdir queue show` and `pimdir queue cancel`, as Himalaya's: the cards queued for creation, which no listing shows before a sync applies them, and the rows the sync engine parked, each as the family's queue row under `--json`, pending or parked, `{row, kind, collection, linkId, itemId, producer, queuedAt, attempts, unconfirmed, error}` plus `fullName`, every key printed (pimalaya-cli's output.md); where one row stands (`pending`, `parked`, `unconfirmed`, `applied` with the card it acted on and the refusals standing there, `refusals: [{collection, source, reason, refusedAt}]` under `--json` and a `Refused by <source> in <collection> at <instant>: <reason>` line each in text, `unknown`); and the way to retract one, taking the store owner's role for that one write and refused while a sync holds it.
- Added the `--json` error codes `index-missing`, `index-outdated`, `index-version`, `index-fts5-missing`, `query-refused` and `query-unanswerable`, raised by `pimdir search` and `pimdir count`.
- Added a welcome for a first run: a person at a terminal meeting no configuration is shown the commands creating it from the shipped sample, then the first command to run.
- Added `nextPage` to `card list --json`, the next page number while cards follow and `null` on the last page, and `--page-token`, taking it back.
- Added `nextPage` to `jmap contact-card query --json`, the position the next page starts at while the query holds more cards than the page, `null` otherwise, and `--page-token`, sending it back as the query's `position`.
- Added `carddav.alpn`, the ALPN identifiers offered to a CardDAV server, `["http/1.1"]` by default.
- Added config.sample.toml to releases and packages, installed as share/doc/cardamum/config.sample.toml.
- Added `account capability`, listing what an account serves of the unified API without connecting: one row per capability, `{name, backend, level, detail}` under `--json`, the backend it resolves to (`null` when the account configures none `--backend` allows), its level and what the level leaves out. Its output is pimalaya-cli's, shared with Himalaya.
- Added a `Backends:` section to the `--help` of each `addressbook` and `card` command, naming which backend serves what.
- Added the `--json` error code `capability-missing`.
- Added `repl`, a session running the CLI's commands over one account, the unified client connecting once.
- Added `mcp`, a Model Context Protocol server exposing the unified API as tools, the writes listed under `--allow-write` only.
- Added the `--json` error code `stdin-taken`, for a command reading stdin inside a session.
- Added `code` to each backend of `account check --json`, the stable code of its failure (`store-busy`, `store-uncreated`...), `null` when it passed or the failure carries none.
- Added MIGRATION.md, the changes to act on coming from 0.3.
- Added an `errors` list to `addressbook list` and `card list`, under `--json` and in text: one item error `{id, code, error, sources}` per card the listing could not read, `[]` when there is none, described under `json-schema` as `cardamum-item-error`.

  A card whose body is no vCard or not UTF-8 is an entry, as are a vdir file that cannot be read, a JMAP card that does not project, a pimdir body that cannot be read and a CardDAV row the server answered without the card's data, each left out of `cards`; a CardDAV listing the server truncated adds an entry with a `null` id. The text output prints each as an `Error:` line at its end, and the MCP `card_list` and `addressbook_list` tools return them beside `isError: true`.
- Added the `--json` error codes `account-missing`, `account-default-missing` and `account-default-several`, for an account `-a` names that the configuration does not hold, and for no account or several marked `default = true`.
- Added the `--json` error code `backend-not-built`: a command over an account whose backend this build leaves out fails naming the cargo feature to rebuild with, where it failed uncoded as if the account configured no backend.
- Added `notBuilt` to `account list --json`, the account's backends this build leaves out, `backends` keeping the ones it carries; the text marks them `(not built)`.
- Added the `--json` error codes `permission-denied`, `quota-exceeded` and `rate-limited`, read from the server's refusal on every network backend and in `account check` rows, and `card-not-found`, `addressbook-not-found`, `card-unreadable` and `card-invalid`, a card a CardDAV server refuses as no vCard it takes (a 400 or a 403 naming `valid-address-data`, `supported-address-data` or the vCard), with the hint naming `UID` and `N`, where such a 403 read `permission-denied`.
- Added `--id <ID>` to `addressbook create`, as calendula's `calendar create <NAME> [--id <ID>]`: the vdir directory or the CardDAV URL segment of the address book, `<NAME>` then its display name alone, the name staying the id without it. JMAP, Microsoft Graph and Google People mint their own ids and pimdir creates none, so they refuse it before connecting, coded `capability-missing`; `account capability` lists it as `addressbook.create.id`.

### Changed

- Changed `carddav.auth` to be optional: a block naming none sends no `Authorization` header, as `auth = "none"` does, for a server that asks for no credential (Radicale with `auth = none`, a server behind a VPN or an authenticating reverse proxy), where it failed "missing field `auth`". A server answering such a block 401 fails with "CardDAV server answered 401: it asks for credentials, and the account configures no `auth`", still uncoded, as carillon reads its DAV blocks.
- Changed `msgraph contacts list`, `msgraph contact-folders list` and `msgraph contact-folders child-folders` to show `--top` long-only, as Himalaya's Graph listings: `-s` stays accepted as a hidden alias.
- **Breaking:** renamed `card create` to `card add`, as Himalaya's `message add`, its `json-schema` key becoming `cardamum-card-add`; `card create` and `card new` stay hidden aliases, and the MCP tool is `card_add`.
- **Breaking:** renamed `vdir item create` to `vdir item add`, as the family names the creation of an item (pimalaya-cli), its `json-schema` key becoming `cardamum-vdir-item-add`; `vdir item create` stays a hidden alias and `vdir item new` a visible one. No MCP tool reaches `vdir`.
- Changed `addressbook delete` and `addressbook update` to take the address book as a positional, as Himalaya's `mailbox delete <MAILBOX>`: `-k/--addressbook` stays a hidden alias, both together refused as `usage-invalid`, and the MCP tools keep their `addressbook` property.
- **Breaking:** changed the `gpeople` resource commands to the plural nouns of the People API, `connections`, `contact-groups` and `other-contacts`, as Himalaya's `gmail` names its resources, their `json-schema` keys following the plural paths; the singular names stay hidden aliases.
- Changed the version a JMAP card reports (`etag`, and what `--if-match` is compared with) to the BLAKE3 digest of its JSON, stable across builds, where a hash from Rust's `DefaultHasher` could change with the Rust release a build used.
- Changed the unified commands to refuse a verb or a flag the account's backend does not serve before connecting, with `capability-missing`, rather than once connected with an uncoded error.
- Changed every dependency to name the features it needs, and base64 to base64-codec.
- Changed config.sample.toml to one `[accounts.example]` table and one commented section per backend or provider, each tested to parse.
- **Breaking:** changed `-c` to take one path per flag, never split on `:`, and stopped reading `CARDAMUM_CONFIG`.
- **Breaking:** changed a command line the grammar refuses under `--json` to print on stdout the failure object coded `usage-invalid`, where clap printed its text on stderr; it still exits 2, the family exit contract (pimalaya-cli) keeping 2 for a refused command line and 3 for a partial outcome. A refused REPL line and an MCP tool call whose arguments the grammar refuses carry `usage-invalid` too.

  Without `-c`, every canonical path that exists is loaded, each deep-merged on top of the ones before.
- **Breaking:** changed every text output to plain text: no border, no colour, nothing sized to the terminal.

  `card list` prints one line per card, the account, the addressbook, the page and the next one above. `-s 0` lists every card. `table.preset`, `table.arrangement` and every `*-color` key are gone, their tables refused by name.
- **Breaking:** changed the configuration to refuse a key it does not know at the top level and in an account, where it ignored it, as every backend and option table already did. The tables v0.3 read for tables and colours (`table`, the top-level `account`, `addressbook.list`, `card.list.table`) are refused by name, the error pointing at MIGRATION.md. The block of a backend the build leaves out still loads unread.
- **Breaking:** renamed `vdir.home-dir` to `vdir.root`, as `pimdir.root` names a store, its `~` and environment variables expanded as before; `vdir.home-dir` is refused by name (``The `vdir.home-dir` key is removed, …: rename it `vdir.root`, the path unchanged, see MIGRATION.md``). Calendula renames its key the same way.
- **Breaking:** changed `addressbook list` and `card list` to exit 3 once they printed their result when a card could not be read (`errors` not empty), and the vdir and JMAP listings to list the other cards where one card failed the whole listing.
- **Breaking:** changed `carddav report query` and `carddav report multiget` to print an `errors` list beside `cards` and `truncated`, one entry per card the server answered without its data, coded as `card list` codes a CardDAV row, an id `report multiget` named that the server does not hold `card-not-found`, and to exit 3 when it is not empty, where they dropped those cards and exited 0. A truncated result stays `truncated` alone.
- Changed a card or an address book not found to fail with `card-not-found` or `addressbook-not-found`, where the failure carried no code.
- Changed account resolution to refuse several accounts marked `default`, naming them.
- **Breaking:** changed `-a default` to take the account named `default`, where it took the account marked `default = true`; omit `-a` to take that one. A missing `default` account says so (pimalaya-config d5a33cba).
- **Breaking:** changed `--backend auto` to resolve to the first backend the account declares, whether the build carries it or not, failing with `backend-not-built` when it does not, where a reduced build took the next declared backend it carries. `--backend` naming a backend the build leaves out names its feature. `account check` reports such a backend as a failed row coded `backend-not-built`.
- Changed `account list` to name an account's backends in the order `auto` resolves them (vdir, pimdir, carddav, jmap, msgraph, gpeople), where CardDAV came first.
- Changed `account list` and `account check` to print pimalaya-cli's shared outputs, their JSON unchanged: the `account list` text has no header row, marks the default account `default` rather than `yes`, and names the built backends before those left out, as Himalaya and Calendula print it.
- **Breaking:** changed `card read --json` to print the row `card list` prints for the card (`addressbookId` and the common fields join) plus its vCard, and the vCard of `card read` and `card build` to carry `encoding` beside `contents`: `utf-8` when `contents` is the text, `base64` when the bytes are not UTF-8 and `contents` holds their base64, where such a card failed. Calendula prints an item's bytes the same way.
- Changed the protocol commands naming one card, `carddav get` and `delete`, `msgraph contacts get`, `update` and `delete`, `gpeople connections get`, `update` and `delete`, to fail `card-not-found` when the server holds no such card, as `card read` does, the server's message kept, where they carried no code.
- Changed the protocol commands naming one contact folder, `msgraph contact-folders get`, `child-folders`, `rename` and `delete`, and `msgraph contacts list`, `create` and `delta` under `-f`, to fail `addressbook-not-found` when Graph holds no such folder, as `card list -k` does, Graph's message kept, where they carried no code, as calendula codes its Graph calendar commands `calendar-not-found`.
- Changed `carddav create` to a hidden alias of `carddav mkcol`, where the help showed it, as calendula hides `caldav create`.
- **Breaking:** changed the `carddav discover` text to print its endpoints as fields, `Server:`, `Principal:` and `Addressbook home-set:` with the values aligned, as calendula's `caldav discover` prints them, where it printed bare `server: …` lines; `--json` is unchanged.
- Changed the help of `carddav propfind <ADDRESSBOOK>` to say it enumerates the cards through an `addressbook-query` REPORT asking `getetag` alone, as io-webdav does and calendula's `caldav propfind` says, where it named a PROPFIND.
- Changed the text of `carddav propfind`, `vdir list` and `msgraph contacts` to print the fields an item holds and no other: `carddav propfind` and `vdir list` print the description and the colour their JSON carries, a property the server did not send prints no line where `Name:` and `Color:` printed empty, and a Graph contact prints every email and every phone, the business and home ones beside the mobile one.
- Changed a CardDAV write refused 412 (a stale `--if-match`) to fail `precondition-failed`, and one refused for a `UID` the address book holds (`no-uid-conflict`) `item-duplicate`, where both carried no code, as calendula codes CalDAV.
- Changed a card a vdir or JMAP account does not hold to fail as on the other backends, with cardamum's sentence and the backend's answer in `sources`, where the error was io-vdir's or JMAP's own text.
- Changed the sentence of a card not found to name its address book on every backend, ``Card `<id>` not found in address book `<book>` ``, as Himalaya words a message not found in its mailbox, where it read ``Card `<id>` not found`` (``Card `<id>` not found in `<book>` `` on pimdir); the code stays `card-not-found`.
- **Breaking:** changed `carddav get`, `carddav report query`, `carddav report multiget` and `vdir item get` to carry `encoding` beside each vCard's `contents`, as `card read` does and calendula's `caldav` and `vdir item get` reads do: `utf-8` when `contents` is the text, `base64` when the bytes are not UTF-8 and `contents` holds their base64, where `carddav get` and `vdir item get` failed on such a card; their text prints such a card as its bytes.
- Changed the `addressbook list` text to print one line per address book under `Account: <name> (<backend>)`, `id  default  color  name` with no header row and `default` on the default one, as calendula's `calendar list` and Himalaya's `mailbox list` print theirs, where it printed an `ID DEFAULT COLOR NAME` header and `yes`; `--json` is unchanged.
- Changed `vdir create <NAME>` to `vdir create <ID>`, adding `-n/--display-name <NAME>` for a display name apart from the directory, the id still when omitted, and named the arguments of `vdir rename <OLD-ID> <NEW-ID>` and `vdir delete <ID>`, as calendula names them; `addressbook update -n` takes a `<NAME>`, where it read `<TEXT>`.
- Changed the summary of the `pimdir`, `vdir`, `jmap` and `gpeople` namespaces to read as Himalaya's and calendula's, `<API>-specific API`: `pimdir-specific API`, `vdir-specific API`, `JMAP-specific API`, `Google People-specific API`.
- Changed the aliases `rm` of `addressbook delete` and `card delete`, `new` of `card add` and `folders` of `msgraph contact-folders` to hidden ones, as calendula shows none for their counterparts.
- Changed the `-b/--backend` help to read as Himalaya's and list its values, `auto` then every backend an account may name, and a value outside them to be refused with the list, where it read `Invalid backend <value>`.
- Changed a `repl` or `mcp` line inside a session to fail coded `usage-invalid`, refused by pimalaya-cli, where it carried no code.
- Changed an MCP tool call missing an argument or giving a value the grammar refuses to name the argument as the tool takes it (``Missing argument `card-id` ``, ``Invalid value `-1` for `page-size`: …``), where it printed clap's usage text.
- Changed the refusal of a removed table or key to pimalaya-config's sentence, ``The `table` table is removed, …: delete it, see MIGRATION.md``, its shape unchanged.
- Bumped pimalaya-cli to 0.6.0: `Next page:` prints after the listing, one blank line below the items and above the `Error:` lines, as `Next page: --page 2`, and a listing's header values sit two spaces past its longest label (`Account:  work`).
- Bumped io-gpeople to 0.8.1: an update of a contact still carrying its stash no longer reports as dropped the properties the stash keeps.
- Bumped io-msgraph to 0.9.2.
- **Breaking:** changed `msgraph request` and `gpeople request` to refuse a full URL off the provider's API origin (another host, scheme or port than `https://graph.microsoft.com` or `https://people.googleapis.com`) before sending it, so the account's bearer token never leaves for another host; a path and a URL on the origin (a next link, Graph's `beta`) are sent as before. `jmap request` takes no URL and posts to the session's `apiUrl` alone.
- Bumped pimalaya-config to 0.5.2: an unknown key's error lists no removed key.
- Changed the `pimdir` feature to build io-pimdir's search, adding ical-rs, mime-rs and ariadne to the binary.
- **Breaking:** changed every write printing a confirmation to print a receipt instead of `{"message": ...}` under `--json`: `addressbook update|delete` and `card delete` print `{id, queued}`, the item acted on and whether the write waits for a sync, `card add` `{id, queued}`, `card update` `{id, keptProperties, queued}` and `addressbook create` `{id, name, queued}`, `queued` `true` on pimdir and `false` elsewhere; `carddav mkcol|proppatch|delete`, `vdir create|rename|delete`, `vdir item update|copy|move|delete`, `jmap address-book destroy`, `jmap contact-card destroy`, `msgraph contacts delete`, `msgraph contact-folders delete` and the `gpeople` deletes print `{id}`, a renamed collection's new id, as do `jmap address-book update` and `jmap contact-card update` when the server returns no object; `jmap contact-card copy` prints `{ids}`, `carddav put` `{id, etag}`, `gpeople contact-groups members` `{id, added, removed}` and `card build -o` `{path, size}`, its text `Saved <size> bytes to <path>` (pimalaya-cli's `SavedOutput`). The text form keeps its sentence. Every command now has a schema under `json-schema` (pimalaya-cli 0.3).
- Bumped pimalaya-cli, pimalaya-config and pimalaya-stream to 0.4 and io-jmap to 0.9. The `--json` failure object always carries `code`, `null` when none applies.

- Bumped rustls-webpki to 0.103.16 and webpki-root-certs to 1.0.11, fixing RUSTSEC-2026-0382 and RUSTSEC-2026-0383.
- Changed piped `--json` output to print one value per line, each schema of `json-schema` to carry an `$id` naming the contract version (`v0.3`), and added `cardamum-error`, the schema of the JSON failure object (pimalaya-cli `55b6693`). The wire objects of `gpeople`, `jmap` and `msgraph` are now described as JSON objects, and the abandoned arm of `card add` and `card update`, which `--json` never prints, is now `{}` rather than `null`.
- **Breaking:** changed the `--json` output of `gpeople request`, `jmap request` and `msgraph request` to wrap the raw response as `{"response": …}`, so the payload is an object even when the server answers no body (`null`) or an array.
- Changed `card list` on pimdir to stream the store rather than load it: it holds one batch of rows and the page it prints instead of every card body of the address book, reads the bodies of that page only, and stops once the page is full. The output is unchanged.
- Changed io-pimdir to 0.14.1 (pimdir storage-draft-13, sync-draft-13 and search-draft-13), io-msgraph to 0.9, io-gpeople to 0.8 and io-http to 0.8, so cardamum shares one io-pimdir and one store schema with neverest, himalaya and calendula: an empty mail date is stored as no date, a band round keeps undated mail, and opening a store creates the `items_by_sort_global` index a page across collections walks. A response whose head passes 64 KiB or whose body passes 256 MiB now fails the exchange with a typed error instead of growing the buffer without bound,, a connection closing mid-body fails rather than spinning, and People and Graph fold vCard lines over 75 octets.
- Bumped io-webdav to 0.8. An address book outside the home set is listed and addressed by its path rather than its last segment, a request the server closed the connection on is resent once, and a CardDAV multistatus is parsed 3.4 to 9 times faster, without quadratic copies and nested 64 levels at most.

  A CardDAV answer that is not one whole multistatus document fails instead of listing fewer cards, and a weak ETag is kept whole.
- Bumped io-webdav to 0.9.1.

  A card `card list`, `carddav report query` or `carddav report multiget` reads keeps its final CRLF, which iCloud and Fastmail escape and the listing trimmed, so it matches the body `card read` and `carddav get` return; a relative address book or card id is sent percent-encoded, a `\` no longer read as a separator.
- **Breaking:** bumped io-webdav to 0.10.0: a CardDAV address book or card id prints percent-decoded when the decoding encodes back to it, in every listing (`addressbook list`, `card list`, `carddav propfind`, `report query` and `report multiget`) and in the id `card add` and `carddav put` print, so `addressbook create "My Book"` prints `My Book`, the id `addressbook list` then shows, where the listing printed `My%20Book`.

  A segment whose decoding would not be sent back as served (`%2F`, `%40`, lowercase hex) keeps its served form, and an id printed before still addresses its resource.
- Bumped io-webdav to 0.11.0, whose client runs the two requests cardamum sent by hand (see Fixed).
- Bumped io-vdir to 0.3.

  A vdir listing leaves out a card removed while it lists and holds regular files only, and an id that is empty or holds a path separator is refused.
- Changed `carddav put` to print the id the write answered, as a listing shows it, where it echoed its argument.
- Changed a JMAP set error of a type io-jmap does not model to be named as the server sent it, where it read `unknown`.
- Bumped vcard-rs to 0.6.

  A 3.0 or 4.0 line longer than 75 octets that `card build`, `card add` or `card update` writes or edits is now folded.

- Changed the pimdir `update_card` and `delete_card` to honour `--if-match` against the `etag` they report, the store's hash of the card's body: a write naming a version the card no longer has is refused before anything is queued, with an error starting `Precondition failed:`.
- Changed `card update` and `card delete` on pimdir to name, on the write they queue, the version `--if-match` matched (pimdir STORAGE §15.3): a write queued over a card a sync changes before applying it is parked rather than applied, and shows in `pimdir status`.
- Changed a `card update` or a `card delete` of a conflicted pimdir card to be parked by the sync engine, the card and its diverging body kept, where a delete settled the conflict: cardamum decides no conflict, a client showing it or the server does.
- Changed the pimdir writes to be refused before they are queued when a source of the store does not support them, naming the capability, the source and why (pimdir draft-03, STORAGE §15.6). A write a source supports only in part logs a warning.
- Changed `card add`, `card update` and `card delete` on pimdir to say the write is queued for the next sync instead of reporting it done; the `--json` output is unchanged.

  A queued create reports the card's link id, its `UID`, and is not listed until the sync applies it.

- **Breaking:** changed pimdir with `pimdir.account` unset to read the one account the store groups its collections under, and to refuse a store grouping several, naming them.
- Changed `account check` on pimdir to probe a producer as well as the reader, so a store whose staging lock cannot be taken fails the check rather than the first write; a sync collecting the store fails it at once with `store-busy`, and the check writes nothing into the store.
- Changed the refusal of a pimdir store cardamum cannot read to say what it asks instead of deleting the store, which loses the queued writes a resync never restores: a store of another format version is reinstalled through neverest, its queue drained first, or asks for a cardamum upgrade.
- **Breaking:** changed the pimdir backend to read a store of pimdir storage-draft-13 alone: a store any earlier neverest wrote, a draft-11 one included, is refused with `store-version` until neverest rebuilds it on draft-13, a search index of an earlier draft with `index-version` until the sync engine rebuilds it, and a store another user can read is refused with `store-exposed`.
- Changed the `addressbook delete` refusal on pimdir to name the server route only, no longer advising to remove the store directory, which loses the queued writes.
- **Breaking:** changed `addressbook create` naming an address book the backend holds to fail coded `addressbook-duplicate`, ``Cannot create address book `<name>`: it exists already``, as the family codes a collection created twice: on vdir, where it rewrote the existing one's metadata and succeeded, and `vdir create` too; on CardDAV, a `MKCOL` refused 403 or 405 on an id the home set then lists, where it read `permission-denied` or carried no code; on Microsoft Graph, `ErrorFolderExists`, and on Google People, the 409 of a group name held, both uncoded before. JMAP lets two address books share a name and pimdir creates none.
- **Breaking:** changed `gpeople connections list`, `gpeople other-contacts list`, `gpeople contact-groups list`, `msgraph contacts list`, `msgraph contact-folders list`, `msgraph contact-folders child-folders` and `msgraph contacts delta` to print the provider's cursor as `nextPage`, `null` on the last page, where they printed `nextPageToken` or `@odata.nextLink`, and to take it back through `--page-token`, as the family pages a protocol listing; the rest of the payload stays raw, `nextSyncToken` and the delta's `@odata.deltaLink` included, which `--delta-link` still takes. The text prints a `Next page:` line where it told to follow the token or raise `--top`.

### Removed

- **Breaking:** removed `fnValue`, `email` and `tel` from the cards of `card list` under `--json`, and from the row `card read` and `pimdir search` print: read `fullName`, the first of `emails` and the first of `phones`, which the text columns now print too, an escape decoded and a `tel:` URI as its number where the removed keys kept them as written.
- Removed IDNA for server and proxy hosts: an `http` or `https` URL with a non-ASCII host is refused, its Punycode form working.
- **Breaking:** removed the `configure` command and its `wizard` alias, with the `wizard` cargo feature.

  A bare `cardamum` prints the help, and a command meeting no configuration fails naming the path it looked at and the documented sample.
- **Breaking:** removed the `discover` route of the `[carddav]` block, which resolved a bare domain on every run through PACC and RFC 6764 SRV and TXT records. Name the server in `carddav.server` (a bare host still goes through its `.well-known/carddav`), or the address book home in `carddav.home`, as Google needs. A configuration holding `carddav.discover` is refused by name, the error pointing at MIGRATION.md.
- Removed the io-pim-discovery dependency, and with it the DNS stack only it pulled (domain, octseq, psl, resolv-conf, ipconfig).

### Fixed

- Fixed `carddav.server = "https://<non-ASCII host>/..."` being read as the host `https`, the URL prefixed with `https://` once it failed to parse: a CardDAV `server` or `home`, or a JMAP `server`, whose host is not ASCII is refused with "Host `bücher.de` is not ASCII: write it in its Punycode form (xn--...)", naming the host, where `home` and the JMAP `server` failed "invalid international domain name".
- Fixed `card add -k <group>` on Google People leaving a contact in myContacts when the group does not exist: it created the contact, then failed adding the membership with a bare 404, so a retry made a second one. Every card command on People now reads the group it names first, and an unknown one fails `addressbook-not-found` before anything is written; a membership refused after the create says the card stays in Contacts.
- Fixed `card list -k <group>` on Google People listing an unknown group as empty with exit 0: it fails `addressbook-not-found`, as vdir and pimdir do. `card read`, `update` and `delete`, which ignored the group, check it too, and `addressbook update` and `delete` code a group People does not hold the same way.
- Fixed `card read` printing a card's control characters as is, which could drive the terminal: the text view replaces them, CRLF read as LF, and `--json` keeps the bytes.
- Fixed the write confirmations printing a server-given id as is.
- Fixed a unified command spawning its secret commands again when it reopened its backend after the composer.
- Fixed `account check` exiting 0 when a backend failed: it now prints its report and exits 3.
- Fixed pimdir accepting writes into an address book no source syncs, which no sync would ever push: they are now refused before anything is staged.
- Fixed a pimdir card whose body blob is missing reporting an `etag`, which an `--if-match` could then match.
- Fixed a year-less birthday (`BDAY:--MMDD`) on the gpeople backend being stashed instead of becoming a People birthday, so Google now shows it and creates its yearly calendar event (io-gpeople `e507d82`, [#26](https://github.com/pimalaya/cardamum/issues/26)).
- Fixed pimdir recognising an address book only by the exact kind `text/vcard`: a kind carrying parameters now counts.
- Fixed `card add` on pimdir reporting as queued a card whose `UID` the address book already holds, synced or queued, which the sync then parked: it is now refused with `item-duplicate`, leaving nothing in the store, and `-i` keeps the composer file.
- Fixed a pimdir `card update` made from the card it read (no vCard source, or `--if-match`) overwriting, unmerged, a server change a sync brought in before applying it: the queued update now names the body it was made against, and the sync engine parks it instead.
- Fixed `card update` on Google People erasing what a vCard cannot show of the fields it changes: a further organization, a relation of another type, a custom type, a sub-field without a vCard slot.
- Fixed `card add` and `card update` on Google People leaving out a property longer than 8 KiB without a word: a warning names it.
- Fixed `addressbook update --description ""` on JMAP leaving the description in place: it is cleared.
- Fixed the Microsoft Graph listings following a next page link off the Graph origin with the account's token: such a link is refused.
- Fixed a credential command prompting for a password (`systemd-ask-password`) hanging the run: it now reads the terminal when cardamum runs in one (pimalaya-config 0.4.1).
- Fixed `account check` passing a CardDAV account that names `carddav.home` whatever its credential, as the home route connected without a request: the check now sends the home one `PROPFIND`, so a refused credential fails its row as with `carddav.server`.
- Fixed `card list` printing a card it could not read as a row of empty fields (a pimdir one as its preview) beside its `errors` entry: the card is in `errors` alone, under `--json` and in text, and keeps its place in the pages.
- Fixed a data loss on vdir: an addressbook id that is not one directory name reached out of the home, so `addressbook delete -k ..` deleted everything in the directory holding the home, the home and its siblings included, `-k ../<name>` a directory beside the home, `-k .` and `-k <book>/..` every addressbook of the home, and `vdir delete ""` the home itself. Such an id (empty, `.`, `..`, holding a separator) is now not found: every vdir command fails `addressbook-not-found` before reading or writing anything, `vdir delete`, `vdir rename` and `vdir item` included, and `addressbook create` and `vdir create` refuse such a name rather than write out of the home. An empty `-k` is coded `addressbook-not-found` on every backend.
- Fixed `card update` with field flags or `-i` and no `--if-match` failing at once on Microsoft Graph ("Microsoft Graph does not support If-Match guarded updates") and on JMAP, sending nothing: the version of the card it read guarded the write on backends that take no guard. There it now sends the plain update, the version it read compared first (see `--if-match` on Graph and JMAP).
- Fixed `card list` on pimdir listing a card whose body blob is gone from the store as its summary preview, exit 0, with a warning in the log: it is a `card-unreadable` entry of `errors` (`Body blob missing from the store`), as calendula names it, and the command exits 3, as in `pimdir search`.
- Fixed `card read` failing, with no code, on a card whose bytes are not UTF-8, and `card build` on such a source: see `encoding`.
- Fixed the name, email and phone columns of `card list` misreading a card's lines: a property with parameters (Fastmail's `FN;DERIVED=TRUE`) listed nameless, a folded line (RFC 6350 §3.2) was cut at its fold (`alan@examp` on Fastmail's JMAP cards), a grouped one (`item1.EMAIL`) was skipped and a `TELEX` was read as a `TEL`. The columns read the decoded vCard, a folded line whole and a property whatever its group and parameters.
- Fixed `card read` and the `jmap`, `msgraph` and `gpeople` `request` text ending without a newline, the shell prompt landing on its last line: a card whose last line lacks one (a CardDAV `REPORT` body) included.
- Fixed `card list` naming an address book a CardDAV server, JMAP or Microsoft Graph does not hold failing with the server's bare refusal and no code: it fails `addressbook-not-found` naming it, the server's answer in `sources`, as vdir, pimdir and People do. On JMAP a query that fails or finds nothing is checked with one `AddressBook/get`; on CardDAV a 404 or 410 of the collection is read so, on Graph a 404 or a malformed folder id, `card add` included; `addressbook update` and `delete` on CardDAV and Graph too.
- Fixed a CardDAV address book or card id addressing another resource reaching the server: `addressbook delete ..` could send a DELETE to the parent of the home set, `/` to the server root and `""` to the home set, `addressbook update` a PROPPATCH there. An id that is empty or holds an empty, `.` or `..` segment now fails before the account's secret is resolved or anything connects, on every unified and `carddav` command: an address book id `addressbook-not-found`, a card id `card-not-found`, as vdir refuses an id that is not one directory name, and a name `addressbook create` or `carddav mkcol` cannot give a collection uncoded.
- Fixed `carddav report raw` counting a card two bytes short in `dataBytes` when the server escapes its final CRLF, and building its path by hand: the REPORT goes to the path io-webdav builds, percent-encoded, an id out of the home refused.
- Fixed `carddav report raw` and the `PROPFIND` `account check` sends a `carddav.home` failing with `reached unexpected EOF` when the server had closed the connection (a keep-alive timed out, `Connection: close`): sent by hand over the client's stream, they missed the resend every other CardDAV request gets. Both now run through io-webdav's client, which sends the request again once on a fresh connection, and a refused `report raw` carries the code a CardDAV refusal carries (`permission-denied`, `quota-exceeded`, `rate-limited`), as `report query` does, where it carried none.

## [0.3.0] - 2026-10-02

### Added

- Added `proxy`, a per-account SOCKS5 or HTTP proxy for every network backend, and `<backend>.proxy` to override it for one backend.

  The password is a secret like any credential, so it stays out of the URL. Without either, `all_proxy` and `https_proxy` are read. CardDAV `discover` lookups only read the environment.

- Added server-side filters to `carddav report query` ([#25](https://github.com/pimalaya/cardamum/issues/25)).

  `--match` and `--not-match <PROP> <TYPE> <VALUE>`, `--defined` and `--not-defined <PROP>` each add an RFC 6352 prop-filter, combined by `--test` (default `allof`). `--collation` and `--limit` complete them, and `report raw` covers the rest.

- Added an FN column to the `carddav report query` and `report multiget` tables, and a `truncated` field to their JSON output.

- Added a composer to `card create` and `card update`, opened by `-i/--interactive` and configured by `card.composer`.

  `card.composer` is a shell line or an argv list, at the top level and per account, and `--composer <COMMAND>` overrides it for one run. It is spawned on a temporary vCard file with every stream inherited, so any command blocking until the edit is done works: `code --wait`, not `code`.

  Changed bytes are the card, an emptied or untouched file abandons the edit, and a non-zero exit is a failure. The result is checked against its version's RFC contract, a re-edit being offered on violations, and a failed write keeps the temporary file and names it.

- Added `card build`, the create pipeline stopped before the write, printing the vCard.

  It takes the same source, field flags and composer, reaches no backend, and reads no configuration unless `-i` needs the composer. So `card build --full-name "Jane Doe"` runs on a machine holding none, and `card read <ID> | card build --title CTO -` previews an update.

  `-o/--output <PATH>` captures the result, `-i` owning stdout, and an abandoned build prints nothing at exit 0. It refuses an invalid card as `card create` does, so `build | create -` is no way past that guard.

- Added vCard field flags to `card build`, `card create` and `card update`.

  The flags are `--full-name`, `--given-name`, `--family-name`, `--middle-name`, `--name-prefix`, `--name-suffix`, `--nickname`, `--organization`, `--title`, `--birthday`, `--email work:a@b`, `--phone cell:+1-555-0100`, `--url`, `--note` and `--uid`.

  A flag replaces every instance of its property and leaves every other line byte for byte.

  `--given-name` and `--family-name` repeat and merge into the card's existing `N` rather than clearing its other components. The flags cover the common fields only: the composer is the complete surface, and `ADR` is deliberately not a flag.

- Added the `json-schema` command (alias `json-schemas`), printing the JSON Schema of a command's `--json` output or writing one file per command into a directory.

- Added `carddav.auth = "none"`, for a server asking no credentials, offered by the wizard where discovery advertised no scheme.

- Added `--fallback` to `carddav report sync`, enumerating the addressbook with a Depth 1 `PROPFIND` on a server without the `sync-collection` REPORT.

- Added the truncation report to `carddav propfind <addressbook>`, which used to drop the fact that the server cut its listing short.

### Changed

- **BREAKING**: renamed the Google People backend from `people` to `gpeople`, as `gmail` and `gcal` already are in Himalaya and Calendula.

  The cargo feature, the account config table (`gpeople.auth…`) and the command (`cardamum gpeople …`) all change, with no alias: rename `people` to `gpeople` in the config. The backend now uses io-gpeople, the renamed io-people.

- **BREAKING**: switched every multi-word `--json` key from snake_case to camelCase, such as `addressbookId`, `fnValue`, `keptProperties` and `syncToken`.

  A key a provider owns keeps its spelling, so `@odata.nextLink`, `nextPageToken`, `contactGroups` and the JMAP `list` are untouched. The TOML configuration stays kebab-case.

- **BREAKING**: made `addressbook create`, `card create`, `card update` and `vdir item create` emit their result under `--json` instead of a prose message.

  The result is `{"id"}`, and `{"id", "keptProperties"}` for the update. Terminal output is unchanged, word for word.

- **BREAKING**: moved the pimdir backend off the store's owner role and removed `pimdir.source`.

  It reads through a lock-free reader and stages each write as one queue action through a short-lived producer. A listing now runs beside a sync instead of failing against it, and a staged write reads back before the sync applies it. The store must already exist, creating it being the sync engine's job.

- **BREAKING**: made the pimdir backend refuse `addressbook create`, `update` and `delete`, a collection being declared by the store's owner.

  `card create` reports the card's link id, a queued create having no store-assigned id until the sync applies it.

- **BREAKING**: made the pimdir backend refuse a store written by an earlier draft of the format, naming the table it lacks.

  There is no migration: delete the store and let the sync recreate it. io-pimdir now holds the sync engine io-replica used to, and its summaries are typed, so a listing previews an undownloaded card from the store's contact summary.

- Kept the vCard UID of a card created or updated on Microsoft Graph or Google People, which used to come back replaced by one derived from the provider id. **Behaviour change.**

  A contact the provider created, or that an earlier version wrote, keeps the UID it reads back with today.

- Turned `vendored` on by default and forwarded it to io-pimdir, so `cargo install` builds SQLite (for `pimdir`) and OpenSSL (for `native-tls`) from source.

  Drop it to link the system libraries. The Nix builds still link the store's.

- Renamed `completions` and `manuals` to `completion` and `manual`, the plurals staying as hidden aliases.

  A command mirroring a vendor API resource keeps that API's spelling, so `gpeople contact-group members`, `jmap address-book changes` and `jmap contact-card changes` are unchanged.

  The `msgraph` family is aligned onto Graph, `contact-folder` and `contact` becoming `contact-folders` and `contacts`, every counterpart spelling staying as a hidden alias.

- Reordered the arguments of `card create` and `card update` to the source, the field flags and `-i`, and made the vCard optional on both.

  So `card create --full-name "Jane Doe" -i` opens the composer on a card already carrying the name. A create with no vCard mints one carrying a fresh `UID` at `--vcard-version`.

  An update with no vCard reads the card first and sends the version the backend answered as `If-Match`, so a long edit no longer silently overwrites a write that landed meanwhile. An explicit `--if-match` still wins.

- Deferred opening the backend connection to the first call that needs it, instead of when the client is built.

  A command that never reaches the network opens no socket, and an interactive edit holds none open while the editor is up. A write landing after a long edit used to fail with `unexpected end of file`, the server having closed the idle connection.

- Resolved each account credential once, so a command named by two backends of an account is spawned once.

  An account naming one `pass` entry from its `carddav` and `jmap` tables used to pay two key unlocks. A shell line and its argv spelling stay two commands, compared as the configuration wrote them.

- Reworded the card argument of `card read`, `card update` and `card delete`, which called itself a card `UID`: it is the backend's own identifier, the one `card list` reports.

### Fixed

- Fixed a whitespace-only vCard source being read as a card, handing the backend an empty body and reporting success.

  It is now refused, naming where it was read from. `printf '' | card create -k <AB> -` used to exit 0.

- Fixed a field flag dropping all but the first card of a multi-card source.

  The source is now refused when a flag is set, a flag rewriting only the card read first: `card create --title CTO two-cards.vcf` used to write one card and lose the other at exit 0. With no flag, the source still passes through as written.

- Fixed `carddav.tls.cert` not expanding `~` and environment variables, so a home-relative certificate was looked for in the current directory.

  `pimdir.root` moved its expansion onto the field for the same reason.

- Fixed the pimdir backend accepting a collection of any kind as an addressbook.

  A sync engine caches mail, calendars and contacts in one store, so `card list -k imap/INBOX` printed a mailbox's messages as blank contacts. A wrong id is now refused, naming the addressbooks the account holds.

- Fixed a card vanishing from the pimdir backend when another card of the same addressbook carries its `UID`.

  RFC 6352 requires that `UID` to be unique, but servers hand over duplicates anyway, most often after a repeated import. The store keys the second copy apart, so both cards list, read and act under their own ids.

- Fixed `card list` leaving the `TEL` column empty for a card writing its phone above its mail, on every backend.

  The preview chained its three property reads, so a line that was not an `EMAIL` never reached the `TEL` read.

- Fixed the pimdir backend staging a card under `uid:<UID>` rather than the bare `UID` the sync engine uses, which stored it twice and synced it as a duplicate contact.

- Fixed the pimdir backend naming a body it writes with a digest of its own choosing rather than the hash the store records, which put the body where no read would look for it.

## [0.2.0] - 2026-08-24

### Added

- Added three remote backends alongside CardDAV: JMAP contacts, Microsoft Graph and Google People, each behind its own cargo feature (`jmap`, `msgraph`, `people`).

  None of the three exposes a vCard, so cardamum synthesizes the document of record and re-projects it on write. JMAP converts through JSContact; Graph and People project field by field and stash server-side the properties that have no slot, so nothing is lost round-trip.

- Added the `pimdir` backend: cardamum over a local [pimdir](https://github.com/pimalaya/pimdir) store, the offline cache a sync engine fills.

  A card the sync listed but has not downloaded still lists, and reading it reports "body not fetched" rather than failing. Writes are staged for the next sync to push, and addressbooks come from the sync, so the collection verbs refuse here.

- Added one protocol-specific command family per backend: `carddav`, `jmap`, `msgraph`, `people` and `vdir item`.

  Each mirrors its protocol's own vocabulary and exposes what the shared API hides: ETags and preconditions, sync tokens, `changes` and `delta` incremental sync, multiget, discovery, and the raw native payloads under `--json`.

  Each also carries an escape hatch for a request the commands do not model: `carddav report raw`, `jmap request`, and `request <METHOD> <PATH>` on Graph and People.

- Added the `configure` command (alias `wizard`), the `-b/--backend` flag selecting between the backends an account declares, and the `account list` and `account check` reports.

### Changed

- Reworked the first run around automatic discovery, aligning it with Himalaya, Ortie, Comodoro and Carillon. **Behaviour change.**

  One prompt takes an email address, a server URL or a local folder path, and its shape orients the setup. Discovery runs in parallel under an 8-second deadline and offers one entry per reachable service, picking one prompting only how to authenticate.

  The wizard tests the account, then writes it into the configuration file, appending an `[accounts.<name>]` block when one is already there. It configures only what it can discover, and points at config.sample.toml otherwise.

  A missing configuration raises an offer rather than a gate, so the command carries on either way. A bare `cardamum` shows the help when a configuration exists, and nothing prompts when stdin is not a terminal or `--json` is set.

- Renamed the remote backend from `webdav` to `carddav` across the public surface: the cargo feature, the subcommand and the config block. **Breaking.**
- Renamed the shared commands to the singular `addressbook` and `card`, the plural forms staying as hidden aliases. **Breaking.**
- Replaced the positional addressbook id with a `-k/--addressbook` flag across the shared API, falling back to `addressbook.default` everywhere except `addressbook delete`, which stays explicit. **Breaking.**
- Changed `card create` and `card update` to take their vCard as a trailing positional argument, a path, raw contents, or `-` for stdin, instead of the `--file` flag. **Breaking.**
- Relicensed from AGPL-3.0-only to dual MIT OR Apache-2.0.
- Replaced the io-addressbook aggregator with a product-owned cross-backend layer, and upgraded the whole Pimalaya dependency stack.
- Bumped comfy-table to v8. The `table.preset` option keeps accepting the v7 spelling, and truncated cells now end with `…` rather than `...`.
- Replaced the docs/ folder with cairn/, following the Cairn convention.

### Removed

- Removed `account configure` and its `edit` alias: the wizard is the single way to generate an account, and `account list` and `account check` cover inspection. **Breaking.**

### Fixed

- Fixed a 404 against a CardDAV server whose discovery hands back a bare origin rather than the context root, as Fastmail does by serving contacts under `/dav/`. The client now probes `.well-known/carddav` and follows its redirect before the principal walk.
- Fixed the shared commands inventing collections and cards on the vdir backend: `card update` on an unknown id created the file, `card create` on an unknown addressbook created the collection, and an unknown addressbook listed as an empty table. Each fails by name now.
- Fixed `card update` creating a card on CardDAV when the given id does not exist, a WebDAV `PUT` being create-or-replace. The update now reads the card first and guards the write with its ETag; passing your own `--if-match` skips that read.
- Fixed `addressbook update` reporting success when the server changed nothing, and `--description ""` / `--color ""` doing nothing although the help documents `""` as the way to clear a property.
- Fixed the Nix package shipping no manual pages and no shell completions, its install step passing the output directory where a command name was expected.

## [0.1.0] - 2025-10-24

### Added

- Added the CardDAV backend, over an I/O-free client with rustls and native-tls support.
- Added the vdir backend, one directory per addressbook.
- Added the `addressbooks` and `cards` command families, each with `list`, `read`, `create`, `update` and `delete`.
- Added the multi-account TOML configuration, its secrets read from a shell command or a raw value.

## [root] - 2025-01-12

### Added

- Init repository

[Unreleased]: https://github.com/pimalaya/cardamum/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/pimalaya/cardamum/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/pimalaya/cardamum/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/pimalaya/cardamum/compare/root...v0.1.0
