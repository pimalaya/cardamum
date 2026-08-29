//! # Addressbook commands
//!
//! Dispatches the shared API addressbook subcommands to the account client.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    addressbook::{
        create::AddressbookCreateCommand, delete::AddressbookDeleteCommand,
        list::AddressbookListCommand, update::AddressbookUpdateCommand,
    },
    client::AddressbookClient,
};

/// Manage addressbooks using the shared API.
///
/// Runs against the first backend defined for the default account, or for
/// the account named by `--account`.
#[derive(Debug, Subcommand)]
pub enum AddressbookCommand {
    #[command(visible_alias = "ls")]
    List(AddressbookListCommand),
    Create(AddressbookCreateCommand),
    Update(AddressbookUpdateCommand),
    #[command(visible_alias = "rm")]
    Delete(AddressbookDeleteCommand),
}

impl AddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, client: AddressbookClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
