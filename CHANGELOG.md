# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-11-25

Cardamum 1 is the first stable release: a low-level CLI over a unified contacts API, the same commands on CardDAV, JMAP, Microsoft Graph, Google People, vdir and pimdir, the protocol-specific ones beside them, plain text for a glance and `--json` for scripts, frontends and AI agents. The offline side is the pimdir store Neverest syncs, which Cardamum reads and queues its writes into, and card editing is left to a composer such as tcard.

The configuration wizard and the DNS discovery of a bare domain are gone, and a 0.x configuration is not carried over: start over from config.sample.toml. A pimdir store an earlier neverest wrote is refused `store-version`, never migrated.

### Added

- Added `card search <QUERY>...`, searching the account's cards in the pimdir query language on every backend, and its MCP tool `card_search`.

  pimdir answers from the store's index, the other backends match every card of the scope. A term only pimdir answers (`tag:`, `label:`, `property:`...) is refused elsewhere coded `query-unsupported`. `--sort` takes `title`, the default, or `relevance` where the backend ranks.

- Added `pimdir search`, searching the account's cards through the search index the sync engine builds.

  A hit carries the `card list` row, `matched` and a snippet: `{hits, nextPage, coverage, errors}` under `--json`. `coverage` says what the answer could not see (cards not downloaded or not indexed, an index behind the store), an `Incomplete:` line in text.

  It pages by `--page-token`, taking back the token `nextPage` holds, so a card arriving between two pages neither repeats nor hides another.

- Added `pimdir count`, counting the cards a pimdir query matches, with the same coverage.
- Added `pimdir status`, reporting the store behind the account: its address books with their downloaded and queued cards, the writes the sync engine parked, and the cards a server refused for good.
- Added `pimdir queue list`, `show` and `cancel`, listing, inspecting and retracting the writes queued for the next sync and the rows the sync engine parked.

  A row is named by its id or its handle `<store-id>.<incarnation>.<id>`, a handle of another store refused coded `store-reset`. `queue list --json` prints `{actions, parked, errors}`; `queue show` says whether a row is `pending`, `parked`, `unconfirmed`, `applied` or `unknown`.

  `cancel` asks first, and is refused with no terminal and no `--yes` coded `confirmation-required`, or while a sync holds the row.

- Added `account capability`, listing what an account serves of the unified API without connecting, `{account, capabilities: [{name, backend, level, detail}]}` under `--json`.

  The unified commands refuse a verb or a flag the backend does not serve before connecting, coded `capability-missing`.

