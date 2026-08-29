//! # Google People commands
//!
//! Dispatches the People command tree to the resource module owning
//! each command.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::people::{
    client::PeopleClient, connection::cli::PeopleConnectionCommand,
    contact_group::cli::PeopleContactGroupCommand, other_contact::cli::PeopleOtherContactCommand,
    profile::cli::PeopleProfileCommand, request::PeopleRequestCommand,
};

/// Google People API-specific API.
///
/// Nested by People resource (contact groups, connections, other
/// contacts, the signed-in user), each command named after its People
/// operation. Works with the raw People model: `create` / `update` take
/// a People person JSON body and `--json` prints the raw People payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PeopleCommand {
    #[command(subcommand, visible_aliases = ["group", "groups"])]
    ContactGroup(PeopleContactGroupCommand),
    #[command(subcommand, visible_aliases = ["people", "contacts"])]
    Connection(PeopleConnectionCommand),
    #[command(subcommand, visible_alias = "other")]
    OtherContact(PeopleOtherContactCommand),
    #[command(subcommand)]
    Profile(PeopleProfileCommand),
    Request(PeopleRequestCommand),
}

impl PeopleCommand {
    pub fn execute(self, printer: &mut impl Printer, client: PeopleClient) -> Result<()> {
        match self {
            Self::ContactGroup(cmd) => cmd.execute(printer, client),
            Self::Connection(cmd) => cmd.execute(printer, client),
            Self::OtherContact(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
        }
    }
}
