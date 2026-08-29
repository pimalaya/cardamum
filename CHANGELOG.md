# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `--fallback` to `carddav report sync`, which enumerates the addressbook with a Depth 1 `PROPFIND` for a server implementing no `sync-collection` REPORT.
- Added the truncation report to `carddav propfind <addressbook>`, which used to drop the fact that the server cut its listing short.

### Changed

- **BREAKING**: the pimdir backend no longer takes the store's owner role, and `pimdir.source` is gone from the configuration.

  It reads through a lock-free reader and stages each write as one queue action through a short-lived producer, which is what the format asks of a process that is not the store's owner.

  A listing therefore runs beside a sync instead of failing against it, and a staged write reads back before the sync applies it. Nothing attributes a write to a replica source any more, so `pimdir.source` has no meaning and an account still carrying it is an error.

  The pimdir store must now already exist: creating one is the sync engine's job, and a root naming no store says so instead of listing an empty set of addressbooks.

- **BREAKING**: the pimdir backend refuses `addressbook create`, and `card create` reports the card's link id.

  Declaring a collection is an owner write, and a collection no sync knows about is one no sync would carry, so all three `addressbook` writes now refuse on pimdir. A queued create has no store-assigned id until the sync applies it, so there is none to report.

- **BREAKING**: renamed `completions` and `manuals` to `completion` and `manual`, the plural staying as a hidden alias.

  A command mirroring a vendor API resource keeps that API's spelling, so `people contact-group members`, `jmap address-book changes` and `jmap contact-card changes` are unchanged, each gaining a hidden singular alias.

  The `msgraph` family, singular where Graph is plural, is aligned onto Graph: `msgraph contact-folder` and `contact` become `contact-folders` and `contacts`, joining the `child-folders` that already sat under the first. Every singular spelling stays as a hidden alias, where it used to be shown beside the plural.

- Reworded the card argument of `card read`, `card update` and `card delete`, which called itself a card `UID`.

  It is the backend's own identifier, the one `card list` reports, and a `UID` names no card on its own.

### Fixed

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