- Added `repl`, a session running the CLI's commands over one account, the backend connecting once.
- Added `mcp`, a Model Context Protocol server over stdio exposing the unified API as tools, the writes listed under `--allow-write` only.
- Added a stable `code` to the `--json` failure object, `null` when none applies, and its schema `cardamum-error`.

  Accounts and command line: `account-missing`, `account-default-missing`, `account-default-several`, `addressbook-default-missing`, `backend-not-built`, `capability-missing`, `usage-invalid`, `stdin-taken`, `confirmation-required`.

  Address books and cards: `addressbook-not-found`, `addressbook-duplicate`, `card-not-found`, `card-unreadable`, `card-invalid`, `body-pending`, `precondition-failed`, `item-duplicate`, `page-token-invalid`, `request-origin`, and from a server's refusal `permission-denied`, `quota-exceeded`, `rate-limited`.

  pimdir store: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-exposed`, `store-busy`, `store-not-current`, `store-owned`, `store-staging`, `store-in-flight`, `store-reset`, `store-account-ambiguous`, `store-account-unknown`, `sync-version`, `lock-replaced`.

  pimdir writes and queue: `unsupported`, `action-malformed`, `no-performer`, `ambiguous-performer`, `flag-invalid`, `conflict-unread`, `tag-refused`, `row-not-found`.

  Search index and queries: `index-missing`, `index-outdated`, `index-version`, `index-fts5-missing`, `index-zone-unknown`, `query-refused`, `query-unanswerable`, `query-unsupported`.

- Added `errors` to `addressbook list` and `card list`, one item error `{id, code, error, sources}` per entry the listing could not read, described by the schema `cardamum-item-error`.

  The other entries are listed and the command exits 3. The text ends with one `Error:` line each, and the MCP `card_list` and `addressbook_list` tools return them with `isError: true`.

- Added the common fields of a card to the `card list` and `card read` JSON: `uid`, `fullName`, `givenName`, `familyName`, `emails`, `phones`, `organization`, `organizationUnits`, `title` and `note`.
- Added `default` to each address book of `addressbook list --json`, `true` on the one the server names the default.

  JMAP, Microsoft Graph, Google People and pimdir state one; CardDAV and vdir name none, always `false`.

- Added `nextPage` to `card list --json`, `null` on the last page, and `--page-token` to take it back.
- Added `nextPage` to `jmap contact-card query --json`, the position the next page starts at, and `--page-token` to take it back.
- Added `--if-match` to `card delete`, and to `card update` on Microsoft Graph and JMAP.

  CardDAV sends it as `If-Match`, pimdir checks it against the card's `etag`, Graph and JMAP compare it with the version read just before the write, and vdir and People refuse it. A mismatch fails `precondition-failed` before anything is sent or queued.

- Added `--id <ID>` to `addressbook create`, the vdir directory or CardDAV URL segment, `<NAME>` then the display name alone; the other backends refuse it coded `capability-missing`.
- Added `carddav rmcol <ADDRESSBOOK>`, deleting an address book collection.
- Added `client.cert` and `client.key` to every `tls` table, a PEM client certificate and its key presented to a server asking for one (mutual TLS).
- Added the account-level `tls` table, each backend's `tls` taking from it what it leaves unset.
- Added `carddav.alpn`, the ALPN identifiers offered to a CardDAV server, `["http/1.1"]` by default.
- Added `notBuilt` to `account list --json`, the account's backends this build leaves out, marked `(not built)` in the text.
- Added `accounts`, a hidden alias of `account`.
- Added a welcome on a first run: a terminal meeting no configuration shows the commands creating one from the shipped sample.
- Added config.sample.toml to releases and packages, installed as share/doc/cardamum/config.sample.toml.
- Added a `Backends:` section to the `--help` of each `addressbook` and `card` command, naming which backend serves what.

### Changed

- **BREAKING**: Changed the pimdir backend to read stores of pimdir storage-v1 and search indexes of search-v1 alone, through io-pimdir 1.0.0.

  A store an earlier neverest wrote is refused coded `store-version`, never migrated: drain its queue with that neverest, move it aside and let a current neverest sync a new one.

- **BREAKING**: Changed every write to print a receipt under `--json`, where most printed `{"message"}`.

  `card add`, `card delete`, `addressbook update` and `addressbook delete` print `{id, queued}`, `card update` `{id, keptProperties, queued}` and `addressbook create` `{id, name, queued}`. `queued` is `true` on pimdir, whose text says the write waits for the next sync.

  The protocol writes print `{id}`, `jmap address-book create` and `update` included, `carddav put` `{id, etag}`, `gpeople contact-groups members` `{id, added, removed}` and `card build -o` `{path, size}`.

- **BREAKING**: Changed `jmap contact-card copy` to print `{ids, errors}`, the copies made and one item error per card refused, exiting 3 when one was, where it failed on the first refusal.
- **BREAKING**: Changed a server's refusal in the `carddav`, `jmap`, `msgraph` and `gpeople` commands to print as the result, `{"refusal": {status, reason, message, code, retryAfter}}`, and exit 3.

  A command getting no answer (network, TLS, an unparsable body) still fails, exit 1, and the unified commands keep the failure object.

- **BREAKING**: Changed a command line refused under `--json` to print the failure object coded `usage-invalid` on stdout, exit 2, where clap printed text on stderr.

  A bare `cardamum --json`, a refused REPL line and an MCP tool call with refused arguments carry it too.

- **BREAKING**: Changed `account check --json` to `{accounts: [{account, backends: [{backend, ok, code, error, tls}]}]}`, and a failed backend to exit 3, where it exited 0.

  `tls` is the TLS configured, `null` for a local backend or a cleartext connection; the text appends `, TLS` and its parts.

- **BREAKING**: Changed `card read --json` to print the `card list` row of the card plus its vCard.
- **BREAKING**: Changed `card read`, `card build`, `carddav get`, `carddav report query`, `carddav report multiget` and `vdir item get` to carry `encoding` beside each vCard's `contents`.

  `utf-8` when `contents` is the text, `base64` when the bytes are not UTF-8 and `contents` holds their base64, where such a card failed.

- **BREAKING**: Changed every text output to plain text: no border, no colour, nothing sized to the terminal.

  `card list` and `addressbook list` print one line per item under `Account:`, then `Next page:` and the `Error:` lines; `-s 0` lists every card. `table.preset`, `table.arrangement` and every `*-color` key are gone.

- **BREAKING**: Changed the configuration to refuse an unknown key at the top level and in an account, where it ignored it.

  The `table`, top-level `account`, `addressbook.list` and `card.list.table` tables are refused as unknown keys. The block of a backend the build leaves out still loads unread.

- **BREAKING**: Changed `-c` to take one path per flag, never split on `:`, and stopped reading `CARDAMUM_CONFIG`.

  Without `-c`, every canonical path that exists is loaded, each deep-merged on top of the ones before.

- **BREAKING**: Changed `-a default` to take the account named `default`, where it took the account marked `default = true`; omit `-a` to take that one.
- **BREAKING**: Changed `--backend auto` to take the first backend the account declares, failing `backend-not-built` when the build leaves it out, where it took the next built one.

  `--backend` naming a backend left out names its cargo feature, and `account check` reports it as a failed row.

- **BREAKING**: Renamed `card create` to `card add`, its MCP tool `card_add` and its schema `cardamum-card-add`; `card create` and `card new` stay hidden aliases.
- **BREAKING**: Renamed `vdir item create` to `vdir item add`, its schema `cardamum-vdir-item-add`; `create` stays a hidden alias and `new` a visible one.
- **BREAKING**: Renamed the `gpeople` resource commands to `connections`, `contact-groups` and `other-contacts`, the singular names staying hidden aliases.
- **BREAKING**: Renamed `vdir.home-dir` to `vdir.root`; `vdir.home-dir` is refused as an unknown key.
- **BREAKING**: Changed the `gpeople` and `msgraph` listings to print the provider's cursor as `nextPage`, `null` on the last page, and to take it back through `--page-token`.

  They printed `nextPageToken` or `@odata.nextLink`. The rest of the payload stays raw, `nextSyncToken` and the delta's `@odata.deltaLink` included.

- **BREAKING**: Changed the `--json` output of `gpeople request`, `jmap request` and `msgraph request` to wrap the raw response as `{"response": …}`.
- **BREAKING**: Changed `msgraph request` and `gpeople request` to refuse a full URL off the provider's API origin, coded `request-origin`, so the account's token never leaves for another host.

  Only an argument holding `://` is a full URL: a path, a custom method (`people:createContact`) and a URL on the origin are sent as before.

