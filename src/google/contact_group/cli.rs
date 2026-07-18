use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::google::{
    client::GoogleClient,
    contact_group::{
        create::GoogleContactGroupCreateCommand, delete::GoogleContactGroupDeleteCommand,
        get::GoogleContactGroupGetCommand, list::GoogleContactGroupListCommand,
        members::GoogleContactGroupMembersCommand, update::GoogleContactGroupUpdateCommand,
    },
};

/// Manage People contact groups (the addressbooks).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GoogleContactGroupCommand {
    List(GoogleContactGroupListCommand),
    Get(GoogleContactGroupGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(GoogleContactGroupCreateCommand),
    Update(GoogleContactGroupUpdateCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(GoogleContactGroupDeleteCommand),
    Members(GoogleContactGroupMembersCommand),
}

impl GoogleContactGroupCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GoogleClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::Members(cmd) => cmd.execute(printer, client),
        }
    }
}
