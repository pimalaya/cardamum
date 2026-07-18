use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::google::{
    client::GoogleClient,
    connection::{
        create::GoogleConnectionCreateCommand, delete::GoogleConnectionDeleteCommand,
        get::GoogleConnectionGetCommand, list::GoogleConnectionListCommand,
        search::GoogleConnectionSearchCommand, update::GoogleConnectionUpdateCommand,
    },
};

/// Manage the signed-in user's contacts (`people.connections`).
///
/// `create` / `update` take a raw People person JSON body (file, inline,
/// or `-` for stdin); `--json` prints the raw People person.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GoogleConnectionCommand {
    List(GoogleConnectionListCommand),
    Get(GoogleConnectionGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(GoogleConnectionCreateCommand),
    Update(GoogleConnectionUpdateCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(GoogleConnectionDeleteCommand),
    Search(GoogleConnectionSearchCommand),
}

impl GoogleConnectionCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GoogleClient) -> Result<()> {
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
