//! # Google People commands
//!
//! Dispatches the People command tree to the resource module owning
//! each command.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{
    client::GpeopleClient, connection::cli::GpeopleConnectionCommand,
    contact_group::cli::GpeopleContactGroupCommand, other_contact::cli::GpeopleOtherContactCommand,
    profile::cli::GpeopleProfileCommand, request::GpeopleRequestCommand,
};

/// Google People API-specific API.
///
/// Nested by People resource (contact groups, connections, other
/// contacts, the signed-in user), each command named after its People
/// operation. Works with the raw People model: `create` / `update` take
/// a People person JSON body and `--json` prints the raw People payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GpeopleCommand {
    #[command(subcommand, visible_aliases = ["group", "groups"])]
    ContactGroup(GpeopleContactGroupCommand),
    #[command(subcommand, visible_aliases = ["people", "contacts"])]
    Connection(GpeopleConnectionCommand),
    #[command(subcommand, visible_alias = "other")]
    OtherContact(GpeopleOtherContactCommand),
    #[command(subcommand)]
    Profile(GpeopleProfileCommand),
    Request(GpeopleRequestCommand),
}

impl GpeopleCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GpeopleClient) -> Result<()> {
        match self {
            Self::ContactGroup(cmd) => cmd.execute(printer, client),
            Self::Connection(cmd) => cmd.execute(printer, client),
            Self::OtherContact(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
        }
    }
}
