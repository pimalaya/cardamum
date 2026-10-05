# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `default` to every address book in `addressbook list --json` (pimdir draft-04, io-pimdir `ef8eae0`): `true` on the one the server names the default, read on the pimdir backend from the role the sync engine recorded (`collections.role`), `false` on the other backends for now.
- Added the `wizard` cargo feature, on by default, gating the interactive configuration: the `configure` command and the offer a first run makes. A build without it drops the prompts and the dependencies only they use, and a missing configuration points at the documented sample instead.
- Added the common fields of a card to the `card list` and `card read` JSON: `uid`, `fullName`, `givenName`, `familyName`, every address in `emails` and number in `phones`, `organization`, `organizationUnits`, `title` and `note`, read through the decoded vCard.
- Added `--if-match` to `card delete`: CardDAV sends it as `If-Match`, pimdir checks it against the card's version, and the backends that cannot gate a delete refuse it.
- Added a stable `code` to the `--json` error output for failures a caller acts on: `body-pending`, `precondition-failed`.

### Changed

- Changed io-pimdir to the git revision `8c83c04` (pimdir draft-04), the one neverest, himalaya and calendula pin, so the set shares one io-pimdir and one store schema.
- Changed the pimdir `update_card` and `delete_card` to honour `--if-match` against the `etag` they report, the store's hash of the card's body: a write naming a version the card no longer has is refused before anything is queued, with an error starting `Precondition failed:`.
- Changed the pimdir writes to be refused before they are queued when a source of the store does not support them, naming the capability, the source and why (pimdir draft-03, STORAGE §15.6). A write a source supports only in part logs a warning.

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
