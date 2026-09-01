# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added a composer, the command `card.composer` names, opened by `-i/--interactive` on `card create` and `card update`: it is spawned on the path of a temporary vCard file with every stream inherited, and what it leaves there is the decision. Changed bytes are the card, an emptied or untouched file is an edit given up on, and a non-zero exit is a failure. Any command works as long as it blocks until the edit is done: `code --wait`, not `code`. `--composer <COMMAND>` overrides it for one run. What a composer wrote is still checked against its version's RFC contract, printing its violations and offering a re-edit, and a failed write keeps the temporary file and names it.
- Added `card build`, the create pipeline stopped before the write: it applies the same source, field flags and composer and prints the vCard. It reaches no backend and reads no configuration unless `-i` needs the configured composer, so `card build --full-name "Jane Doe"` runs on a machine holding none, and `card read <ID> | card build --title CTO -` previews an update. `-o/--output <PATH>` captures it, `-i` owning stdout, and an abandoned build prints nothing at exit 0. It checks what `card create` checks, a card built from flags with no source being refused when it is not valid, so `build | create -` is no way past that guard.
- Added `card.composer`, a shell line or an argv list, at the top level and per account.
- Added the vCard field flags to `card build`, `card create` and `card update`: `--full-name`, `--given-name`, `--family-name`, `--middle-name`, `--name-prefix`, `--name-suffix`, `--nickname`, `--organization`, `--title`, `--birthday`, `--email work:a@b`, `--phone cell:+1-555-0100`, `--url`, `--note` and `--uid`. A flag replaces every instance of the property it names and leaves every other line byte for byte; `--given-name` and `--family-name` repeat, merge into the card's existing `N` rather than clearing its other components, and are named for the role rather than the position. They are a convenience over the common fields, not the whole of vCard: the composer is the complete surface, and `ADR` is deliberately not a flag.
- Added the `json-schema` command (aliased `json-schemas`), printing the JSON Schema of a command's `--json` output or writing one file per command into a directory. Every command returning data now hands the printer a named output type.
- Added `carddav.auth = "none"`, for a server that asks for no credentials; the wizard offers it where discovery advertised no scheme at all.
- Added `--fallback` to `carddav report sync`, enumerating the addressbook with a Depth 1 `PROPFIND` for a server implementing no `sync-collection` REPORT.
- Added the truncation report to `carddav propfind <addressbook>`, which used to drop the fact that the server cut its listing short.

### Changed

- **BREAKING**: every `--json` key spelling more than one word is camelCase, where it used to be snake_case: `addressbookId`, `fnValue`, `keptProperties`, `syncToken`. A key a provider owns keeps that provider's spelling, so `@odata.nextLink`, `nextPageToken`, `contactGroups` and the JMAP `list` are untouched, and the TOML configuration stays kebab-case.
- **BREAKING**: `addressbook create`, `card create`, `card update` and `vdir item create` emit their result under `--json` instead of a prose message: `{"id"}`, and `{"id", "keptProperties"}` for the update. Terminal output is unchanged, word for word.
- **BREAKING**: the pimdir backend no longer takes the store's owner role, and `pimdir.source` is gone. It reads through a lock-free reader and stages each write as one queue action through a short-lived producer, so a listing runs beside a sync instead of failing against it and a staged write reads back before the sync applies it. The store must already exist, creating one being the sync engine's job.
- **BREAKING**: the pimdir backend refuses `addressbook create`, `update` and `delete`, declaring a collection being an owner write, and `card create` reports the card's link id, a queued create having no store-assigned id until the sync applies it.
- **BREAKING**: renamed `completions` and `manuals` to `completion` and `manual`, the plural staying as a hidden alias. A command mirroring a vendor API resource keeps that API's spelling, so `people contact-group members`, `jmap address-book changes` and `jmap contact-card changes` are unchanged; the `msgraph` family is aligned onto Graph, `contact-folder` and `contact` becoming `contact-folders` and `contacts`. Every counterpart spelling stays as a hidden alias.
- `card create` and `card update` take the source, the field flags and `-i` in that order, so `card create --full-name "Jane Doe" -i` opens the composer on a card already carrying the name. Neither requires a vCard any more: a create with none mints one carrying a fresh `UID` at `--vcard-version`, and an update with none reads the card first and sends the version the backend answered as `If-Match`, so an edit that took a minute no longer silently overwrites a write that landed during it. An explicit `--if-match` still wins.
- The backend connection is opened by the call that needs it instead of when the client is built, so a command that never reaches the network opens no socket and an interactive edit holds none open while the editor is up. A server closes an idle connection, so a write landing after a long edit used to fail with `unexpected end of file` for a card that was perfectly good.
- Resolved an account's credentials once, so a command two of its backends name is spawned once: an account naming one `pass` entry from its `carddav` and `jmap` tables used to pay two key unlocks. A command is compared as the configuration wrote it, so a shell line and its argv spelling stay two commands.
- Reworded the card argument of `card read`, `card update` and `card delete`, which called itself a card `UID`: it is the backend's own identifier, the one `card list` reports.

### Fixed

- A vCard source carrying nothing but whitespace is refused, naming where it was read from, instead of read as a card: `printf '' | card create -k <AB> -` used to hand the backend an empty body and report a clean success.
- A source holding several vCards is refused when a field flag is set, instead of keeping the first card and dropping the rest: a flag rewrites the card the parser reads first, so `card create --title CTO two-cards.vcf` used to write one card and lose the other at exit 0. With no flag the source still passes through as it was written.
- `carddav.tls.cert` expands a `~` or an environment variable, a certificate written home-relative having been looked for in the current directory. `pimdir.root` moved its expansion onto the field for the same reason.
- The pimdir backend no longer accepts a collection of any kind as an addressbook. A sync engine caches mail, calendars and contacts in one store and only the listing narrowed them to `text/vcard`, so `card list -k imap/INBOX` printed a mailbox's messages as blank contacts. A wrong id is refused naming the addressbooks the account holds.
- A card no longer vanishes from the pimdir backend when another card of the same addressbook carries its `UID`. RFC 6352 requires that `UID` to be unique and servers hand over duplicates anyway, most often after a repeated import; the store keys the second copy apart, so both cards list, read and act under their own ids.
- `card list` no longer leaves the `TEL` column empty for a card writing its phone above its mail. The preview chained its three property reads, so a line that was not an `EMAIL` never reached the `TEL` read. Every backend was affected.
- The pimdir backend links a card it stages under the bare `UID` the sync engine uses, rather than `uid:<UID>`, which would have stored the card twice and synced it as a duplicate contact.
- The pimdir backend names a body it writes with the hash the store records rather than a digest of its own choosing, which put the body where no read would look for it.

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
