//! # pimdir queue command
//!
//! The `pimdir queue` command, dispatching onto its subcommands.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::pimdir::{
    backend::PimdirBackend,
    queue::{
        cancel::PimdirQueueCancelCommand, list::PimdirQueueListCommand,
        show::PimdirQueueShowCommand,
    },
};

/// Read and retract the writes cardamum staged.
///
/// A pimdir store is a replica the sync engine owns, so a write is
/// appended to the store's queue and applied on the engine's next run.
///
/// A staged update or deletion shows in the ordinary listing straight
/// away. A staged creation has no id until the engine applies it, which
/// is what these commands are for, and so is a write the engine parked.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PimdirQueueCommand {
    #[command(alias = "ls")]
    List(PimdirQueueListCommand),
    Show(PimdirQueueShowCommand),
    Cancel(PimdirQueueCancelCommand),
}

impl PimdirQueueCommand {
    /// Runs the subcommand against the account's pimdir store.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        account: &str,
        backend: &PimdirBackend,
    ) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, account, backend),
            Self::Show(cmd) => cmd.execute(printer, backend),
            Self::Cancel(cmd) => cmd.execute(printer, backend),
        }
    }
}
