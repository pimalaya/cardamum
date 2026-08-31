//! # cardamum
//!
//! CLI to manage contacts. It writes no protocol or storage logic of its
//! own: it drives the sans-I/O io-* libraries below it, consuming their
//! blocking clients and rendering what they return.
//!
//! ## Layout
//!
//! The frontend: main dispatches and meets the bare invocation, [`cli`]
//! declares the parser and resolves the account a command runs against,
//! [`config`] parses the TOML accounts and renders a generated one back,
//! [`account`] inspects and validates them.
//!
//! The command tree splits in three. The shared API (`addressbook`,
//! `card`) is the cross-protocol least-common-denominator surface,
//! behaving the same whatever backend serves the account.
//!
//! The protocol-specific APIs (`carddav`, `jmap`, `msgraph`, `people`,
//! `vdir`) each expose the full surface of one backend, including what
//! the shared API cannot model. The meta commands (`configure`,
//! `account`, `completions`, `manuals`) cover the rest.
//!
//! pimdir has no protocol-specific surface by design: it is a store
//! rather than a protocol, and its operator commands ship as the
//! separate pimdir binary.
//!
//! The shared commands run over [`shared::client`]'s
//! `AddressbookClient`, one `BackendClient` variant per compiled-in
//! backend. The global `--backend` flag ([`backend`]) picks it: `auto`
//! takes the first configured one, a named value bails without it.
//!
//! Each shared method matches the active backend and calls its
//! per-protocol backend.rs adapter, which maps the shared
//! [`shared::addressbook`] and [`shared::card`] types onto that protocol
//! crate's client.
//!
//! The backends: carddav over io-webdav, jmap over io-jmap, msgraph over
//! io-msgraph, people over io-people, vdir over io-vdir and pimdir over
//! io-pimdir, each behind its own cargo feature.
//!
//! The cross-backend layer is owned here, not by a per-domain
//! aggregator crate, per the org's aggregator-retirement decision: the
//! interface aggregates, the protocol crates stay leaf libraries, and a
//! partial concept lives in a protocol command, not an ownerless API.
//!
//! CardDAV, vdir and pimdir speak vCard natively, JMAP, Graph and People
//! do not. For those three the shared card contents is a vCard this
//! crate synthesizes and re-projects on the way back, in [`project`] and
//! the per-backend project.rs.
//!
//! JMAP converts through vcard-rs's JSContact codec, Graph and People
//! project field by field with a provider-side stash for the properties
//! that have no slot. Those modules are ported from cardamum-android, so
//! both products treat the same quirks identically.
//!
//! The [`wizard`]: one prompt takes an email address, a server URL or a
//! folder path, io-pim-discovery turns it into the reachable services,
//! and the chosen backend's module prompts its credential. A file, an
//! appended block or a document on stdout is then configure's call.
//!
//! Output follows the Pimalaya rule: data and errors go to stdout
//! through the printer, `--json` switching every command to JSON, and
//! stderr carries logs only.
//!
//! Each command's doc comment is its `--help` text and ends with its
//! JSON output shape, so `cardamum <command> --help` is the canonical
//! per-command reference. The behavioural truth behind this header lives
//! under cairn/spec, one file per capability.

mod account;
mod backend;
#[cfg(feature = "carddav")]
mod carddav;
mod cli;
mod config;
#[cfg(feature = "jmap")]
mod jmap;
mod json_schema;
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
/// A missing configuration raises the offer, everything else gets the
/// help: a script, a JSON caller, `--account`, which reads as a
/// half-typed command, and a file that fails to parse, which counts as a
/// configuration so the offer never writes over a broken one.
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

        // NOTE: nothing to run after the offer, so a declined one falls
        // back to the help; the wizard says what to run next when it ran.
        if cli::offer_configuration(printer, config_paths, &path)? {
            return Ok(());
        }
    }

    Cli::command().print_help()?;

    Ok(())
}