- **BREAKING**: Changed the `get` commands of `msgraph`, `gpeople` and `jmap` to print each resource as the provider sent it, every property kept.

  They dropped the properties cardamum did not model and added defaults the server never sent.

- **BREAKING**: Changed `msgraph contacts create` and `update` to send the JSON as written and print what Graph answered, where a key cardamum did not model was dropped.
- **BREAKING**: Changed `jmap contact-card query` to print the query's state as `queryState`, where it printed `state`.
- **BREAKING**: Changed `carddav delete` to require its card id, where `carddav delete <ADDRESSBOOK>` deleted the whole address book; delete a collection with `carddav rmcol`.
- **BREAKING**: Changed `carddav put --if-none-match` to take `*` alone, where any value sent `*`.
- **BREAKING**: Changed `carddav report query` and `report multiget` to print an `errors` list beside `cards` and `truncated`, exiting 3 when it is not empty.

  One entry per card answered without its data, and for `multiget` per id the server does not hold (`card-not-found`), where they dropped them and exited 0.

- **BREAKING**: Changed `vdir item move` and `copy` to fail coded `item-duplicate` when the target holds an item of the same id, where the target's card was replaced.
- **BREAKING**: Changed `addressbook create` and `vdir create` naming an existing address book to fail coded `addressbook-duplicate`, where vdir rewrote its metadata and succeeded.
- **BREAKING**: Changed `card read`, `update` and `delete` on JMAP, Microsoft Graph and Google People to act only on a card the address book `-k` names holds, answering `card-not-found` otherwise.
- **BREAKING**: Changed a CardDAV `server` naming a scheme to be read as written and refused when it does not parse, where `https://` was prefixed to it.

  The `.well-known/carddav` redirect is followed over https only.

- **BREAKING**: Changed a CardDAV address book or card id to print percent-decoded when the decoding encodes back to it: `My Book` where it printed `My%20Book`.

  A segment that would not round-trip (`%2F`, `%40`, lowercase hex) keeps its served form, and an id printed before still addresses its resource.

- **BREAKING**: Changed a field flag to refuse a source holding anything after its first card, a partial second card included, where it dropped it.
- **BREAKING**: Changed the pimdir backend to count a collection of the empty kind as no address book: it is no longer listed, read or written.
- **BREAKING**: Changed an unset `pimdir.account` to be derived from the one account holding address books in the store, refused when several do, naming them.

  A `pimdir.account` naming none of the store's accounts fails naming those it holds.

