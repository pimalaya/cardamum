# 📇 Cardamum [![crates.io](https://img.shields.io/crates/v/cardamum.svg)](https://crates.io/crates/cardamum) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

CLI to manage contacts, written in Rust

> [!CAUTION]
> Cardamum is `v0.x`: expect breaking changes between releases until it stabilises.

## Table of contents

- [Features](#features)
- [RFC coverage](#rfc-coverage)
- [Installation](#installation)
  - [Pre-built binary](#pre-built-binary)
  - [Cargo](#cargo)
  - [Nix](#nix)
  - [Sources](#sources)
- [Configuration](#configuration)
  - [Apple](#apple)
  - [Google](#google)
  - [Microsoft](#microsoft)
  - [Fastmail](#fastmail)
  - [Posteo](#posteo)
  - [Proton](#proton)
  - [Local addressbooks](#local-addressbooks)
- [Usage](#usage)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- **Shared API**: `addressbook` and `card` work the same whichever backend serves the account.
- **Protocol-specific APIs**: `carddav`, `jmap`, `msgraph`, `people` and `vdir` each expose what only that backend has, down to a raw request escape hatch.
- **CardDAV**: talk to any standard address book server, with basic or bearer authentication.
- **JMAP**: contact cards and address books over the JSON protocol, including its incremental `changes` sync.
- **Microsoft Graph**: the native contacts API, where Microsoft offers no CardDAV, with the vCard synthesized both ways.
- **Google People**: the native API, richer than Google's CardDAV bridge, with contact groups mapped to addressbooks.
- **vdir**: read and write a local [vdir](https://vdirsyncer.pimutils.org/en/stable/vdir.html) home, one directory per addressbook.
- **pimdir**: read and stage writes against a local [pimdir](https://github.com/pimalaya/pimdir) store, the offline cache a sync engine fills.
- **Discovery**: an email address is enough to find a provider's server, through SRV records, `.well-known` and the provider configuration documents.
- **Interactive wizard**: `cardamum configure` turns an email address into a tested account.
- **Multi-account**: one TOML file, one block per account, several files deep-merged when you want secrets apart.
- **JSON output**: every command switches to JSON with `--json`, for scripts and other tools.
- Full standard, blocking client with **TLS** support:
  - [Rustls](https://crates.io/crates/rustls) with ring crypto (requires `rustls-ring` feature, enabled by default)
  - [Rustls](https://crates.io/crates/rustls) with aws crypto (requires `rustls-aws` feature)
  - [Native TLS](https://crates.io/crates/native-tls) (requires `native-tls` feature)

> [!TIP]
> Each backend sits behind its own cargo feature. `carddav`, `jmap`, `msgraph`, `people` and `vdir` are enabled by default, `pimdir` is opt-in. Build with `--no-default-features` and pick the ones you need.

## RFC coverage

| RFC    | What is covered                                                                              |
|--------|----------------------------------------------------------------------------------------------|
| [6352] | CardDAV: address book collections, address object resources, and the `addressbook-query` and `addressbook-multiget` REPORTs |
| [4918] | WebDAV: the `PROPFIND`, `PROPPATCH`, `MKCOL`, `GET`, `PUT` and `DELETE` methods CardDAV builds on |
| [5397] | Current-user-principal, the first step of the CardDAV discovery walk                          |
| [6578] | Collection synchronization: the `sync-collection` REPORT and the sync token a listing reports |
| [6764] | CardDAV service discovery: the `_carddav` and `_carddavs` SRV records, and `.well-known/carddav` |
| [6350] | vCard: the contact format read, written and rendered by every backend                        |
| [8620] | JMAP: the session resource, its `.well-known/jmap` discovery and the request and response envelope |
| [9610] | JMAP contacts: the `AddressBook` and `ContactCard` types, their `changes` and their `copy`    |
| [9555] | JSContact, the JMAP contact model, converted to and from vCard                                |
| [7617] | HTTP Basic authentication                                                                     |
| [6750] | HTTP Bearer authentication, for a provider-issued or broker-refreshed API token               |

[4918]: https://www.rfc-editor.org/rfc/rfc4918
[5397]: https://www.rfc-editor.org/rfc/rfc5397
[6350]: https://www.rfc-editor.org/rfc/rfc6350
[6352]: https://www.rfc-editor.org/rfc/rfc6352
[6578]: https://www.rfc-editor.org/rfc/rfc6578
[6750]: https://www.rfc-editor.org/rfc/rfc6750
[6764]: https://www.rfc-editor.org/rfc/rfc6764
[7617]: https://www.rfc-editor.org/rfc/rfc7617
[8620]: https://www.rfc-editor.org/rfc/rfc8620
[9555]: https://www.rfc-editor.org/rfc/rfc9555
[9610]: https://www.rfc-editor.org/rfc/rfc9610

Microsoft Graph and Google People are provider APIs rather than RFCs: their contact resources are covered through the [Graph contact](https://learn.microsoft.com/en-us/graph/api/resources/contact) and [People](https://developers.google.com/people) references.

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

These commands install the latest binary from the GitHub [releases](https://github.com/pimalaya/cardamum/releases) section. For a more up-to-date version, check the [releases](https://github.com/pimalaya/cardamum/actions/workflows/releases.yml) workflow and look for the *Artifacts* section: those are built from `master`, with the default cargo features.

### Cargo

```sh
cargo install --locked --git https://github.com/pimalaya/cardamum.git
```

With CardDAV and vdir support only:

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

Override the path with `cardamum -c <PATH>` or `CARDAMUM_CONFIG=<PATH>`. Multiple paths can be passed at once, separated by `:`; the first is the base and the rest are deep-merged on top, which is how a public configuration and a private one stay separate files. The full field reference lives in [config.sample.toml](./config.sample.toml).

Run `cardamum configure` to launch the wizard, which a bare `cardamum` also offers when it finds no configuration. It asks one question, taking an email address, a server URL, or a local folder path, and the shape of what you type decides the rest. An address is discovered: every reachable service is offered, and picking one prompts only how to authenticate against it. A URL discovers from its host, its scheme narrowing the results. A folder is detected as a vdir home or a pimdir store. The wizard tests the account, then writes it into your configuration file, appending an `[accounts.<name>]` block when one is already there.

What it cannot discover it does not ask for: a self-hosted server publishing no discovery record is written by hand, against the sample above. Credentials come from an OS keyring (`secret-tool`, `kwallet-query`, `security`, `pass`), a token broker such as [Ortie](https://github.com/pimalaya/ortie), any custom command, or a raw value in the file. Cardamum reads secrets but never issues or refreshes them, so a provider requiring OAuth 2.0 needs a broker to keep its token fresh.

Ready-made blocks for common providers follow.

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

Use the `people` backend, which speaks the [People API](https://developers.google.com/people) directly and is richer than Google's CardDAV bridge. Both routes need [OAuth 2.0](https://developers.google.com/identity/protocols/oauth2), and an access token expires within the hour, so point the token at a broker rather than pasting one in. Contact groups map to addressbooks, the `myContacts` group listing as Contacts:

```toml
[accounts.example]
people.auth.token.command = ["ortie", "token", "show"]
addressbook.default = "myContacts"
```

Google models contacts as JSON and exposes no vCard representation, so cardamum synthesizes the document you read and re-projects what you write. Properties with a well-defined vCard slot are authoritative in both directions, and everything else is stashed verbatim on the server so it round-trips instead of being dropped on the next write.

CardDAV works too, with a bearer token and the address book home addressed by hand, its discovery entry point being off-spec:

```toml
[accounts.example]
carddav.home = "https://www.googleapis.com/carddav/v1/principals/example@gmail.com/lists"
carddav.auth.bearer.token.command = ["ortie", "token", "show"]
addressbook.default = "default"
```

The two routes need different scopes, which are not interchangeable: `auth/carddav` authorizes CardDAV, `auth/contacts` the People API.

### Microsoft

Microsoft offers no CardDAV, only the [Graph API](https://learn.microsoft.com/en-us/graph/api/resources/contact), which takes an OAuth 2.0 bearer token and nothing else. Contact folders map to addressbooks, the default folder listing under the `contacts` id:

```toml
[accounts.example]
msgraph.auth.token.command = ["ortie", "token", "show"]
addressbook.default = "contacts"
```

As with Google, contacts are JSON: the vCard is synthesized on read and re-projected on write, with the unmapped properties stashed server-side so they survive.

### Fastmail

Standard CardDAV with the mailbox address and its [app password](https://www.fastmail.help/hc/en-us/articles/360058752854-App-passwords):

```toml
[accounts.example]
carddav.discover = "fastmail.com"
carddav.server = "https://carddav.fastmail.com/dav/"
# The home URL is usually of this shape:
#carddav.home = "https://carddav.fastmail.com/dav/addressbooks/user/<email>/"

carddav.auth.basic.username = "example@fastmail.com"
carddav.auth.basic.password.raw = "***"

addressbook.default = "Default"
```

Fastmail also serves JMAP, bearer-token only, the token coming from the [Fastmail settings](https://www.fastmail.com/settings/security/tokens). It forbids creating, updating and destroying an address book, so the `addressbook` write commands fail there by design:

```toml
[accounts.example]
jmap.server = "fastmail.com"
jmap.auth.bearer.token.raw = "***"
```

### Posteo

Standard CardDAV with the mailbox address and its password:

```toml
[accounts.example]
carddav.discover = "posteo.de"
carddav.server = "https://posteo.de:8843/"
# The home URL is usually of this shape:
#carddav.home = "https://posteo.de:8843/addressbooks/<username>/"

carddav.auth.basic.username = "example@posteo.net"
carddav.auth.basic.password.raw = "***"

addressbook.default = "default"
```

### Proton

Not supported: Proton exposes no contacts API, neither CardDAV nor through [Proton Bridge](https://proton.me/mail/bridge), which proxies mail only. Contacts are reachable only from Proton's own web and mobile apps.

### Local addressbooks

No server is involved, so nothing needs discovering. Point cardamum at a directory and it works offline.

A [vdir](https://vdirsyncer.pimutils.org/en/stable/vdir.html) home is one directory per addressbook, holding one `.vcf` file per card. This is what vdirsyncer writes, and what most local tools read:

```toml
[accounts.local]
vdir.home-dir = "~/.local/share/vdirsyncer/contacts"
addressbook.default = "personal"
```

A [pimdir](https://github.com/pimalaya/pimdir) store is the offline cache a sync engine fills: a SQLite index plus content-addressed bodies, shared with the other Pimalaya clients reading the same store. It is a cache, not a server, so addressbooks come from the sync and the collection verbs refuse here. Writes are staged for the next sync to push:

```toml
[accounts.cached]
pimdir.root = "~/.local/state/neverest/example"
# Usually left unset: a store synced as a single source is opened as it.
#pimdir.source = "carddav"
```

A card that is listed but not downloaded reads as "body not fetched" until a sync hydrates it.

## Usage

Run `cardamum --help` for the full command tree, and `cardamum <command> --help` for any subcommand's arguments and its JSON output shape (printed when the global `--json` flag is set).

A few real command lines:

```sh
cardamum configure
cardamum addressbook list
cardamum card list --addressbook personal
cardamum card read --addressbook personal <CARD-ID>
cardamum card create --addressbook personal ada.vcf
cardamum -a work -b carddav card list
cardamum carddav report sync personal
cardamum jmap contact-card changes <SINCE-STATE>
cardamum people connection list --sync-token <TOKEN>
cardamum msgraph contact delta --folder contacts
```

Logs go to stderr, so they can be redirected to a file while the command output stays on stdout:

```sh
cardamum card list --log-level debug 2>/tmp/cardamum.log
```

Use `--log-file <PATH>` to append them to a file directly. When `--log-level` is omitted the `RUST_LOG` environment variable is consulted, and `RUST_BACKTRACE=1` adds the full error backtrace.

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
