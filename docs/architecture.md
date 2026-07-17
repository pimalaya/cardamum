# cardamum architecture reference

The narrative architecture — where cardamum fits in the Pimalaya stack, the backends and plumbing, the three command families, backend selection, vCard projection, the configuration and wizard model, and the output rule — lives in the [`main.rs`](../src/main.rs) crate header. This document holds the reference detail that outgrows that header: the module map, the command conventions, and the deeper mechanics of email discovery and CardDAV resolution.

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) for the conventions every repository shares (layering, `no_std`, module and error rules, code style, licensing). If a statement here conflicts with the code, the code wins; please flag it.

## Module layout

```
src/
  main.rs                entry point + crate architecture header
  cli.rs                 Cli/Command, global flags, resolve_account, execute dispatch
  backend.rs             Backend enum (auto/carddav/jmap/msgraph/google/vdir)
  config.rs              TOML schema: Config, AccountConfig, per-backend blocks, to_toml_string
  project.rs             vCard projection helpers shared by msgraph/google
  shared/                cross-protocol least-common-denominator API
    client.rs            AddressbookClient (BackendClient enum + dispatch)
    addressbook/         Addressbook/AddressbookDiff types + list/create/update/delete
    card/                Card type + list/read/create/update/delete (+ vcard helper)
  carddav/               [carddav] backend + protocol-specific API
    client.rs            WebdavClientStd builder + discovery routes
    backend.rs           shared-API glue over io-webdav
    discover/propfind/report/list/create/delete
  jmap/                  [jmap] backend
    backend.rs           shared-API glue over io-jmap
    project.rs           ContactCard <-> vCard (JSContact via calcard)
  msgraph/               [msgraph] backend
    backend.rs           shared-API glue over io-msgraph
    project.rs           Graph contact <-> vCard
  google/                [google] backend
    backend.rs           shared-API glue over io-people
    project.rs           People person <-> vCard
  vdir/                  [vdir] backend + protocol-specific API
    client.rs            VdirClient builder
    backend.rs           shared-API glue over io-vdir
    list/create/rename/delete
  account/               account list/check + Account context
  wizard/                bare-cardamum interactive config generator + email discovery
```

`shared/` is the portable surface; each protocol folder holds its `backend.rs` glue (and, for CardDAV/vdir, its per-protocol escape-hatch commands); `account/` and `wizard/` are the meta and bootstrap concerns.

## Command conventions

Each subcommand is a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method (the shared nested-execute convention). `Command::execute` in `cli.rs` is the single dispatch point: it resolves the account (`resolve_account`, proposing the wizard when no config exists), builds the appropriate client, and hands it to the subcommand.

Each command's doc comment is its `--help` text: the first paragraph is the short summary shown by `-h`, and the full text (shown by `--help`) ends with the command's JSON output shape. So `cardamum <command> --help` is the canonical usage reference for both humans and AI agents; the README intentionally documents no per-command usage.

## The wizard and email discovery

The wizard (`wizard/discover.rs`) opens with a single prompt that takes an email address, a server URL, or a local vdir path, and its shape orients the setup, mirroring the cardamum-android onboarding. Bare `cardamum` (no subcommand) runs it, and `cli::resolve_account` proposes it when no config file is found. It writes nothing to disk: the resulting account is tested (`account::check::test_account`) then printed as a ready-to-save TOML document on stdout (via `Config::to_toml_string`, which prunes empty tables and renders backend blocks as dotted keys), while every prompt renders on stderr, so `cardamum > <config>` is the write-back, exactly like Ortie.

An email (or bare domain) feeds `wizard/search.rs`, which runs io-pim-discovery's parallel `DiscoveryComposeClientStd::compose_all` over the CardDAV and JMAP services (fixed Google/Microsoft provider rules, PACC, RFC 6764 CardDAV resolve, RFC 8620 JMAP resolve, refined by a `WWW-Authenticate` probe). Each discovered service and authentication-method combination becomes one selectable entry, mirroring the cardamum-android configuration screen; a detected Google or Microsoft account collapses to its proprietary People/Graph API, which no record advertises. Picking an entry pins the endpoint and auth method, so the per-backend flow (`wizard/{carddav,jmap,msgraph,google}.rs`) only prompts for the credentials. A `scheme://` URL is a CardDAV server configured by hand; a filesystem path is a local vdir. The DNS resolver honours `CARDAMUM_DNS_RESOLVER`, then the system resolver, then Cloudflare's `1.1.1.1`. Only a password or a token is offered: the CLI never runs an OAuth 2.0 grant nor refreshes tokens, so OAuth-only services surface as a token issued and refreshed by an external manager such as Ortie.

## CardDAV discovery routes

The `[carddav]` block resolves the addressbook home set through one of three routes, in decreasing order of magic (`carddav/client.rs`, `open_carddav_client`):

- `home`: short-circuits all discovery; the configured URL is used as the addressbook home set directly;
- `server`: connects to the given context root, then walks current-user-principal (RFC 5397) and addressbook-home-set (RFC 6352);
- `discover`: resolves a bare domain to a context root before running that same walk: PACC first, then RFC 6764 (SRV record, its TXT `path`, then `.well-known`) through io-pim-discovery; Google domains use an authenticated `.well-known` probe.

Because PACC and RFC 6764 can hand back a bare origin rather than the real context root (fastmail serves contacts under `/dav/` and 404s everything else), the `server`/`discover` routes probe `.well-known/carddav` and follow its redirect before the principal walk whenever the resolved path is `/`.