- **BREAKING**: Changed a pimdir card whose body the store no longer holds to read as not downloaded: listed with a `null` preview and `etag`, `card read` failing `body-pending`.

  A sync brings the body back.

- **BREAKING**: Changed a pimdir write into an address book no source syncs to be refused coded `no-performer` before anything is staged, where it was queued and never pushed.
- Changed `card update` and `card delete` on pimdir to queue the version of the card they were made against.

  The sync engine parks a write over a card a sync changed since, or over a conflicted card, rather than applying it; `pimdir status` and `pimdir queue list` show it.

- Changed a pimdir write to be refused before it is queued when a source of the store does not support it, naming the capability, the source and why.
- Changed a pimdir card write to drop every `X-PIMDIR-` property from the card before it is queued.
- Changed `account check` on pimdir to probe the store's staging lock too, writing nothing, so a store a sync holds fails at once `store-busy`.
- Changed `addressbook delete` and `addressbook update` to take the address book as a positional, `-k/--addressbook` staying a hidden alias.
- Changed `vdir create <NAME>` to `vdir create <ID>`, adding `-n/--display-name <NAME>` for a display name apart from the directory.
- Changed `carddav.auth` to be optional: a block naming none sends no `Authorization` header, as `auth = "none"` does.
- Changed `msgraph contacts list`, `msgraph contact-folders list` and `child-folders` to show `--top` long-only, `-s` staying a hidden alias.
- Changed the aliases `rm` of `addressbook delete` and `card delete`, `folders` of `msgraph contact-folders` and `carddav create` of `carddav mkcol` to hidden ones.
- Changed `account list` to name an account's backends in the order `auto` resolves them (vdir, pimdir, carddav, jmap, msgraph, gpeople), its text marking the default account `default`.
- Changed the `carddav discover` text to print `Server:`, `Principal:` and `Addressbook home-set:` as aligned fields.
- Changed the text of `carddav propfind`, `vdir list` and `msgraph contacts` to print the fields an item holds, every email and phone of a Graph contact included, and no empty line otherwise.
- Changed `card list` to read only the page it prints on pimdir, JMAP, Microsoft Graph and Google People, where it read the whole address book.
- Changed the version of a JMAP card (`etag`, what `--if-match` compares) to the BLAKE3 digest of its JSON, stable across builds.
- Changed `carddav put` to print the id the write answered, as a listing shows it, where it echoed its argument.
- Changed a CardDAV address book outside the home set to be listed and addressed by its path rather than its last segment.
- Changed a JMAP set error of a type cardamum does not model to be named as the server sent it, where it read `unknown`.
- Changed a vdir listing to hold regular files only and to leave out a card removed while it lists.
- Changed piped `--json` output to one value per line, and each `json-schema` schema to carry an `$id` naming the contract version.
- Changed `--log-file` to refuse a path naming a link to an existing file.
- Changed an HTTP response whose head passes 64 KiB or whose body passes 256 MiB to fail the exchange.
- Changed config.sample.toml to Himalaya's shape: one `[accounts.example]` table, one commented key per line, alternatives in brackets, one section per backend, the provider sections left to the README's recipes.

### Removed

- **BREAKING**: Removed the `configure` command, its `wizard` alias and the `wizard` cargo feature.

  A bare `cardamum` prints the help, and a command meeting no configuration fails naming the path it looked at and the sample.

- **BREAKING**: Removed the `discover` route of the `[carddav]` block, resolving a bare domain through PACC and DNS SRV and TXT records.

  Name the server in `carddav.server` (a bare host still goes through its `.well-known/carddav`), or the address book home in `carddav.home` as Google needs. `carddav.discover` is refused as an unknown key.

- **BREAKING**: Removed `fnValue`, `email` and `tel` from the card rows under `--json`: read `fullName`, `emails` and `phones`.
- Removed IDNA for server and proxy hosts: a non-ASCII host is refused, naming it, its Punycode form (`xn--...`) working.

### Fixed

