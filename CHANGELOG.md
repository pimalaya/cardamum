# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added a `pimdir` local backend: Cardamum over a [pimdir](https://github.com/pimalaya/pimdir) store, the SQLite-indexed, content-addressed offline cache a sync engine such as Neverest populates. It is a cache rather than a server, so a card whose body has not been fetched yet still lists (projected from its stored summary) but reports "body not fetched" on read, and every write is staged as a pending action the next sync pushes. Its link ids, summaries, sort keys and content hashes match Neverest's, so a card added here deduplicates against the same card arriving through a sync.
- Added three remote backends alongside CardDAV: JMAP contacts (RFC 8620 + RFC 9610, via io-jmap), the Microsoft Graph contacts API (via io-msgraph), and the Google People API (via io-people), each behind its own cargo feature (`jmap`, `msgraph`, `people`). The `--backend` flag and the `account list` / `account check` reports gained the matching variants.
- Synthesized a vCard document of record for the backends with no native vCard representation: JMAP ContactCards convert through vcard-rs's JSContact codec (RFC 9555), while Graph and People contacts project field-by-field with a provider-side stash for the properties that have no first-class slot. The projection modules are ported from cardamum-android so both products treat provider quirks identically.
- Added the `vdir item` subcommand (`list`, `get`, `create`, `update`, `delete`) to the vdir-specific API, operating on the raw item files of any kind byte-for-byte. Unlike the shared `card` API it surfaces iCalendar items too and reports each item's kind; `create` infers the kind from the input (`--kind` to override) and `update` preserves it.
- Restructured the CardDAV-specific API into a flat list of WebDAV / CardDAV methods (RFC 4918 / 5689 / 6352 / 6578): `discover`, `propfind`, `proppatch`, `mkcol`, `report {query,multiget,sync}`, `get`, `put`, `delete`. This adds the machinery the shared API hides: `report sync` (RFC 6578 sync-token), `report multiget`, CTag/sync-token surfacing in `propfind`, PROPPATCH, ETag-aware `get`/`put` (`--if-match` / `--if-none-match`), and card-level `delete`, and replaces the previous `list` / `create` / card-scoped `propfind` / flat `report` commands.
- Added a Microsoft Graph-specific API (`msgraph`), nested by Graph resource: `contact-folder {list, child-folders, get, create, rename, delete}`, `contact {list, get, create, update, delete, delta}` and `profile get`. It works with the raw Graph contact model: `create`/`update` take a Graph JSON body (file, inline, or stdin) and `--json` prints the raw Graph payload, and exposes `contact delta`, the Graph-native incremental sync the shared API hides.
- Added a Google People-specific API (`people`), nested by People resource: `contact-group {list, get, create, update, delete, members}`, `connection {list, get, create, update, delete, search}`, `other-contact {list, search, copy}` and `profile get`. It works with the raw People model: `create`/`update` take a People person JSON body and `--json` prints the raw People payload, `connection update` deriving the `updatePersonFields` mask from the body's keys, and exposes the People-native surface the shared API hides: `connection list --sync-token`, the `other-contact` source and `contact-group members`.
- Added a JMAP-specific API (`jmap`), nested by JMAP object type: `address-book {get, create, update, destroy, changes}`, `contact-card {get, query, create, update, destroy, changes, copy}` and `session get`. It works with the raw JSContact model: `create`/`update` take a JSContact JSON body (`update` a JMAP patch), `--json` prints the raw JMAP payload, and exposes the JMAP-native surface the shared API hides: `changes` incremental sync on both objects, `session get`, and cross-account `contact-card copy`.
- Added raw protocol escape hatches to the specific APIs: `msgraph`/`people` `request <method> <path> [json]` and `jmap request <json>` (a raw JMAP method-call), each printing the raw response, plus CardDAV `report raw <addressbook> <xml>` for an arbitrary REPORT body. Also added `vdir item copy` / `move` between collections.

