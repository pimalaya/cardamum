//! # cardamum
//!
//! CLI to manage contacts. cardamum is an **application**, the top layer
//! of the Pimalaya stack: it has no library target (only this binary)
//! and writes no protocol or storage logic of its own. It is a thin
//! shell that drives the sans-I/O io-* libraries below it, consuming
//! their blocking `*Std` clients and orchestrating and rendering the
//! results. All real I/O (network, filesystem, clock, DNS) is
//! concentrated in those libraries; cardamum never implements a
//! coroutine.
//!
//! ## Backends and plumbing
//!
//! The network backends are io-webdav (CardDAV, WebDAV), io-jmap (RFC
//! 8620 + RFC 9610), io-msgraph (Microsoft Graph) and io-people (Google
//! People); the local storage backend is io-vdir. calcard and vcard-rs
//! parse and build vCard/JSContact for the backends with no native
//! vCard. Account discovery comes from io-pim-discovery (fixed provider
//! rules, PACC, RFC 6764 CardDAV resolve, RFC 8620 JMAP resolve, a
//! `WWW-Authenticate` probe). The CLI plumbing (clap args, printer,
//! logger), TOML config loading and the blocking stream runtime come
//! from pimalaya-cli, pimalaya-config and pimalaya-stream. Every backend
//! sits behind its own cargo feature, so a build ships only the
//! protocols it needs.
//!
//! ## No aggregator crate
//!
//! cardamum owns its cross-backend abstraction rather than depending on
//! a per-domain aggregator library (the retired io-addressbook). Per the
//! org's aggregator-retirement decision, the least-common-denominator
//! layer is a *product* decision with a single owner: the interface
//! aggregates, the protocol crates stay leaf libraries. Partial-coverage
//! concepts (CardDAV ETags, JMAP m:n memberships, Graph delta) then live
//! in a product-owned protocol-specific command instead of being ejected
//! from an ownerless shared API.
//!
//! ## Command families
//!
//! The command tree ([`cli`], `Command`) splits into three groups. The
//! shared API (`addressbook`, `card`) is the cross-protocol
//! least-common-denominator surface, behaving the same whatever backend
//! serves the active account. The protocol-specific APIs (`carddav`,
//! `vdir`) each expose the full surface of one backend, including
//! operations the shared API cannot model (`carddav propfind`/`report`,
//! `vdir rename`). The meta commands (`account`, `completions`,
//! `manuals`) cover account inspection, shell completions and man pages.
//!
//! ## Shared commands and backend selection
//!
//! The shared commands run over a cross-protocol [`shared::client`]
//! `AddressbookClient` that owns one `BackendClient` enum variant per
//! compiled-in backend. The global `--backend` flag ([`backend`]) picks
//! it: `auto` (the default) takes the first configured-and-allowed
//! backend in priority order, a named value pins that backend and bails
//! when the account has no matching config block or the operation has no
//! arm for it. Each shared method matches the active backend and calls
//! its per-protocol `backend.rs` adapter, which maps the shared
//! [`shared::addressbook`] / [`shared::card`] types onto that protocol
//! crate's `*Std` client. The protocol-specific commands skip this and
//! build their own `CarddavClient` / `VdirClient`, ignoring `--backend`.
//!
//! ## vCard projection
//!
//! CardDAV and vdir speak vCard natively; JMAP, Microsoft Graph and
//! Google People do not. For those three, the shared `Card.contents` is
//! a vCard document of record that cardamum *synthesizes* from the
//! backend's own contact resource and re-projects on the way back
//! (`{jmap,msgraph,google}/project.rs`): JMAP ContactCards convert
//! through calcard's JSContact codec, while Graph and People contacts
//! project field-by-field with a provider-side stash for the properties
//! that have no first-class slot, so nothing is lost round-trip. These
//! modules are ported verbatim from cardamum-android, so both products
//! treat the same provider quirks identically.
//!
//! ## Configuration and the wizard
//!
//! Config is loaded by pimalaya-config from the first existing canonical
//! path (or the `-c` / `CARDAMUM_CONFIG` override), later paths
//! deep-merged on top; the schema ([`config`]) is multi-account, a
//! top-level block plus named `[accounts.<name>]` blocks each carrying
//! one backend sub-block. `cli::resolve_account` selects the account
//! (`-a` or `default`); a config that exists but lacks it is a hard
//! error. When no config exists, the wizard is proposed; bare `cardamum`
//! (no subcommand) also runs it. The [`wizard`] writes nothing to disk:
//! from a single email / server-URL / vdir-path prompt it discovers an
//! account, tests it, and prints it as a ready-to-save TOML document on
//! stdout (prompts on stderr), so `cardamum > <config>` is the
//! write-back, exactly like Ortie.
//!
//! ## Output
//!
//! Output follows the Pimalaya rule: all data and errors go to stdout
//! through the printer (`--json` switches every command to JSON), stderr
//! carries logs only. Each command's doc comment is its `--help` text
//! and ends with its JSON output shape, so `cardamum <command> --help`
//! is the canonical per-command usage reference; the README documents no
//! per-command usage. The module map and the deeper design notes (the
//! CardDAV discovery routes, the wizard internals) live under docs/.

mod account;
mod backend;
#[cfg(feature = "carddav")]
mod carddav;
mod cli;
mod config;
#[cfg(feature = "google")]
mod google;
#[cfg(feature = "jmap")]
mod jmap;
#[cfg(feature = "msgraph")]
mod msgraph;
#[cfg(any(feature = "msgraph", feature = "google"))]
mod project;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;
mod wizard;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

use crate::{cli::Cli, wizard::discover};

fn main() {
    let cli = Cli::parse();
    let mut printer = StdoutPrinter::new(&cli.json);
    let result = execute(cli, &mut printer);
    ErrorReport::eval(&mut printer, result);
}

fn execute(cli: Cli, printer: &mut StdoutPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;
    let config = cli.config_paths.as_ref();
    let account = cli.account.name.as_deref();
    let backend = cli.backend;

    match cli.cmd {
        Some(cmd) => cmd.execute(printer, config, account, backend),
        None => discover::run(printer),
    }
}
