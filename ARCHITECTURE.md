# cardamum architecture

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) first: it describes the conventions every Pimalaya repository shares (layering, `no_std`, module and error rules, code style, licensing). This document only covers what is specific to cardamum, and assumes you know that shared context.

If a statement here conflicts with the code, the code wins; please flag it.

## Where cardamum fits

cardamum is an **application**, the top layer of the Pimalaya stack: it has no library target (only `main.rs`) and writes no protocol or storage logic of its own. It is a thin CLI shell that drives the sans-I/O libraries below it:

- [io-addressbook](https://github.com/pimalaya/io-addressbook): the cross-protocol addressbook/card domain API;
- [io-webdav](https://github.com/pimalaya/io-webdav): the CardDAV (WebDAV) protocol coroutines;
- [io-vdir](https://github.com/pimalaya/io-vdir): the local vdir filesystem coroutines;
- [pimconf](https://github.com/pimalaya/pimconf): CardDAV server discovery (PACC, and RFC 6764 SRV + TXT + `.well-known`);
- [pimalaya-cli](https://github.com/pimalaya/cli), [pimalaya-config](https://github.com/pimalaya/config), [pimalaya-stream](https://github.com/pimalaya/stream): shared CLI plumbing (clap args, printer, logger), TOML config loading, and the blocking I/O runtime.

The sans-I/O coroutines live in those libraries; cardamum never implements one. It consumes their blocking `*Std` clients (`AddressbookClientStd`, `WebdavClientStd`, `VdirClient`), which run pimalaya-stream under the hood. So all real I/O (network, filesystem, clock, DNS) is concentrated in the libraries; cardamum only orchestrates them and renders results.

## Three command families

The command tree (`cli.rs`) is split into three groups, in this order:

1. **Shared API** (`addressbook`, `card`): the cross-protocol, least-common-denominator surface. Every operation here works the same regardless of which backend serves the active account. Backend-specific concepts that are not common to both backends do not appear here.
2. **Protocol-specific APIs** (`carddav`, `vdir`): each exposes the full surface of one backend, including operations the shared API cannot model (`carddav discover`, `carddav propfind`/`report`, `vdir rename`). Each is gated behind its own cargo feature.
3. **Meta** (`account`, `completions`, `manuals`): account configuration and inspection, shell completions, and man pages.

This is the standard Pimalaya CLI split: a portable shared API plus per-protocol escape hatches.

## Backend selection

The shared commands target a backend chosen by the global `--backend` flag, a `Backend` enum (`backend.rs`) with `auto` (default), `vdir` and `carddav` variants (each named variant gated behind its feature):

- `auto` picks the first configured-and-allowed backend in cardamum's priority order;
- a named value pins the command to that backend, and bails if the account has no matching config block or the operation has no arm for it.

The shared commands receive an `AddressbookClient` (`shared/client.rs`): a wrapper that holds the merged `Account` plus an inner `AddressbookClientStd`. `AddressbookClientStd` is an enum holding exactly one backend, built by trying each allowed backend in turn and wrapping the first configured one via `From`. The wrapper `Deref`s onto the inner client, so command code calls the io-addressbook API directly. The protocol-specific commands skip this entirely and build their own `CarddavClient` / `VdirClient`, ignoring `--backend`.

## Command conventions

Each subcommand is a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method (the shared nested-execute convention). `Command::execute` in `cli.rs` is the single dispatch point: it loads the config (running the wizard if none exists), selects the account, builds the appropriate client, and hands it to the subcommand.

Output follows the Pimalaya stdout/stderr rule: all data and errors go to stdout through `pimalaya_cli::printer` (with `--json` switching every command to JSON), and stderr carries logs only. A command returns a `Serialize + Display` type to the printer rather than printing inline.

## Configuration and the wizard

Config is loaded by pimalaya-config from the first existing path among the three canonical locations (or the `-c` / `CARDAMUM_CONFIG` override), with later paths deep-merged on top of the first. The schema is multi-account: a top-level block plus named `[accounts.<name>]` blocks, each carrying an optional `[carddav]` and/or `[vdir]` sub-block. `Account::from(config).merge(Account::from(account_config))` flattens the global defaults under the selected account.

When no config file exists, `load_or_wizard` runs the interactive wizard (`wizard/`) to bootstrap one, prompting for an account, then walking the vdir or CardDAV setup before writing the file at the target path.

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
  backend.rs             Backend enum (auto/vdir/carddav) + selection rules
  config.rs              TOML schema: Config, AccountConfig, per-backend blocks
  shared/                cross-protocol least-common-denominator API
    client.rs            AddressbookClient wrapper (picks one backend)
    addressbook/         addressbook list/create/update/delete
    card/                card list/read/create/update/delete (+ vcard helper)
  carddav/               [carddav] protocol-specific API
    client.rs            WebdavClientStd builder + discovery routes
    discover/propfind/report/list/create/delete
  vdir/                  [vdir] protocol-specific API
    client.rs            VdirClient builder
    list/create/rename/delete
  account/               account list/check/configure + Account context
  wizard/                first-run interactive config bootstrap
```

`shared/` is the portable surface; `carddav/` and `vdir/` are the per-protocol escape hatches; `account/` and `wizard/` are the meta and bootstrap concerns.
