//! # Connection commands
//!
//! Dispatches the contact commands to the People operation each runs.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{
    client::GpeopleClient,
    connection::{
        create::GpeopleConnectionCreateCommand, delete::GpeopleConnectionDeleteCommand,
        get::GpeopleConnectionGetCommand, list::GpeopleConnectionListCommand,
        search::GpeopleConnectionSearchCommand, update::GpeopleConnectionUpdateCommand,
    },
};

/// Manage the signed-in user's contacts (`people.connections`).
///
/// `create` / `update` take a raw People person JSON body (file, inline,
/// or `-` for stdin); `--json` prints the raw People person.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GpeopleConnectionCommand {
    List(GpeopleConnectionListCommand),
    Get(GpeopleConnectionGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(GpeopleConnectionCreateCommand),
    Update(GpeopleConnectionUpdateCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(GpeopleConnectionDeleteCommand),
    Search(GpeopleConnectionSearchCommand),
}

impl GpeopleConnectionCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GpeopleClient) -> Result<()> {
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
