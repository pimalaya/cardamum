//! # cardamum
//!
//! CLI to manage contacts. It writes no protocol or storage logic of its
//! own: it drives the sans-I/O io-* libraries below it, consuming their
//! blocking clients and rendering what they return.
//!
//! ## Layout
//!
//! The frontend: main dispatches, meeting a person missing a
//! configuration with the welcome, [`cli`] declares the parser and
//! resolves the account a command runs against, [`config`] parses the
//! TOML accounts, [`account`] inspects and validates them. Every command
//! runs over a [`session`], which keeps the unified client across the
//! commands of a [`repl`] and the tool calls of an [`mcp`] server.
//!
//! The command tree splits in three. The unified API (`addressbook`,
//! `card`) is the cross-protocol surface, serving the union of what the
//! backends can do: [`capability`] declares each backend's level per
//! verb, and a verb the account's backend does not serve is refused
//! before connecting.
//!
//! The protocol-specific APIs (`carddav`, `jmap`, `msgraph`, `gpeople`,
//! `vdir`) each expose the full surface of one backend, including what
//! the unified API cannot model. The meta commands (`account`, `repl`,
//! `mcp`, `completion`, `manual`, `json-schema`) cover the rest.
//!
//! pimdir is a store rather than a protocol: its `pimdir` family reports
//! on the store (`status`), reads and retracts the writes cardamum queued
//! (`queue list`, `show`, `cancel`) and queries its search index
//! (`search`, `count`), and its other operator commands ship as the
//! separate pimdir binary.
//!
//! The unified commands run over [`shared::client`]'s
//! `AddressbookClient`, one `BackendClient` variant per compiled-in
//! backend. The global `--backend` flag ([`backend`]) picks it: `auto`
//! takes the first configured one, a named value bails without it.
//!
//! Each unified method matches the active backend and calls its
//! per-protocol backend.rs adapter, which maps the shared
//! [`shared::addressbook`] and [`shared::card`] types onto that protocol
//! crate's client.
//!
//! The backends: carddav over io-webdav, jmap over io-jmap, msgraph over
//! io-msgraph, people over io-gpeople, vdir over io-vdir and pimdir over
//! io-pimdir, each behind its own cargo feature.
//!
//! The cross-backend layer is owned here, not by a per-domain
//! aggregator crate, per the org's aggregator-retirement decision: the
//! interface aggregates, the protocol crates stay leaf libraries, and a
//! partial concept lives in a protocol command, not an ownerless API.
//!
//! CardDAV, vdir and pimdir speak vCard natively, JMAP, Graph and People
//! do not. For those three the shared card contents is a synthesized
//! vCard, re-projected on the way back.
//!
//! JMAP converts through vcard-rs's JSContact codec in src/jmap/project.rs.
//! Graph and People project field by field with a provider-side stash for
//! the properties that have no slot, through the `vcard` features of
//! io-msgraph and io-gpeople, which Neverest shares.
//!
//! Output follows the Pimalaya rule: data and errors go to stdout
//! through the printer, `--json` switching every command to JSON, and
//! stderr carries logs only. Text is plain: pimalaya-cli's columns,
//! records and fields, no border, no colour.
//!
//! Each command's doc comment is its `--help` text and ends with its
//! JSON output shape, so `cardamum <command> --help` is the canonical
//! per-command reference. The behavioural truth behind this header lives
//! under cairn/spec, one file per capability.

mod account;
mod backend;
mod capability;
#[cfg(feature = "carddav")]
mod carddav;
mod cli;
mod config;
mod error;
#[cfg(feature = "gpeople")]
mod gpeople;
#[cfg(feature = "jmap")]
mod jmap;
mod json_schema;
mod mcp;
#[cfg(feature = "msgraph")]
mod msgraph;
#[cfg(feature = "pimdir")]
mod pimdir;
mod repl;
#[cfg(all(
    test,
    any(
        feature = "carddav",
        feature = "gpeople",
        feature = "jmap",
        feature = "msgraph"
    )
))]
mod script;
mod session;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;

use std::process;

use anyhow::Result;
use clap::CommandFactory;
use pimalaya_cli::{
    clap::{meet_bare_invocation, parse},
    error::{EXIT_FAILURE, ErrorReport},
    log::Logger,
    printer::StdoutPrinter,
    welcome::{is_wanted, shipped_sample},
};
use pimalaya_config::toml::{ConfigMissingError, TomlConfig};
use simd_cpu::SimdCpu;

use crate::{
    cli::Cli,
    config::Config,
    error::ErrorCode,
    session::{Session, SessionAccount},
};

fn main() {
    // NOTE: the vCard codecs read the installed CPU token, the portable
    // baseline until one is installed.
    let _ = SimdCpu::install_detected();

    let cli = parse::<Cli>();
    let mut printer = StdoutPrinter::new(&cli.json);
    let config_paths = cli.config.paths.clone();
    let result = execute(cli, &mut printer);

    // A person missing a configuration meets the welcome, a script or a
    // JSON caller the error.
    if let Err(err) = &result
        && let Some(missing) = err.downcast_ref::<ConfigMissingError>()
        && is_wanted(&printer)
    {
        let sample = shipped_sample("cardamum");
        let welcome = cli::welcome(&missing.path, &config_paths, sample.as_deref());
        eprint!("{welcome}");
        process::exit(EXIT_FAILURE);
    }

    ErrorReport::<ErrorCode>::eval_coded(&mut printer, result);
}

/// Runs the subcommand, or meets a bare `cardamum`.
fn execute(cli: Cli, printer: &mut StdoutPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;
    let account = SessionAccount {
        config_paths: &cli.config.paths,
        name: cli.account.name.as_deref(),
        backend: cli.backend,
    };

    let Some(cmd) = cli.cmd else {
        let config_missing = || {
            let config = Config::load(account.config_paths);
            config.err().filter(|err| err.is::<ConfigMissingError>())
        };
        return meet_bare_invocation(printer, Cli::command(), config_missing);
    };

    let mut session = Session {
        account,
        ..Default::default()
    };

    cmd.execute(printer, &mut session)
}
