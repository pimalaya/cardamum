use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::vdir::{
    client::VdirClient, create::VdirCollectionCreateCommand, delete::VdirCollectionDeleteCommand,
    item::cli::VdirItemCommand, list::VdirCollectionListCommand,
    rename::VdirCollectionRenameCommand,
};

/// Vdir-specific API.
///
/// Gives access to the raw vdir filesystem API on the active
/// account's `vdir.home-dir`. The flat verbs operate on collections
/// (directories); `item` operates on the raw item files inside them.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum VdirCommand {
    Create(VdirCollectionCreateCommand),
    Rename(VdirCollectionRenameCommand),
    Delete(VdirCollectionDeleteCommand),
    List(VdirCollectionListCommand),
    #[command(subcommand, visible_alias = "items")]
    Item(VdirItemCommand),
}

impl VdirCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Rename(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Item(cmd) => cmd.execute(printer, client),
        }
    }
}
