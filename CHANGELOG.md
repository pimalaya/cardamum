# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `card build`, which builds a vCard from the same source and the same field flags as `card create` and prints it instead of sending it anywhere.

  It reaches no backend and resolves no account, so it runs on a machine holding no configuration: `cardamum card build --full-name "Jane Doe" --email work:jane@corp.example` shows what those flags produce. Both write verbs take `-` as a source, so it pipes into them, and `cardamum card read <ID> | cardamum card build --title CTO -` previews an update. It checks what `card create` checks: a card built from flags with no source is refused when it is not a valid vCard, a vCard given as a source passes through untouched.

- Added `card.composer`, at the top level and per account: the command a card is edited through.

  It is spawned on the path of a temporary vCard file it edits in place, with stdin, stdout and stderr all inherited and nothing captured, so a terminal editor gets a real terminal. The path is appended as the last argument, so `card.composer = "tcard edit"` runs `tcard edit <PATH>`. Any command works as long as it blocks until the edit is done: use `code --wait`, not `code`.

- Added `-i/--interactive` to `card create` and `card update`, which opens the card in that composer before writing it, and `--composer <COMMAND>` to override the configured one for a single invocation.

  What the composer wrote is checked against its version's RFC contract first: a card that does not pass has its violations printed and offers a re-edit. Then a menu asks what to do with the card: `Save`, `Preview`, `Edit again` or `Abort`. Aborting keeps the file and names it unless nothing was typed. A card the composer left without a `BEGIN:VCARD` line is reported and the menu drops `Save`.

  On an update with no vCard source the card is read first, and the version the backend answered is sent as `If-Match`, so an edit that took a minute no longer silently overwrites a write that landed during it. An explicit `--if-match` still wins.

- Added the vCard field flags to `card create` and `card update`: `--full-name`, `--given-name`, `--family-name`, `--middle-name`, `--name-prefix`, `--name-suffix`, `--nickname`, `--organization`, `--title`, `--birthday`, `--email work:a@b`, `--phone cell:+1-555-0100`, `--url`, `--note` and `--uid`.

  A flag sets the property it names, replacing every instance the card carried, and leaves every other line byte for byte. `--given-name` and `--family-name` repeat, each component of `N` being a comma-separated list per RFC 6350 section 6.2.2, and merge into the card's existing `N` rather than clearing its other components. They are named for the role rather than the position: which of the two is written first is what varies between cultures. They are a convenience over the common fields rather than the whole of vCard: the composer is the complete surface, and `ADR` is deliberately not a flag.

  The source, the flags and `-i` stack in that order, so `card create --full-name "Jane Doe" -i` opens the composer on a card already carrying the name. `card create` no longer requires a vCard: with none it mints one carrying a fresh `UID`, at `--vcard-version`.

- Added the `json-schema` command, aliased `json-schemas`, which prints the JSON Schema of a command's `--json` output or writes one file per command into a directory.

  Every command returning data now hands the printer a named output type describing what it emits, so a script consuming `--json` has a schema rather than the prose in a help page. A command only confirming a write still prints a plain message, and none is registered.

- Added `carddav.auth = "none"`, for a CardDAV server that asks for no credentials.

  The wizard offers it where discovery advertised no authentication scheme at all. Every existing spelling of `carddav.auth` parses unchanged.

- Added `--fallback` to `carddav report sync`, which enumerates the addressbook with a Depth 1 `PROPFIND` for a server implementing no `sync-collection` REPORT.
- Added the truncation report to `carddav propfind <addressbook>`, which used to drop the fact that the server cut its listing short.

### Fixed

- The backend connection is opened by the call that needs it, instead of when the client is built.

  A command that never reaches the network no longer opens a socket, and, more to the point, an interactive edit holds none open while the editor is up. A server closes an idle connection, so a write landing after a long edit used to fail with `unexpected end of file` for a card that was perfectly good.

### Changed

- **BREAKING**: every `--json` key spelling more than one word is camelCase, where it used to be snake_case.

  A listing now answers `addressbookId` and `fnValue`, an update `keptProperties`, a sync `syncToken`: the convention of the wire formats the backends already speak, and one no `jq` path has to quote. A key a provider owns keeps that provider's spelling, so `@odata.nextLink`, `nextPageToken`, `contactGroups` and the JMAP `list` are untouched. The TOML configuration is not concerned and stays kebab-case.

