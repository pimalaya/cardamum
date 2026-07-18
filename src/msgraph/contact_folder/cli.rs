use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::MsgraphClient,
    contact_folder::{
        child_folders::MsgraphContactFolderChildFoldersCommand,
        create::MsgraphContactFolderCreateCommand, delete::MsgraphContactFolderDeleteCommand,
        get::MsgraphContactFolderGetCommand, list::MsgraphContactFolderListCommand,
        rename::MsgraphContactFolderRenameCommand,
    },
};

/// Manage Graph contact folders (the addressbooks).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphContactFolderCommand {
    List(MsgraphContactFolderListCommand),
    #[command(visible_aliases = ["children", "child"])]
    ChildFolders(MsgraphContactFolderChildFoldersCommand),
    Get(MsgraphContactFolderGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(MsgraphContactFolderCreateCommand),
    Rename(MsgraphContactFolderRenameCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(MsgraphContactFolderDeleteCommand),
}

impl MsgraphContactFolderCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::ChildFolders(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Rename(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
