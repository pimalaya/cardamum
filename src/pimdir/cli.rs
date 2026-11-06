//! # Pimdir CLI
//!
//! The `pimdir` command family, reporting on the local store the
//! account reads from and searching it through its index.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    account::context::Account,
    pimdir::{
        backend::PimdirBackend, count::PimdirCountCommand, search::PimdirSearchCommand,
        status::PimdirStatusCommand,
    },
};

/// Report on and search the local pimdir store behind the account.
///
/// What the store holds, how much of it is downloaded, and the writes
/// its owner parked rather than applied. Searches and counts the
/// account's cards through the store's search index, which its sync
/// engine builds.
///
/// The store's own operator tooling, the `pimdir` binary io-pimdir
/// ships, covers the rest, including retrying or cancelling a parked
/// write and the retained items.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PimdirCommand {
    Status(PimdirStatusCommand),
    Search(PimdirSearchCommand),
    Count(PimdirCountCommand),
}

impl PimdirCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        account: &Account,
        backend: PimdirBackend,
    ) -> Result<()> {
        match self {
            Self::Status(cmd) => cmd.execute(printer, backend),
            Self::Search(cmd) => cmd.execute(printer, account, &backend),
            Self::Count(cmd) => cmd.execute(printer, &backend),
        }
    }
}
