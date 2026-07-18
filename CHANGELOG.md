# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added three remote backends alongside CardDAV: JMAP contacts (RFC 8620 + RFC 9610, via io-jmap), the Microsoft Graph contacts API (via io-msgraph), and the Google People API (via io-people), each behind its own cargo feature (`jmap`, `msgraph`, `google`). The `--backend` flag and the `account list` / `account check` reports gained the matching variants.
- Synthesized a vCard document of record for the backends with no native vCard representation: JMAP ContactCards convert through vcard-rs's JSContact codec (RFC 9555), while Graph and People contacts project field-by-field with a provider-side stash for the properties that have no first-class slot. The projection modules are ported from cardamum-android so both products treat provider quirks identically.

### Changed

- Reworked the first-run wizard around email-driven discovery, matching the Cardamum Android onboarding: a single opening prompt (asked before the account name) takes an email address, a server URL, or a local vdir path. An email (or bare domain) feeds io-pim-discovery's parallel discovery (fixed provider rules, PACC, RFC 6764 CardDAV, RFC 8620 JMAP, refined by a `WWW-Authenticate` probe) and lists every reachable service and authentication method as a selectable entry; a `scheme://` URL is a CardDAV server to set up by hand; a filesystem path is a local vdir (validated for existence). JMAP, Microsoft Graph and Google People are reached through discovery, replacing the manual per-backend picker.
- Made the wizard write nothing to disk, like Ortie: it tests the account, then prints the resulting configuration as a ready-to-save TOML document on stdout (prompts render on stderr), so `cardamum > <config>` saves it. Running bare `cardamum` (no subcommand) launches it, and it is proposed when a command finds no config. Empty tables are pruned and backend blocks rendered as dotted keys, so the output matches the shape of config.sample.toml.
- Dropped OAuth 2.0 from the wizard's authentication choices: since the CLI never runs a grant nor refreshes tokens, it offers only a password or a token, and OAuth-only services surface as a token (issued and refreshed by an external manager such as Ortie).
- Upgraded every Pimalaya dependency to its released version (io-http 0.3, io-jmap / io-msgraph 0.2, io-people 0.2, io-vdir / io-webdav 0.1, pimalaya-cli / pimalaya-config / pimalaya-stream 0.1) and dropped the local path / git patches. Account discovery moved from the obsolete pimconf crate to its successor io-pim-discovery.
- Replaced the io-addressbook aggregator dependency with a product-owned cross-backend layer: the shared `Addressbook`/`Card` types now live in `shared/`, and each backend maps them onto its protocol crate directly through a `backend.rs` glue module. This follows the org decision to retire the per-domain aggregator crates (the interface aggregates, the protocol crates stay leaf libraries).
- Aligned the config schema with the latest Pimalaya API: `[jmap]` mirrors himalaya's (server, TLS, ALPN, `header`/`bearer`/`basic` auth), and `[msgraph]` / `[google]` are OAuth 2.0 bearer-token only, with the same `Secret` shape (a shell `command` or a `raw` value) so tokens from Ortie are configured the same way across tools.
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

### Fixed

- Fixed a 404 when connecting to a CardDAV server whose discovery (PACC or RFC 6764) hands back a bare origin rather than the context root (fastmail serves contacts under `/dav/` and 404s everything else). The client now probes `.well-known/carddav` and follows its redirect before the principal walk whenever the resolved server path is `/`, mirroring the cardamum-android behaviour.

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
