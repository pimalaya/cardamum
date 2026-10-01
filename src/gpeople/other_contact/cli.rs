//! # Other contact commands
//!
//! Dispatches the other contact commands to the People operation each
//! runs.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{
    client::GpeopleClient,
    other_contact::{
        copy::GpeopleOtherContactCopyCommand, list::GpeopleOtherContactListCommand,
        search::GpeopleOtherContactSearchCommand,
    },
};

/// The "other contacts": people interacted with but never added
/// (`otherContacts`).
///
/// Read-only, except `copy`.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GpeopleOtherContactCommand {
    List(GpeopleOtherContactListCommand),
    Search(GpeopleOtherContactSearchCommand),
    Copy(GpeopleOtherContactCopyCommand),
}

impl GpeopleOtherContactCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GpeopleClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Search(cmd) => cmd.execute(printer, client),
            Self::Copy(cmd) => cmd.execute(printer, client),
        }
    }
}