### Changed

- Reworked the first-run wizard around automatic discovery, aligning it with Himalaya's requirement for requirement so both products share one onboarding model. A single opening prompt takes an email address, a server URL, or a local folder path. An email (or bare domain) feeds io-pim-discovery's parallel discovery (fixed provider rules, PACC, RFC 6764 CardDAV, RFC 8620 JMAP, refined by a `WWW-Authenticate` probe), bounded by an 8-second deadline so one unreachable endpoint cannot stall the prompt, and lists one entry per reachable service. Picking one then prompts its authentication method among those advertised, skipped when only one qualifies. A `scheme://` URL discovers from its host, its scheme narrowing the results. A filesystem path is a local vdir or pimdir store, told apart by their on-disk markers.

  The wizard now configures only what it can discover: it no longer prompts for hand-entered server fields, and stops with a pointer to config.sample.toml when discovery finds nothing. A self-hosted server that publishes no discovery record must be configured by hand.

- Made the wizard save the generated configuration for you: it tests the account, then offers to write it to `$XDG_CONFIG_HOME/cardamum/config.toml`, never overwriting an existing file without confirmation and falling back to printing when declined. In JSON mode, or when stdout is redirected, it prints the TOML document instead, so `cardamum > <config>` still works. The guidance that used to head that document as comments moved to a stderr welcome banner, leaving the output bare config. The generated account is no longer marked `default`, so merging it into an existing configuration cannot hijack the default account.
- Stopped prompting for an account name: it is only the TOML table key, so it is derived from the input (the domain's first label, or the folder name) and renamed by editing that key.
- Reworked the wizard's credential prompt onto pimalaya-cli's shared OS-aware picker: a secret now comes from an OS keyring (`secret-tool`, `kwallet-query`, `security`, `pass`), an OAuth 2.0 token broker (`ortie`, `pizauth`, `oama`), a custom command, or a raw value. OAuth is not a standalone choice, since the CLI never runs a grant: it folds into the API-token prompt, unlocking the brokers only when the service advertises it.
- Upgraded every Pimalaya dependency to its current version (io-http 0.3, io-jmap 0.2.1, io-msgraph 0.2.2, io-people 0.2, io-pim-discovery 0.5, io-vdir 0.1, pimalaya-cli 0.2, pimalaya-config 0.1.1, pimalaya-stream 0.1.2, vcard-rs 0.2), and moved io-webdav to its current revision for the Pimalaya naming canon and its card-addressing fixes. Account discovery moved from the obsolete pimconf crate to its successor io-pim-discovery.
- Bumped comfy-table from v7 to v8, which replaces the positional preset string with a typed style builder. The `table.preset` option keeps accepting the v7 spelling, mapped onto the new builder, so existing configurations stay valid. Truncated cells now end with `…` rather than `...`.
- Replaced the io-addressbook aggregator dependency with a product-owned cross-backend layer: the shared `Addressbook`/`Card` types now live in `shared/`, and each backend maps them onto its protocol crate directly through a `backend.rs` glue module. This follows the org decision to retire the per-domain aggregator crates (the interface aggregates, the protocol crates stay leaf libraries).
- Aligned the config schema with the latest Pimalaya API: `[jmap]` mirrors himalaya's (server, TLS, ALPN, `header`/`bearer`/`basic` auth), and `[msgraph]` / `[people]` are OAuth 2.0 bearer-token only, with the same `Secret` shape (a shell `command` or a `raw` value) so tokens from Ortie are configured the same way across tools.
- Documented each command's JSON output shape as the last paragraph of its `--help` text, and slimmed the README Usage section down to a pointer to `cardamum --help` instead of duplicating per-command usage.
- Extracted the `-k/--addressbook ADDRESSBOOK-ID` flag into a shared argument reused across the whole shared API; `card` commands and `addressbook update` now take it as a non-positional flag (replacing positional ids) with the usual fallback: the flag wins, otherwise `addressbook.default`, otherwise the command bails.
- Replaced `addressbook delete`'s positional id with a `-k/--addressbook` flag (consistent name with the rest of the API) that stays mandatory: deletion never falls back to `addressbook.default`, so it always targets an explicit addressbook.
- Documented per-provider CardDAV setup in the README (iCloud, Gmail via OAuth 2.0 / ortie, Posteo) plus unsupported-provider notes (Proton, Microsoft).
- Changed `card create` and `card update` to take their vCard as a trailing positional `VCARD` argument (a path, raw vCard contents, or `-` for stdin) instead of the `--file` flag, matching `tcard edit`.
- Accepted a bare domain or `domain:port` in addition to a full URL for `carddav.server`, defaulting bare authorities to `https://` (matching Himalaya's server-string handling).
- Migrated to the new pimalaya-cli / pimalaya-config / pimalaya-stream stack and adopted the Himalaya v2 CLI structure (shared/ + vdir/ + carddav/).
- Renamed the shared subcommands to the singular `addressbook` and `card` to match Himalaya; the plural `addressbooks` / `cards` forms stay as hidden aliases.
- Renamed the remote backend from `webdav` to `carddav` across the public surface: the `carddav` cargo feature, the `cardamum carddav` subcommand, and the `[carddav]` config block. Only the underlying io-webdav dependency keeps the WebDAV name.
- Relicensed from AGPL-3.0-only to dual MIT OR Apache-2.0, matching Himalaya.

### Removed

- Removed `account configure` (and its `edit` alias): the wizard, run via bare `cardamum`, is now the single way to create an account, and `account list` / `account check` cover inspection and validation.
- Removed the docs/ folder, replaced by cairn/ following the Cairn convention (spec, changes, log) with its AGENTS.md activation stanza. The provider test reports moved to cairn/spec/testing unchanged; the design notes were rewritten as one requirement-based spec file per capability.

### Fixed

- Fixed a 404 when connecting to a CardDAV server whose discovery (PACC or RFC 6764) hands back a bare origin rather than the context root (fastmail serves contacts under `/dav/` and 404s everything else). The client now probes `.well-known/carddav` and follows its redirect before the principal walk whenever the resolved server path is `/`, mirroring the cardamum-android behaviour.
- Fixed the raw OS error (`No such file or directory`) surfaced by the vdir-specific `rename` / `delete` (and now `item`) commands when the collection does not exist; they bail with "Collection `<name>` not found" instead.
- Fixed `carddav discover` failing outright on a `home`-configured account (e.g. iCloud), where the principal is never resolved: it now reports each endpoint best-effort, showing `principal: (unresolved)` instead of erroring.

## [0.1.0] - 2025-10-24

### Added

- Add missing desc column to listing
- Add vdir support

### Changed

- Init nix and rust env
- Init sans I/O module with std connector
- Init http, tls and carddav modules
- Init cli structure
- Init config system without wizard
- Init basic backend account config
- Init cli list command
- Init cli list command
- Init cli read command
- Init table for list command
- Init create command
- Init update command
- Init delete command
- Init create command
- Init update command
- Init delete command
- Integrate vparser to list cards command
- Integrate pimalaya/addressbook libs
- Plug cargo features from addressbook-* libs
- Introduce internal cargo flags
- Bump nix flakes and activate ci on master
- Init config.sample.toml
- Bump addressbook libs
- Use rustls ring by default instead of aws-lc
- Improve api and docs
- Switch to AGPL
- Bump dependencies

### Fixed

- Put back addressbook commands
- Put back create card command
- Put back create and read card commands
- Put back update and delete card commands
- Shell-expand home-dir
- Fix builds

### Removed

- Remove full default features
- Remove unused examples folder

## [root] - 2025-01-12

### Added

- Init repository

[0.1.0]: https://github.com/pimalaya/ortie/compare/root..v0.1.0
