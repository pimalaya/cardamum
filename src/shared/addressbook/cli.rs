//! # Addressbook commands
//!
//! Dispatches the unified API addressbook subcommands to the account client.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::capability;
use crate::shared::{
    addressbook::{
        create::AddressbookCreateCommand, delete::AddressbookDeleteCommand,
        list::AddressbookListCommand, update::AddressbookUpdateCommand,
    },
    client::AddressbookClient,
};

/// Manage addressbooks using the unified API.
///
/// Runs against the first backend defined for the default account, or for
/// the account named by `--account`.
#[derive(Debug, Subcommand)]
pub enum AddressbookCommand {
    #[command(visible_alias = "ls", after_long_help = capability::help("addressbook", "list"))]
    List(AddressbookListCommand),
    #[command(after_long_help = capability::help("addressbook", "create"))]
    Create(AddressbookCreateCommand),
    #[command(after_long_help = capability::help("addressbook", "update"))]
    Update(AddressbookUpdateCommand),
    #[command(visible_alias = "rm", after_long_help = capability::help("addressbook", "delete"))]
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
