use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::MsgraphClient, contact::cli::MsgraphContactCommand,
    contact_folder::cli::MsgraphContactFolderCommand, profile::cli::MsgraphProfileCommand,
};

/// Microsoft Graph-specific API.
///
/// Nested by Graph resource (contact folders, contacts, the signed-in
/// user), each command named after its Graph operation. Works with the
/// raw Graph contact model: `create` / `update` take a Graph JSON body
/// and `--json` prints the raw Graph payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphCommand {
    #[command(subcommand, visible_aliases = ["folder", "folders"])]
    ContactFolder(MsgraphContactFolderCommand),
    #[command(subcommand, visible_alias = "contacts")]
    Contact(MsgraphContactCommand),
    #[command(subcommand)]
    Profile(MsgraphProfileCommand),
}

impl MsgraphCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        match self {
            Self::ContactFolder(cmd) => cmd.execute(printer, client),
            Self::Contact(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
        }
    }
}
