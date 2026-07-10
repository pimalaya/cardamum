# cardamum architecture

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) first: it describes the conventions every Pimalaya repository shares (layering, `no_std`, module and error rules, code style, licensing). This document only covers what is specific to cardamum, and assumes you know that shared context.

If a statement here conflicts with the code, the code wins; please flag it.

## Where cardamum fits

cardamum is an **application**, the top layer of the Pimalaya stack: it has no library target (only `main.rs`) and writes no protocol or storage logic of its own. It is a thin CLI shell that drives the sans-I/O protocol libraries below it, one per backend:

- [io-webdav](https://github.com/pimalaya/io-webdav): the CardDAV (WebDAV) protocol coroutines;
- [io-jmap](https://github.com/pimalaya/io-jmap): the JMAP contacts coroutines (RFC 8620 + RFC 9610);
- [io-msgraph](https://github.com/pimalaya/io-msgraph): the Microsoft Graph contacts coroutines;
- [io-google-people](https://github.com/pimalaya/io-google-people): the Google People API coroutines;
- [io-vdir](https://github.com/pimalaya/io-vdir): the local vdir filesystem coroutines;
- [calcard](https://crates.io/crates/calcard) and [vcard-rs](https://github.com/pimalaya/vcard): vCard/JSContact parsing and building, for the backends with no native vCard;
- [pimconf](https://github.com/pimalaya/pimconf): email-driven service discovery (fixed provider rules, PACC, RFC 6764 DAV resolve, RFC 8620 JMAP resolve, `WWW-Authenticate` probe);
- [pimalaya-cli](https://github.com/pimalaya/cli), [pimalaya-config](https://github.com/pimalaya/config), [pimalaya-stream](https://github.com/pimalaya/stream): shared CLI plumbing (clap args, printer, logger), TOML config loading, and the blocking I/O runtime.

The sans-I/O coroutines live in those libraries; cardamum never implements one. It consumes their blocking `*Std` clients (`WebdavClientStd`, `JmapClientStd`, `MsgraphClientStd`, `PeopleClientStd`, `VdirClient`), which run pimalaya-stream under the hood. So all real I/O (network, filesystem, clock, DNS) is concentrated in the libraries; cardamum only orchestrates them and renders results.

**No aggregator crate.** cardamum owns its cross-backend abstraction rather than depending on a per-domain aggregator library (the retired io-addressbook). Per the org's [aggregator retirement](https://github.com/pimalaya/.github) decision, the least-common-denominator layer is a *product* decision with a single owner: the interface aggregates, the protocol crates stay leaf libraries. The upside is that partial-coverage concepts (CardDAV ETags, JMAP m:n memberships, Graph delta) can live in a product-owned protocol-specific command instead of being ejected from an ownerless shared API.

## Three command families

The command tree (`cli.rs`) is split into three groups, in this order:

1. **Shared API** (`addressbook`, `card`): the cross-protocol, least-common-denominator surface. Every operation here works the same regardless of which backend serves the active account. Backend-specific concepts that not every backend shares do not appear here.
2. **Protocol-specific APIs** (`carddav`, `vdir`): each exposes the full surface of one backend, including operations the shared API cannot model (`carddav discover`, `carddav propfind`/`report`, `vdir rename`). Each is gated behind its own cargo feature.
3. **Meta** (`account`, `completions`, `manuals`): account configuration and inspection, shell completions, and man pages.

This is the standard Pimalaya CLI split: a portable shared API plus per-protocol escape hatches.

## Backend selection

The shared commands target a backend chosen by the global `--backend` flag, a `Backend` enum (`backend.rs`) with `auto` (default), `carddav`, `jmap`, `msgraph`, `google` and `vdir` variants (each named variant gated behind its feature):

- `auto` picks the first configured-and-allowed backend in cardamum's priority order;
- a named value pins the command to that backend, and bails if the account has no matching config block or the operation has no arm for it.

The shared commands receive an `AddressbookClient` (`shared/client.rs`): a struct holding the merged `Account` plus a `BackendClient` enum with exactly one variant per compiled-in backend. It is built by trying each allowed backend in turn and constructing the first configured one; every shared-API method is a `match` dispatching to the active backend's glue. That glue is one `backend.rs` module per protocol folder (`carddav/backend.rs`, `jmap/backend.rs`, `msgraph/backend.rs`, `google/backend.rs`, `vdir/backend.rs`): each maps the shared `Addressbook`/`Card` operations (defined in `shared/addressbook` and `shared/card`) onto its protocol crate's `*Std` client. The protocol-specific commands skip this entirely and build their own `CarddavClient` / `VdirClient`, ignoring `--backend`.

## vCard projection

CardDAV and vdir speak vCard natively; JMAP, Microsoft Graph and Google People do not. For those three, the shared `Card.contents` is a vCard document of record that cardamum *synthesizes* from the backend's own contact resource, and re-projects on the way back:

- `jmap/project.rs` converts a JMAP ContactCard (JSContact, RFC 9553/9555) to and from vCard through calcard, computing a JSON-hash ETag and building minimal `ContactCard/set` patches against a base;
- `msgraph/project.rs` and `google/project.rs` build a vCard from the Graph/People JSON contact and back, stashing every property with no first-class slot in a provider-side extended property / clientData entry so nothing is lost round-trip. Fields shared by `project.rs` (canonical properties, date normalization, stash splicing, RFC 6350 escaping) live at the crate root.

These modules are ported from cardamum-android, which pioneered the projections; the CLI reuses them verbatim so both products treat the same provider quirks identically.

## Command conventions

Each subcommand is a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method (the shared nested-execute convention). `Command::execute` in `cli.rs` is the single dispatch point: it loads the config (running the wizard if none exists), selects the account, builds the appropriate client, and hands it to the subcommand.

Output follows the Pimalaya stdout/stderr rule: all data and errors go to stdout through `pimalaya_cli::printer` (with `--json` switching every command to JSON), and stderr carries logs only. A command returns a `Serialize + Display` type to the printer rather than printing inline.

Each command's doc comment is its `--help` text: the first paragraph is the short summary shown by `-h`, and the full text (shown by `--help`) ends with the command's JSON output shape. So `cardamum <command> --help` is the canonical usage reference for both humans and AI agents; the README intentionally documents no per-command usage.

## Configuration and the wizard

Config is loaded by pimalaya-config from the first existing path among the three canonical locations (or the `-c` / `CARDAMUM_CONFIG` override), with later paths deep-merged on top of the first. The schema is multi-account: a top-level block plus named `[accounts.<name>]` blocks, each carrying exactly one backend sub-block (`[carddav]`, `[jmap]`, `[msgraph]`, `[google]` or `[vdir]`). Auth blocks and the `Secret` shape (`command` shell line or `raw` plaintext) match himalaya's, so a token retrieved from Ortie is configured the same way across tools. `Account::from(config).merge(Account::from(account_config))` flattens the global defaults under the selected account.

When no config file exists, `load_or_wizard` runs the interactive wizard (`wizard/`) to bootstrap one, prompting for an account, then walking the chosen backend's setup before writing the file at the target path.

## The wizard and email discovery

The wizard (`wizard/account.rs`) offers a backend picker whose first entry is **automatic discovery**. That path (`wizard/search.rs`) feeds the email address to pimconf's parallel `SearchClientStd`, which runs every discovery mechanism at once (fixed Google/Microsoft provider rules, PACC, RFC 6764 CardDAV resolve, RFC 8620 JMAP resolve, refined by a `WWW-Authenticate` probe). Each discovered service and authentication-method combination becomes one selectable entry, mirroring the cardamum-android configuration screen; Google and Microsoft accounts additionally surface their proprietary People/Graph APIs, which no record advertises. Picking an entry pins the endpoint and auth method, so the per-backend flow (`wizard/{carddav,jmap,msgraph,google}.rs`) only prompts for the secret and tests the connection before saving. The manual picker entries run the same per-backend flows without a pinned endpoint.

## CardDAV discovery

The `[carddav]` block resolves the addressbook home set through one of three routes, in decreasing order of magic (`carddav/client.rs`, `open_carddav_client`):

- `home`: short-circuits all discovery; the configured URL is used as the addressbook home set directly;
- `server`: connects to the given context root, then walks current-user-principal (RFC 5397) and addressbook-home-set (RFC 6352);
- `discover`: resolves a bare domain to a context root before running that same walk: PACC first, then RFC 6764 (SRV record, its TXT `path`, then `.well-known`) through pimconf; Google domains use an authenticated `.well-known` probe.

## Module layout

```
src/
  main.rs                entry point: parse Cli, build printer, dispatch
  cli.rs                 Cli/Command, global flags, execute dispatch
  backend.rs             Backend enum (auto/carddav/jmap/msgraph/google/vdir)
  config.rs              TOML schema: Config, AccountConfig, per-backend blocks
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
    backend.rs           shared-API glue over io-google-people
    project.rs           People person <-> vCard
  vdir/                  [vdir] backend + protocol-specific API
    client.rs            VdirClient builder
    backend.rs           shared-API glue over io-vdir
    list/create/rename/delete
  account/               account list/check/configure + Account context
  wizard/                first-run interactive config bootstrap + email discovery
```

`shared/` is the portable surface; each protocol folder holds its `backend.rs` glue (and, for CardDAV/vdir, its per-protocol escape-hatch commands); `account/` and `wizard/` are the meta and bootstrap concerns.
