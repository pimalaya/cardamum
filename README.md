# 📇 Cardamum [![crates.io](https://img.shields.io/crates/v/cardamum.svg)](https://crates.io/crates/cardamum) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

CLI to manage contacts.

> [!IMPORTANT]
> This README documents Cardamum v0.2.0. If you are running v0.1.0, refer to the [v0.1.0 README](https://github.com/pimalaya/cardamum/blob/v0.1.0/README.md). The [MIGRATION.md](./MIGRATION.md) guide walks v0.1 users through the breaking changes.

## Table of contents

- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- Shared API mapping `addressbook` and `card` commands to the active backend
- Protocol-specific APIs exposing each backend's full surface (`cardamum carddav/jmap/msgraph/people/vdir`)
- Remote backends:
  - **CardDAV** (RFC 6352)
  - **JMAP** contacts (RFC 8620 + RFC 9610)
  - **Microsoft Graph** contacts API
  - **Google People** API
- Local backends:
  - **vdir** [specs](https://vdirsyncer.pimutils.org/en/stable/vdir.html), one directory per addressbook
  - **pimdir**, the offline store a sync engine such as [Neverest](https://github.com/pimalaya/neverest) populates: read your contacts and stage changes without a network round-trip
- vCard document of record synthesized for the backends with no native vCard (JMAP via JSContact, Graph, People)
- HTTP auth support: basic, bearer (OAuth 2.0 access tokens issued by an external tool such as [Ortie](https://github.com/pimalaya/ortie))
- TLS support:
  - [Rustls](https://crates.io/crates/rustls) with ring crypto
  - [Rustls](https://crates.io/crates/rustls) with aws crypto (requires `rustls-aws` feature)
  - [Native TLS](https://crates.io/crates/native-tls) (requires `native-tls` feature)
- Email-driven service discovery (fixed provider rules, PACC, RFC 6764 DAV, RFC 8620 JMAP, with a `WWW-Authenticate` probe), plus per-backend discovery:
  - `.well-known/carddav` [rfc6764](https://datatracker.ietf.org/doc/html/rfc6764)
  - Current-user-principal [rfc5397](https://datatracker.ietf.org/doc/html/rfc5397)
  - Addressbook-home-set [rfc6352](https://datatracker.ietf.org/doc/html/rfc6352)
  - JMAP session `.well-known/jmap` [rfc8620](https://datatracker.ietf.org/doc/html/rfc8620)
- Per-backend Cargo features: `carddav`, `jmap`, `msgraph`, `people`, `vdir`, `pimdir` (all on by default)
- TOML configuration with multi-account support
- Interactive wizard, on first run and from `cardamum configure`: it discovers a provider from your email address, tests the connection, and saves a ready-to-use account
- JSON output via `--json`

## Installation

### Pre-built binary

As root:

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/cardamum/master/install.sh | sudo sh
```

As a regular user:

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/cardamum/master/install.sh | PREFIX=~/.local sh
```

These commands install the latest binary from the GitHub [releases](https://github.com/pimalaya/cardamum/releases) section.

For a more up-to-date version than the latest release, check out the [releases](https://github.com/pimalaya/cardamum/actions/workflows/releases.yml) GitHub workflow and look for the *Artifacts* section. These pre-built binaries are built from the `master` branch.

> [!NOTE]
> Such binaries are built with the default cargo features. If you need specific features, please use another installation method.

### Cargo

```sh
cargo install --locked --git https://github.com/pimalaya/cardamum.git
```

With only a subset of backends (each backend is a Cargo feature: `carddav`, `jmap`, `msgraph`, `people`, `vdir`, `pimdir`):

```sh
cargo install --locked --git https://github.com/pimalaya/cardamum.git \
  --no-default-features \
  --features carddav,vdir,rustls-ring
```

### Nix

If you have the [Flakes](https://nixos.wiki/wiki/Flakes) feature enabled:

```sh
nix profile install github:pimalaya/cardamum
```

Or run without installing:

```sh
nix run github:pimalaya/cardamum
```

### Sources

```sh
git clone https://github.com/pimalaya/cardamum
cd cardamum
nix run
```

## Configuration

The configuration is loaded from the first existing path among:

- `$XDG_CONFIG_HOME/cardamum/config.toml`
- `$HOME/.config/cardamum/config.toml`
- `$HOME/.cardamumrc`

Override with `cardamum -c <PATH>`. Multiple paths can be passed at once, separated by `:`; the first is the base and the rest are deep-merged on top.

Run `cardamum configure` to launch the wizard. A bare `cardamum` offers it when it finds no configuration, and so does any command needing an account; already configured, a bare `cardamum` shows the help instead. The wizard opens with a single prompt that takes an email address, a server URL, or a local folder path, and its shape orients the rest of the setup, exactly like the Cardamum Android onboarding. An email address (or bare domain) runs discovery: the wizard detects the provider then searches every reachable contacts service (CardDAV, JMAP, plus the Google People and Microsoft Graph APIs for those providers) and lets you pick one, then asks how to authenticate against it. A `scheme://` URL discovers from its host, its scheme narrowing the results. A filesystem path is a local vdir or pimdir store, told apart by their on-disk markers.

The wizard configures only what it can discover automatically. It never asks you to type a server field by hand, and when discovery finds nothing it stops and points you at the documented sample. A self-hosted server that publishes no discovery record is configured by writing the account yourself.

It then tests the connection and offers to save the account, creating the configuration file or appending an `[accounts.<name>]` block to the one already there, so a second account never means merging by hand. The append is plain text, leaving your comments and formatting untouched. When stdout is redirected, or with `--json`, it prints the block instead of writing anything:

```sh
cardamum configure > ~/.config/cardamum/config.toml
```

Credentials come from an OS keyring (`secret-tool`, `kwallet-query`, `security`, `pass`), an OAuth 2.0 token broker (`ortie`, `pizauth`, `oama`), a custom command, or a raw value in the file. Cardamum reads secrets but never issues or refreshes them: it runs no OAuth 2.0 grant, so for providers that require OAuth (Google, Microsoft, and any CardDAV or JMAP server behind it) pick a broker such as [Ortie](https://github.com/pimalaya/ortie), which refreshes and prints a fresh token on every read. See the [Google](#google) example below.

A documented sample lives at [config.sample.toml](./config.sample.toml).

### Apple

Apple exposes contacts via CardDAV, but you cannot use your regular password. You need to generate an [app-specific password](https://support.apple.com/en-us/HT204397) (required once two-factor authentication is on):

```toml
[accounts.example]
carddav.discover = "icloud.com"
carddav.server = "https://contacts.icloud.com/"
# The home URL is usually of this shape:
#carddav.home = "https://contacts.icloud.com/<id>/principal/"

carddav.auth.basic.username = "example@icloud.com"
carddav.auth.basic.password.raw = "***"

addressbook.default = "card"
```

### Google

Google exposes contacts via CardDAV and via the richer [People API](https://developers.google.com/people); both require [OAuth 2.0](https://developers.google.com/identity/protocols/oauth2). Manage the access token with any tool that refreshes it (for example [Ortie](https://github.com/pimalaya/ortie)).

CardDAV:

```toml
[accounts.google-carddav]
carddav.home = "https://www.googleapis.com/carddav/v1/principals/<email>/lists"
carddav.auth.bearer.token.command = ["ortie", "token", "show"]
addressbook.default = "default"
```

People API (contact groups map to addressbooks; the `myContacts` group lists as `Contacts`):

```toml
[accounts.google-people]
google.auth.token.command = ["ortie", "token", "show"]
addressbook.default = "myContacts"
```

### Microsoft

Microsoft exposes contacts through the [Graph API](https://learn.microsoft.com/en-us/graph/api/resources/contact) (OAuth 2.0 bearer only; no CardDAV). Contact folders map to addressbooks, with the default Contacts folder listed under the `contacts` id.

```toml
[accounts.microsoft]
msgraph.auth.token.command = ["ortie", "token", "show"]
addressbook.default = "contacts"
```

### Proton

Not supported: Proton exposes no contacts API, neither CardDAV nor through [Proton Bridge](https://proton.me/mail/bridge) (which proxies mail only). Contacts are reachable only from Proton's own web and mobile apps.

### Fastmail

Standard CardDAV with the mailbox address and its app password:

```toml
[accounts.fastmail-carddav]
carddav.discover = "fastmail.com"
carddav.server = "https://carddav.fastmail.com/dav/"
# The home URL is usually of this shape:
#carddav.home = "https://carddav.fastmail.com/dav/addressbooks/user/<email>/"

carddav.auth.basic.username = "example@fastmail.com"
carddav.auth.basic.password.raw = "***"

addressbook.default = "Default"
```

Or JMAP, which Fastmail serves bearer-token only (an API token from the [Fastmail settings](https://www.fastmail.com/settings/security/tokens)):

```toml
[accounts.fastmail-jmap]
jmap.server = "fastmail.com"
jmap.auth.bearer.token.raw = "***"
```

### Posteo

Standard CardDAV with the mailbox address and its password.

```toml
[accounts.posteo]
carddav.discover = "posteo.de"
carddav.server = "https://posteo.de:8843/"
# The home URL is usually of this shape:
#carddav.home = "https://posteo.de:8843/addressbooks/<username>/"

carddav.auth.basic.username = "example@posteo.net"
carddav.auth.basic.password.raw = "***"

addressbook.default = "default"
```

## Usage

Run `cardamum --help` for the full command tree, and `cardamum <command> --help` for any subcommand's arguments and its JSON output shape (printed when the global `--json` flag is set).

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- 2026 → 2027: [NGI Zero Commons Fund](https://nlnet.nl/project/Pimalaya-pimdir/)

This program is part of Pimalaya, free software funded entirely by grants and donations. If you find it useful, consider [sponsoring](https://pimalaya.org/sponsor/) its development:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/pimalaya)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/pimalaya)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/pimalaya)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/u/gh/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