- Fixed the vdir backend reaching out of its root: an address book id that is empty, `.`, `..` or holds a separator, which let `addressbook delete -k ..` delete the root's parent, is now `addressbook-not-found`.
- Fixed a CardDAV address book or card id that is empty or holds an empty, `.` or `..` segment reaching another resource, the server root or the home set: it fails before connecting.
- Fixed `card update` on Microsoft Graph and JMAP failing with no `--if-match`, where the version read was sent as a guard those backends do not take.
- Fixed the card commands on Google People acting on a group that does not exist: they fail `addressbook-not-found` before writing anything.
- Fixed `card list -k <addressbook>` on CardDAV, JMAP and Microsoft Graph failing with the server's bare refusal for an address book it does not hold, and on Google People listing it as empty.
- Fixed `card add` on pimdir queueing a card whose `UID` the address book already holds, which the sync then parked: it fails `item-duplicate`.
- Fixed `card update` on Google People erasing what a vCard cannot show of the fields it changes: a further organization, a relation of another type, a custom type.
- Fixed `card update` on Google People reporting as dropped the properties its stash keeps.
- Fixed `card add` and `card update` on Google People leaving out a property longer than 8 KiB without a warning.
- Fixed a year-less birthday (`BDAY:--MMDD`) on Google People being stashed instead of becoming a People birthday ([#26](https://github.com/pimalaya/cardamum/issues/26)).
- Fixed `gpeople connections update` replacing the `etag` its JSON names with the person's current one, letting a stale write overwrite a newer person.
- Fixed `gpeople other-contacts copy` failing on every contact with `HTTP 400 INVALID_ARGUMENT`.
- Fixed `gpeople contact-groups list --sync-token` failing on the `totalItems: -1` Google answers with.
- Fixed the `addressbook delete` help saying a Google People group is deleted with its contacts: the group alone goes.
- Fixed `addressbook list` on Microsoft Graph showing the top-level contact folders alone: every folder is listed whatever its depth.
- Fixed the Microsoft Graph listings following a next page link off the Graph origin with the account's token: such a link is refused.
- Fixed `addressbook update --description ""` on JMAP leaving the description in place.
- Fixed `jmap contact-card get` and `jmap address-book get` dropping the ids the server found nothing for: each is an `errors` entry, the command exiting 3.
- Fixed `vdir rename` onto an existing collection failing with a raw OS error or replacing an empty directory: it fails `addressbook-duplicate`, nothing moved.
- Fixed the card verbs on vdir reading an `.ics` item as a card: they answer `card-not-found`.
- Fixed pimdir recognising an address book only by the exact kind `text/vcard`: a kind carrying parameters counts.
- Fixed the name, email and phone columns of `card list` misreading a property with parameters, a folded line, a grouped property (`item1.EMAIL`) and `TELEX`.
- Fixed `card list` printing vCard 2.1 quoted-printable values undecoded, a foreign `CHARSET` now transcoded too.
- Fixed a name flag of `card update` dropping the parameters of `N` and writing a 2.1 quoted-printable family name back as literal text.
- Fixed `--phone tel:+33...` writing `TEL;TYPE=tel:+33...`: a `tel:` or `mailto:` scheme is no type.
- Fixed a field flag writing CRLF lines into an LF card.
- Fixed `card build`, `card add` and `card update`, and the Google People and Microsoft Graph cards, leaving vCard lines longer than 75 octets unfolded.
- Fixed `card read` and the write confirmations printing control characters as is, which could drive the terminal: `--json` keeps the bytes.
- Fixed `card read` and the `request` text ending without a newline.
- Fixed a CardDAV listing trimming a card's final CRLF, so it matches the body `card read` and `carddav get` return.
- Fixed a CardDAV answer that is not one whole multistatus document listing fewer cards instead of failing, and a weak ETag being cut.
- Fixed a CardDAV relative id being sent unencoded, a `\` read as a separator.
- Fixed a CardDAV request failing with `reached unexpected EOF` when the server had closed the connection: it is sent again once on a fresh one.
- Fixed `carddav report raw` counting a card two bytes short in `dataBytes` when the server escapes its final CRLF.
- Fixed a connection closing mid-body spinning rather than failing.
- Fixed `account check` passing a CardDAV account naming `carddav.home` whatever its credential: the home is sent one `PROPFIND`.
- Fixed a credential command prompting for a password (`systemd-ask-password`) hanging the run: it reads the terminal when there is one.
- Fixed a unified command spawning its secret commands again when it reopened its backend after the composer.
- Fixed a shell composer failing to open its draft on Windows.
- Fixed the composer's file being readable by other users under a permissive umask: it is created 0600 on Unix, a file or link planted at its path refused.
- Fixed install.sh saving an HTTP error page as the archive.

### Security

- Bumped rustls-webpki to 0.103.16 and webpki-root-certs to 1.0.11, fixing RUSTSEC-2026-0382 and RUSTSEC-2026-0383.

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

[Unreleased]: https://github.com/pimalaya/cardamum/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/pimalaya/cardamum/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/pimalaya/cardamum/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/pimalaya/cardamum/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/pimalaya/cardamum/compare/root...v0.1.0
