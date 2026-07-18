use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::google::{
    client::GoogleClient, connection::cli::GoogleConnectionCommand,
    contact_group::cli::GoogleContactGroupCommand, other_contact::cli::GoogleOtherContactCommand,
    profile::cli::GoogleProfileCommand,
};

/// Google People API-specific API.
///
/// Nested by People resource (contact groups, connections, other
/// contacts, the signed-in user), each command named after its People
/// operation. Works with the raw People model: `create` / `update` take
/// a People person JSON body and `--json` prints the raw People payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GoogleCommand {
    #[command(subcommand, visible_aliases = ["group", "groups"])]
    ContactGroup(GoogleContactGroupCommand),
    #[command(subcommand, visible_aliases = ["people", "contacts"])]
    Connection(GoogleConnectionCommand),
    #[command(subcommand, visible_alias = "other")]
    OtherContact(GoogleOtherContactCommand),
    #[command(subcommand)]
    Profile(GoogleProfileCommand),
}

impl GoogleCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GoogleClient) -> Result<()> {
        match self {
            Self::ContactGroup(cmd) => cmd.execute(printer, client),
            Self::Connection(cmd) => cmd.execute(printer, client),
            Self::OtherContact(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
        }
    }
}
