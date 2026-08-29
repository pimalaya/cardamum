//! # Connection commands
//!
//! Dispatches the contact commands to the People operation each runs.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::people::{
    client::PeopleClient,
    connection::{
        create::PeopleConnectionCreateCommand, delete::PeopleConnectionDeleteCommand,
        get::PeopleConnectionGetCommand, list::PeopleConnectionListCommand,
        search::PeopleConnectionSearchCommand, update::PeopleConnectionUpdateCommand,
    },
};

/// Manage the signed-in user's contacts (`people.connections`).
///
/// `create` / `update` take a raw People person JSON body (file, inline,
/// or `-` for stdin); `--json` prints the raw People person.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PeopleConnectionCommand {
    List(PeopleConnectionListCommand),
    Get(PeopleConnectionGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(PeopleConnectionCreateCommand),
    Update(PeopleConnectionUpdateCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(PeopleConnectionDeleteCommand),
    Search(PeopleConnectionSearchCommand),
}

impl PeopleConnectionCommand {
    pub fn execute(self, printer: &mut impl Printer, client: PeopleClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::Search(cmd) => cmd.execute(printer, client),
        }
    }
}
