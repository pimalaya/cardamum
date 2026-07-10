# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fixed a 404 when connecting to a CardDAV server whose discovery (PACC or RFC 6764) hands back a bare origin rather than the context root (fastmail serves contacts under `/dav/` and 404s everything else). The client now probes `.well-known/carddav` and follows its redirect before the principal walk whenever the resolved server path is `/`, mirroring the cardamum-android behaviour.

### Changed

- Collapsed the wizard onto a single opening prompt that takes an email address, a server URL, or a local vdir path (asked before the account name), matching the Cardamum Android onboarding: an email runs provider detection and discovery, a `scheme://` URL is a CardDAV server, and a filesystem path is a local vdir (validated for existence). Dropped the manual per-backend picker; JMAP, Microsoft Graph and Google People are reached through discovery.
- Dropped OAuth 2.0 from the wizard's authentication choices: since the CLI never runs a grant nor refreshes tokens, it offers only a password or a token, and OAuth-only services surface as a token (issued and refreshed by an external manager such as Ortie).

### Added

- Added three remote backends alongside CardDAV: JMAP contacts (RFC 8620 + RFC 9610, via io-jmap), the Microsoft Graph contacts API (via io-msgraph), and the Google People API (via io-google-people), each behind its own cargo feature (`jmap`, `msgraph`, `google`). The `--backend` flag and the `account list` / `account check` reports gained the matching variants.
- Synthesized a vCard document of record for the backends with no native vCard representation: JMAP ContactCards convert through calcard's JSContact codec, while Graph and People contacts project field-by-field with a provider-side stash for the properties that have no first-class slot. The projection modules are ported from cardamum-android so both products treat provider quirks identically.
- Reworked the wizard around email-driven discovery: a new first pick feeds the address to pimconf's parallel search (fixed provider rules, PACC, RFC 6764 DAV, RFC 8620 JMAP, refined by a `WWW-Authenticate` probe) and lists every discovered service and authentication method as a selectable entry, matching the cardamum-android configuration screen.

### Changed

- Replaced the io-addressbook aggregator dependency with a product-owned cross-backend layer: the shared `Addressbook`/`Card` types now live in `shared/`, and each backend maps them onto its protocol crate directly through a `backend.rs` glue module. This follows the org decision to retire the per-domain aggregator crates (the interface aggregates, the protocol crates stay leaf libraries).
- Aligned the config schema with the latest Pimalaya API: `[jmap]` mirrors himalaya's (server, TLS, ALPN, `header`/`bearer`/`basic` auth), and `[msgraph]` / `[google]` are OAuth 2.0 bearer-token only, with the same `Secret` shape (a shell `command` or a `raw` value) so tokens from Ortie are configured the same way across tools.
- Documented each command's JSON output shape as the last paragraph of its `--help` text, and slimmed the README Usage section down to a pointer to `cardamum --help` instead of duplicating per-command usage.
- Extracted the `-k/--addressbook ADDRESSBOOK-ID` flag into a shared argument reused across the whole shared API; `card` commands and `addressbook update` now take it as a non-positional flag (replacing positional ids) with the usual fallback: the flag wins, otherwise `addressbook.default`, otherwise the command bails.
- Switched `account configure` from a positional account name to the global `-a/--account` flag, for consistency with the rest of the CLI; without it, the default account is edited.
- Replaced `addressbook delete`'s positional id with a `-k/--addressbook` flag (consistent name with the rest of the API) that stays mandatory: deletion never falls back to `addressbook.default`, so it always targets an explicit addressbook.
- Reused an existing account's secret when editing: a stored shell-command secret now prefills the command prompt (and defaults the secret strategy) instead of showing the generic one.
- Documented per-provider CardDAV setup in the README (iCloud, Gmail via OAuth 2.0 / ortie, Posteo) plus unsupported-provider notes (Proton, Microsoft).
- Reworked the first-run wizard: defaulted the account name to `personal` instead of the reserved `default`, replaced the per-backend yes/no questions with a single backend picker, and asked for the email only on the CardDAV path.
- Reused the shared pimalaya-cli CardDAV wizard in both the first-run wizard and `account configure`, and wrapped it with a connection test that re-runs the wizard (prefilled) on failure instead of leaving the program.
- Unified `account configure` with the first-run wizard onto a single shared flow (same backend picker and prompts); the only difference is that editing seeds every default from the current account and skips CardDAV discovery (to discover a new server, create a new account).
- Reworked CardDAV discovery to run PACC, then `.well-known`, then RFC 6764 SRV (last), combined in the CLI rather than the lib; the wizard probes them behind the scenes with one spinner per mechanism and prefills a single `CardDAV server:` prompt with the result.
- Replaced the wizard's hostname/encryption/port/home prompts with the single `server` prompt, defaulted the CardDAV username to the account email, recorded both `discover` and `server` in the written config (server wins), and printed the config path once everything succeeds.
- Deferred writing the configuration file until the wizard fully succeeds, so a failed run no longer leaves a half-written file that blocks a restart.
- Pruned empty tables, rendered account backend blocks as dotted keys, and wrote shell-command secrets as plain strings when saving the configuration, so generated files match the shape of config.sample.toml.
- Made the parent addressbook of every `card` command an optional `-k/--addressbook ADDRESSBOOK-ID` flag instead of a positional argument; when omitted it falls back to the new `addressbook.default` config, otherwise the command bails.
- Changed `card create` and `card update` to take their vCard as a trailing positional `VCARD` argument (a path, raw vCard contents, or `-` for stdin) instead of the `--file` flag, matching `tcard edit`.
- Accepted a bare domain or `domain:port` in addition to a full URL for `carddav.server`, defaulting bare authorities to `https://` (matching Himalaya's server-string handling).
- Migrated to the new pimalaya-cli / pimalaya-config / pimalaya-stream stack and adopted the Himalaya v2 CLI structure (shared/ + vdir/ + carddav/).
- Renamed the shared subcommands to the singular `addressbook` and `card` to match Himalaya; the plural `addressbooks` / `cards` forms stay as hidden aliases.
- Renamed the remote backend from `webdav` to `carddav` across the public surface: the `carddav` cargo feature, the `cardamum carddav` subcommand, and the `[carddav]` config block. Only the underlying io-webdav dependency keeps the WebDAV name.
- Relicensed from AGPL-3.0-only to dual MIT OR Apache-2.0, matching Himalaya.

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