- **BREAKING**: `addressbook create`, `card create`, `card update` and `vdir item create` emit their result under `--json` instead of a prose message.

  The four carried data inside `{"message": "..."}`: the identifier the backend assigned, and for `card update` the vCard properties the server would not let go. They now emit `{"id"}`, and `{"id", "keptProperties"}` for the update, which is what the new schemas describe. Terminal output is unchanged, word for word.

- **BREAKING**: the pimdir backend no longer takes the store's owner role, and `pimdir.source` is gone from the configuration.

  It reads through a lock-free reader and stages each write as one queue action through a short-lived producer, which is what the format asks of a process that is not the store's owner.

  A listing therefore runs beside a sync instead of failing against it, and a staged write reads back before the sync applies it. Nothing attributes a write to a replica source any more, so `pimdir.source` has no meaning and an account still carrying it is an error.

  The pimdir store must now already exist: creating one is the sync engine's job, and a root naming no store says so instead of listing an empty set of addressbooks.

- **BREAKING**: the pimdir backend refuses `addressbook create`, and `card create` reports the card's link id.

  Declaring a collection is an owner write, and a collection no sync knows about is one no sync would carry, so all three `addressbook` writes now refuse on pimdir. A queued create has no store-assigned id until the sync applies it, so there is none to report.

- **BREAKING**: renamed `completions` and `manuals` to `completion` and `manual`, the plural staying as a hidden alias.

  A command mirroring a vendor API resource keeps that API's spelling, so `people contact-group members`, `jmap address-book changes` and `jmap contact-card changes` are unchanged, each gaining a hidden singular alias.

  The `msgraph` family, singular where Graph is plural, is aligned onto Graph: `msgraph contact-folder` and `contact` become `contact-folders` and `contacts`, joining the `child-folders` that already sat under the first. Every singular spelling stays as a hidden alias, where it used to be shown beside the plural.

- Resolved an account's credentials once, so a command two of its backends name is spawned once.

  `account check` and the wizard's connection test reach every backend the account configures, and each of them used to spawn its own credential command: an account naming one `pass` entry from its `carddav` and `jmap` tables paid two key unlocks for one entry. Both now resolve the whole account through one resolver, which spawns each distinct command once and hands its value to every backend naming it. A command is compared as the configuration wrote it, so a shell line and its argv spelling stay two commands.

- Reworded the card argument of `card read`, `card update` and `card delete`, which called itself a card `UID`.

  It is the backend's own identifier, the one `card list` reports, and a `UID` names no card on its own.

### Fixed

- Fixed `carddav.tls.cert` never expanding a `~` or an environment variable, so a certificate written home-relative was looked for in the current directory.

  The `pimdir.root` expansion moved off its call site onto the field for the same reason: a path is expanded as the configuration is read, so no reader of the field can forget to.

- Fixed the pimdir backend accepting a collection of any kind as an addressbook.

  A sync engine caches mail, calendars and contacts in one store, and only the listing narrowed them to `text/vcard`, so `card list -k imap/INBOX` printed a mailbox's messages as blank contacts and a create would have staged a vCard into it.

  An addressbook id now names an addressbook or nothing, and a wrong one is refused naming the addressbooks the account holds.

- Fixed a card vanishing from the pimdir backend when another card of the same addressbook carried its `UID`.

  RFC 6352 requires that `UID` to be unique and servers hand over duplicates anyway, most often after a repeated import. The store keys the second copy apart now, so both cards list, read and act as ordinary cards under their own ids.

- Fixed `card list` leaving the `TEL` column empty for any card writing its phone above its mail.

  The preview chained its three property reads, so a line that was not an `EMAIL` never reached the `TEL` read. Every backend was affected.

- Fixed the pimdir backend linking a card it stages under `uid:<UID>` where the sync engine uses the bare `UID`.

  It would have stored the card twice and synced it as a duplicate contact. The derivations now come from io-pimdir's own conventions.

- Fixed the pimdir backend naming a body it writes with a digest of its own choosing instead of the hash the store records, which put the body where no read would look for it.

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

[Unreleased]: https://github.com/pimalaya/cardamum/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/pimalaya/cardamum/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/pimalaya/cardamum/compare/root...v0.1.0
