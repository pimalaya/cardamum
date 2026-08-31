//! # Card commands
//!
//! Dispatches the shared API card subcommands to the account client.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    card::{
        create::CardCreateCommand, delete::CardDeleteCommand, list::CardListCommand,
        read::CardReadCommand, update::CardUpdateCommand,
    },
    client::AddressbookClient,
};

/// Manage vCards using the shared API.
///
/// Runs against the first backend defined for the default account, or for
/// the account named by `--account`.
#[derive(Debug, Subcommand)]
pub enum CardCommand {
    #[command(visible_alias = "ls")]
    List(CardListCommand),
    Read(CardReadCommand),
    #[command(visible_alias = "new")]
    Create(Box<CardCreateCommand>),
    Update(Box<CardUpdateCommand>),
    #[command(visible_alias = "rm")]
    Delete(CardDeleteCommand),
}

impl CardCommand {
    pub fn execute(self, printer: &mut impl Printer, client: AddressbookClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Read(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
