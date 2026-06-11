use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::vdir::{
    client::VdirClient, create::VdirCollectionCreateCommand, delete::VdirCollectionDeleteCommand,
    list::VdirCollectionListCommand, rename::VdirCollectionRenameCommand,
};

/// Vdir-specific API.
///
/// Gives access to the raw vdir filesystem API on the active
/// account's `vdir.home-dir` (collections + items as files on disk).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum VdirCommand {
    Create(VdirCollectionCreateCommand),
    Rename(VdirCollectionRenameCommand),
    Delete(VdirCollectionDeleteCommand),
    List(VdirCollectionListCommand),
}

impl VdirCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Rename(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::List(cmd) => cmd.execute(printer, client),
        }
    }
}
