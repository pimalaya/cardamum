//! # Pimdir CLI
//!
//! The `pimdir` command family, reporting on the local store the
//! account reads from, reading and retracting the writes it queued, and
//! searching it through its index.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    account::context::Account,
    pimdir::{
        backend::PimdirBackend, count::PimdirCountCommand, queue::cli::PimdirQueueCommand,
        search::PimdirSearchCommand, status::PimdirStatusCommand,
    },
};

/// Report on, queue into and search the local pimdir store behind the
/// account.
///
/// What the store holds and how much of it is downloaded, the writes
/// cardamum queued for its owner and those the owner parked rather than
/// applied, each of which can be cancelled. Searches and counts the
/// account's cards through the store's search index, which its sync
/// engine builds.
///
/// The store's own operator tooling, the `pimdir` binary io-pimdir
/// ships, covers the rest, including retrying a parked write and the
/// retained items.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PimdirCommand {
    Status(PimdirStatusCommand),
    #[command(subcommand)]
    Queue(PimdirQueueCommand),
    Search(PimdirSearchCommand),
    Count(PimdirCountCommand),
}

impl PimdirCommand {
    /// Runs the subcommand against the pimdir store of the account
    /// `name`.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        name: &str,
        account: &Account,
        backend: PimdirBackend,
    ) -> Result<()> {
        match self {
            Self::Status(cmd) => cmd.execute(printer, backend),
            Self::Queue(cmd) => cmd.execute(printer, name, &backend),
            Self::Search(cmd) => cmd.execute(printer, account, &backend),
            Self::Count(cmd) => cmd.execute(printer, &backend),
        }
    }
}
