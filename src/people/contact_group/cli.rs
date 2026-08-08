use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::people::{
    client::PeopleClient,
    contact_group::{
        create::PeopleContactGroupCreateCommand, delete::PeopleContactGroupDeleteCommand,
        get::PeopleContactGroupGetCommand, list::PeopleContactGroupListCommand,
        members::PeopleContactGroupMembersCommand, update::PeopleContactGroupUpdateCommand,
    },
};

/// Manage People contact groups (the addressbooks).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PeopleContactGroupCommand {
    List(PeopleContactGroupListCommand),
    Get(PeopleContactGroupGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(PeopleContactGroupCreateCommand),
    Update(PeopleContactGroupUpdateCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(PeopleContactGroupDeleteCommand),
    Members(PeopleContactGroupMembersCommand),
}

impl PeopleContactGroupCommand {
    pub fn execute(self, printer: &mut impl Printer, client: PeopleClient) -> Result<()> {
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
