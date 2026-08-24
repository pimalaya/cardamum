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
//! People); the local storage backends are io-vdir (a filesystem vdir)
//! and io-pimdir (a pimdir store, the SQLite-indexed offline cache a
//! sync engine populates). vcard-rs parses and
//! builds vCard (and converts to/from JSContact) for the backends with
//! no native vCard. Account discovery comes from io-pim-discovery (fixed provider
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
//! `vdir rename`). pimdir has none by design: it is a store rather than
//! a protocol, and its operator surface is the separate `pimdir` binary
//! shipped by io-pimdir. The meta commands (`account`, `completions`,
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
//! CardDAV, vdir and pimdir speak vCard natively; JMAP, Microsoft Graph
//! and Google People do not. For those three, the shared `Card.contents` is
//! a vCard document of record that cardamum *synthesizes* from the
//! backend's own contact resource and re-projects on the way back
//! (`{jmap,msgraph,people}/project.rs`): JMAP ContactCards convert
//! through vcard-rs's JSContact codec, while Graph and People contacts
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
//! error. Bare `cardamum` (no subcommand) runs the interactive
//! [`wizard`], which discovers an account and offers to save it to a
//! config file (or prints it on stdout when redirected); it is also
//! proposed when a command finds no config, and `cardamum configure`
//! runs it by name. Bare `cardamum --account <NAME>` shows the help
//! instead. The wizard mirrors Himalaya's: from a single email /
//! server-URL / folder-path prompt it discovers an account, prompts the
//! authentication method among those the service advertised, tests the
//! account, then writes it as an `[accounts.<name>]` block, creating the
//! config file or appending to the one already there. It configures only
//! what it can discover, stopping with a pointer to config.sample.toml
//! rather than prompting for a hand-entered server field.
//!
//! ## Output
//!
//! Output follows the Pimalaya rule: all data and errors go to stdout
//! through the printer (`--json` switches every command to JSON), stderr
//! carries logs only. Each command's doc comment is its `--help` text
//! and ends with its JSON output shape, so `cardamum <command> --help`
//! is the canonical per-command usage reference; the README documents no
//! per-command usage. The behavioural truth behind this header, one file
//! per capability, lives under cairn/spec.

mod account;
mod backend;
#[cfg(feature = "carddav")]
mod carddav;
mod cli;
mod config;
#[cfg(feature = "jmap")]
mod jmap;
#[cfg(feature = "msgraph")]
mod msgraph;
#[cfg(feature = "people")]
mod people;
#[cfg(feature = "pimdir")]
mod pimdir;
#[cfg(any(feature = "msgraph", feature = "people"))]
mod project;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;
mod wizard;

use std::{
    io::{IsTerminal, stdin},
    path::PathBuf,
};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::Printer, printer::StdoutPrinter};
use pimalaya_config::toml::TomlConfig;

use crate::{cli::Cli, config::Config};

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

    let Some(cmd) = cli.cmd else {
        return meet_bare_invocation(printer, config, account.is_some());
    };

    cmd.execute(printer, config, account, backend)
}

/// Meets a bare `cardamum`, which is where a newcomer lands.
///
/// With no command there is nothing to run: a missing configuration
/// raises the offer, and an existing one gets the help, which is also
/// what a script or a JSON caller gets since neither can answer a
/// prompt. A file that exists but fails to parse counts as a
/// configuration, so the offer never proposes to write over a broken
/// one: the parse error surfaces when a real command reads it.
///
/// `--account` names an account to act on, so with no subcommand it is a
/// half-typed command rather than a first run: it gets the help, which
/// points at the commands, instead of an offer to create an account.
fn meet_bare_invocation(
    printer: &mut StdoutPrinter,
    config_paths: &[PathBuf],
    named_account: bool,
) -> Result<()> {
    let configured = Config::from_paths_or_default(config_paths)
        .ok()
        .flatten()
        .is_some();

    if !configured && !named_account && !printer.is_json() && stdin().is_terminal() {
        let path = Config::target_path(config_paths)?;

        // NOTE: a bare invocation has nothing to run after the offer, so
        // a declined one falls back to the help. The wizard already says
        // what to run next when it ran.
        if cli::offer_configuration(printer, config_paths, &path)? {
            return Ok(());
        }
    }

    Cli::command().print_help()?;

    Ok(())
}
