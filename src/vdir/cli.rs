//! # Vdir command
//!
//! Declares the vdir command tree and dispatches each subcommand against
//! the account's vdir client.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::vdir::{
    client::VdirClient, create::VdirCollectionCreateCommand, delete::VdirCollectionDeleteCommand,
    item::cli::VdirItemCommand, list::VdirCollectionListCommand,
    rename::VdirCollectionRenameCommand,
};

/// Manage vdir collections and items on the filesystem.
///
/// Operates on the raw vdir API rooted at the active account's
/// `vdir.home-dir`. The flat verbs act on collections, which are
/// directories, and `item` on the raw item files inside them.
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
