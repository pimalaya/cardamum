//! # Microsoft Graph commands
//!
//! Dispatches the Graph-specific command tree, one subtree per Graph
//! resource.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::MsgraphClient, contact_folders::cli::MsgraphContactFoldersCommand,
    contacts::cli::MsgraphContactsCommand, profile::cli::MsgraphProfileCommand,
    request::MsgraphRequestCommand,
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
    #[command(subcommand, visible_alias = "folders", aliases = ["contact-folder", "folder"])]
    ContactFolders(MsgraphContactFoldersCommand),
    #[command(subcommand, alias = "contact")]
    Contacts(MsgraphContactsCommand),
    #[command(subcommand)]
    Profile(MsgraphProfileCommand),
    Request(MsgraphRequestCommand),
}

impl MsgraphCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        match self {
            Self::ContactFolders(cmd) => cmd.execute(printer, client),
            Self::Contacts(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
        }
    }
}
