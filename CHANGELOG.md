# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `default` to every address book in `addressbook list --json` (pimdir draft-04): `true` on the one the server names the default, read on the pimdir backend from the role the sync engine recorded (`collections.role`), `false` on the other backends for now.
- Added the common fields of a card to the `card list` and `card read` JSON: `uid`, `fullName`, `givenName`, `familyName`, every address in `emails` and number in `phones`, `organization`, `organizationUnits`, `title` and `note`, read through the decoded vCard.
- Added `--if-match` to `card delete`: CardDAV sends it as `If-Match`, pimdir checks it against the card's version, and the backends that cannot gate a delete refuse it.
- Added a stable `code` to the `--json` error output for failures a caller acts on: `body-pending`, `precondition-failed`, and on pimdir the code io-pimdir names for a store or a write it refuses: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-exposed`, `store-busy`, `lock-replaced`, `unsupported`, `item-duplicate`, `action-malformed`.
- Added `pimdir status`, reporting the local pimdir store behind the account: its address books, how many of their cards are downloaded and how many creations are queued, and every write the sync engine parked rather than applied, with its queue row and the reason.
- Added `pimdir search`, searching the account's cards in the pimdir query language through the store's search index.

  The sync engine builds and refreshes the index, cardamum only reads it. A hit carries the `card list` fields, its address book, what matched and a snippet; `-k` narrows to one address book, `--sort name|relevance` orders. The answer carries its coverage, `{complete, bodiless, missing, unparseable, unindexed, behind, filling}` under `--json` and one line under the list otherwise: the cards whose body is not local or whose body the index could not read or parse, each counted when a term of the query reads card bodies, and the index lagging the store.
- Added `pimdir count`, counting the cards a pimdir query matches, with the same coverage.
- Added the `--json` error codes `index-missing`, `index-outdated`, `index-version`, `index-fts5-missing`, `query-refused` and `query-unanswerable`, raised by `pimdir search` and `pimdir count`.
- Added a welcome for a first run: a person at a terminal meeting no configuration is shown the commands creating it from the shipped sample, then the first command to run.
- Added `nextPage` to `card list --json`, the next page number while cards follow and `null` on the last page, and `--page-token`, taking it back.
- Added `carddav.alpn`, the ALPN identifiers offered to a CardDAV server, `["http/1.1"]` by default.
- Added config.sample.toml to releases and packages, installed as share/doc/cardamum/config.sample.toml.
- Added `account capability`, listing what the backend an account resolves to serves of the unified API, with each level and its detail, without connecting.
- Added a `Backends:` section to the `--help` of each `addressbook` and `card` command, naming which backend serves what.
- Added the `--json` error code `capability-missing`.
- Added `repl`, a session running the CLI's commands over one account, the unified client connecting once.
- Added `mcp`, a Model Context Protocol server exposing the unified API as tools, the writes listed under `--allow-write` only.
- Added the `--json` error code `stdin-taken`, for a command reading stdin inside a session.
- Added `code` to each backend of `account check --json`, the stable code of its failure (`store-busy`, `store-uncreated`...), `null` when it passed or the failure carries none.
- Added MIGRATION.md, the changes to act on coming from 0.3.

### Changed

- Changed the unified commands to refuse a verb or a flag the account's backend does not serve before connecting, with `capability-missing`, rather than once connected with an uncoded error.
- Changed every dependency to name the features it needs, and base64 to base64-codec.
- Changed config.sample.toml to one `[accounts.example]` table and one commented section per backend or provider, each tested to parse.
- **Breaking:** changed `-c` to take one path per flag, never split on `:`, and stopped reading `CARDAMUM_CONFIG`.
- **Breaking:** changed a command line the grammar refuses under `--json` to print on stdout the failure object coded `usage-invalid`, where clap printed its text on stderr; it still exits 2, the family exit contract (pimalaya-cli) keeping 2 for a refused command line and 3 for a partial outcome. A refused REPL line and an MCP tool call whose arguments the grammar refuses carry `usage-invalid` too.

  Without `-c`, every canonical path that exists is loaded, each deep-merged on top of the ones before.
- **Breaking:** changed every text output to plain text: no border, no colour, nothing sized to the terminal.

  `card list` prints one line per card, the account and the addressbook above, the page and the next one below. `-s 0` lists every card. `table.preset`, `table.arrangement` and every `*-color` key are gone, refused under `card.list` and `addressbook`.
- Changed account resolution to refuse several accounts marked `default`, naming them.
- Changed the `pimdir` feature to build io-pimdir's search, adding ical-rs, mime-rs and ariadne to the binary.
- **Breaking:** changed every write printing a confirmation to print a receipt instead of `{"message": ...}` under `--json`: `addressbook update|delete` and `card delete` print `{id, queued}`, the item acted on and whether the write waits for a sync; `carddav mkcol|proppatch|delete`, `vdir create|rename|delete`, `vdir item update|copy|move|delete`, `jmap address-book destroy`, `jmap contact-card destroy`, `msgraph contacts delete`, `msgraph contact-folders delete` and the `gpeople` deletes print `{id}`, a renamed collection's new id, as do `jmap address-book update` and `jmap contact-card update` when the server returns no object; `jmap contact-card copy` prints `{ids}`, `carddav put` `{id, etag}`, `gpeople contact-group members` `{id, added, removed}` and `card build -o` `{path, size}`. The text form keeps its sentence. Every command now has a schema under `json-schema` (pimalaya-cli 0.3).
- Bumped pimalaya-cli and pimalaya-config to 0.4, io-jmap to 0.8 and io-pim-discovery to 0.12. The `--json` failure object always carries `code`, `null` when none applies.

- Bumped rustls-webpki to 0.103.16 and webpki-root-certs to 1.0.11, fixing RUSTSEC-2026-0382 and RUSTSEC-2026-0383.
- Changed piped `--json` output to print one value per line, each schema of `json-schema` to carry an `$id` naming the contract version (`v0.3`), and added `cardamum-error`, the schema of the JSON failure object (pimalaya-cli `55b6693`). The wire objects of `gpeople`, `jmap` and `msgraph` are now described as JSON objects, and the abandoned arm of `card create` and `card update`, which `--json` never prints, is now `{}` rather than `null`.
- **Breaking:** changed the `--json` output of `gpeople request`, `jmap request` and `msgraph request` to wrap the raw response as `{"response": …}`, so the payload is an object even when the server answers no body (`null`) or an array.
- Changed `card list` on pimdir to stream the store rather than load it: it holds one batch of rows and the page it prints instead of every card body of the address book, reads the bodies of that page only, and stops once the page is full. The output is unchanged.
- Changed io-pimdir to 0.12 (pimdir storage-draft-11, sync-draft-11 and search-draft-11), io-msgraph to 0.8, io-gpeople to 0.7 and io-http to 0.7, so cardamum shares one io-pimdir and one store schema with neverest, himalaya and calendula: an empty mail date is stored as no date, a band round keeps undated mail, and opening a store creates the `items_by_sort_global` index a page across collections walks. A response whose head passes 64 KiB or whose body passes 256 MiB now fails the exchange with a typed error instead of growing the buffer without bound,, a connection closing mid-body fails rather than spinning, and People and Graph fold vCard lines over 75 octets.
- Bumped io-webdav to 0.8. An address book outside the home set is listed and addressed by its path rather than its last segment, a request the server closed the connection on is resent once, and a CardDAV multistatus is parsed 3.4 to 9 times faster, without quadratic copies and nested 64 levels at most.

  A CardDAV answer that is not one whole multistatus document fails instead of listing fewer cards, and a weak ETag is kept whole.
- Bumped io-vdir to 0.3.

  A vdir listing leaves out a card removed while it lists and holds regular files only, and an id that is empty or holds a path separator is refused.
- Changed `carddav put` to print the id the write answered, percent-encoded as a listing shows it, where it echoed its argument.
- Changed a JMAP set error of a type io-jmap does not model to be named as the server sent it, where it read `unknown`.
- Bumped vcard-rs to 0.6.

  A 3.0 or 4.0 line longer than 75 octets that `card build`, `card create` or `card update` writes or edits is now folded.

- Changed the pimdir `update_card` and `delete_card` to honour `--if-match` against the `etag` they report, the store's hash of the card's body: a write naming a version the card no longer has is refused before anything is queued, with an error starting `Precondition failed:`.
- Changed `card update` and `card delete` on pimdir to name, on the write they queue, the version `--if-match` matched (pimdir STORAGE §15.3): a write queued over a card a sync changes before applying it is parked rather than applied, and shows in `pimdir status`.
- Changed a `card update` or a `card delete` of a conflicted pimdir card to be parked by the sync engine, the card and its diverging body kept, where a delete settled the conflict: cardamum decides no conflict, a client showing it or the server does.
- Changed the pimdir writes to be refused before they are queued when a source of the store does not support them, naming the capability, the source and why (pimdir draft-03, STORAGE §15.6). A write a source supports only in part logs a warning.
- Changed `card create`, `card update` and `card delete` on pimdir to say the write is queued for the next sync instead of reporting it done; the `--json` output is unchanged.

  A queued create reports the card's link id, its `UID`, and is not listed until the sync applies it.

- **Breaking:** changed pimdir with `pimdir.account` unset to read the one account the store groups its collections under, and to refuse a store grouping several, naming them.
- Changed `account check` on pimdir to probe a producer as well as the reader, so a store whose staging lock cannot be taken fails the check rather than the first write; a sync collecting the store fails it at once with `store-busy`, and the check writes nothing into the store.
- Changed the refusal of a pimdir store cardamum cannot read to say what it asks instead of deleting the store, which loses the queued writes a resync never restores: a store of another format version is reinstalled through neverest, its queue drained first, or asks for a cardamum upgrade.
- **Breaking:** changed the pimdir backend to read a store of pimdir storage-draft-11 alone: a store any earlier neverest wrote is refused with `store-version` until neverest reinstalls it, a search index of an earlier draft with `index-version` until the sync engine rebuilds it, and a store another user can read is refused with `store-exposed`.
- Changed the `addressbook delete` refusal on pimdir to name the server route only, no longer advising to remove the store directory, which loses the queued writes.

### Removed

- Removed IDNA for server and proxy hosts: an `http` or `https` URL with a non-ASCII host is refused, its Punycode form working.
- **Breaking:** removed the `configure` command and its `wizard` alias, with the `wizard` cargo feature.

  A bare `cardamum` prints the help, and a command meeting no configuration fails naming the path it looked at and the documented sample.

### Fixed

- Fixed `card read` printing a card's control characters as is, which could drive the terminal: the text view replaces them, CRLF read as LF, and `--json` keeps the bytes.
- Fixed the write confirmations printing a server-given id as is.
- Fixed a unified command spawning its secret commands again when it reopened its backend after the composer.
- Fixed `account check` exiting 0 when a backend failed: it now prints its report and exits 3.
- Fixed pimdir accepting writes into an address book no source syncs, which no sync would ever push: they are now refused before anything is staged.
- Fixed a pimdir card whose body blob is missing reporting an `etag`, which an `--if-match` could then match.
- Fixed pimdir recognising an address book only by the exact kind `text/vcard`: a kind carrying parameters now counts.
- Fixed `card create` on pimdir reporting as queued a card whose `UID` the address book already holds, synced or queued, which the sync then parked: it is now refused with `item-duplicate`, leaving nothing in the store, and `-i` keeps the composer file.
- Fixed a pimdir `card update` made from the card it read (no vCard source, or `--if-match`) overwriting, unmerged, a server change a sync brought in before applying it: the queued update now names the body it was made against, and the sync engine parks it instead.
- Fixed `card update` on Google People erasing what a vCard cannot show of the fields it changes: a further organization, a relation of another type, a custom type, a sub-field without a vCard slot.
- Fixed `card create` and `card update` on Google People leaving out a property longer than 8 KiB without a word: a warning names it.
- Fixed `addressbook update --description ""` on JMAP leaving the description in place: it is cleared.
- Fixed the Microsoft Graph listings following a next page link off the Graph origin with the account's token: such a link is refused.

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
