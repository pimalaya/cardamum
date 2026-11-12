//! # cardamum
//!
//! CLI to manage contacts. It writes no protocol or storage logic of its
//! own: it drives the sans-I/O io-* libraries below it, consuming their
//! blocking clients and rendering what they return.
//!
//! ## Layout
//!
//! The frontend: main dispatches and prints the help on a bare
//! invocation, [`cli`] declares the parser and resolves the account a
//! command runs against, [`config`] parses the TOML accounts, [`account`]
//! inspects and validates them.
//!
//! The command tree splits in three. The unified API (`addressbook`,
//! `card`) is the cross-protocol least-common-denominator surface,
//! behaving the same whatever backend serves the account.
//!
//! The protocol-specific APIs (`carddav`, `jmap`, `msgraph`, `gpeople`,
//! `vdir`) each expose the full surface of one backend, including what
//! the unified API cannot model. The meta commands (`account`,
//! `completions`, `manuals`, `json-schema`) cover the rest.
//!
//! pimdir is a store rather than a protocol: its `pimdir` family reports
//! on the store (`status`, the parked writes included) and queries its
//! search index (`search`, `count`), and its operator commands ship as
//! the separate pimdir binary.
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
mod error;
#[cfg(feature = "gpeople")]
mod gpeople;
#[cfg(feature = "jmap")]
mod jmap;
mod json_schema;
#[cfg(feature = "msgraph")]
mod msgraph;
#[cfg(feature = "pimdir")]
mod pimdir;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;

use std::{path::PathBuf, process};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    error::{EXIT_FAILURE, ErrorReport},
    log::Logger,
    printer::StdoutPrinter,
    welcome::{is_wanted, shipped_sample},
};
use pimalaya_config::toml::ConfigMissingError;

use crate::{cli::Cli, error::ErrorCode};

fn main() {
    let cli = Cli::parse();
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
    let config: &[PathBuf] = &cli.config.paths;
    let account = cli.account.name.as_deref();
    let backend = cli.backend;

    let Some(cmd) = cli.cmd else {
        return cli::meet_bare_invocation(config);
    };

    cmd.execute(printer, config, account, backend)
}
