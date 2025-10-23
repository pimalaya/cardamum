mod create;
mod delete;
mod list;
mod read;
mod update;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::account::Account;

use self::{
    create::CreateCardCommand, delete::DeleteCardCommand, list::ListCardsCommand,
    read::ReadCardCommand, update::UpdateCardCommand,
};

/// Create, list, update and delete cards.
///
/// This subcommand allows you to create, list, update and delete
/// cards from addressbooks.
#[derive(Debug, Subcommand)]
pub enum CardSubcommand {
    #[command(alias = "new", alias = "add")]
    Create(CreateCardCommand),
    #[command(alias = "get")]
    Read(ReadCardCommand),
    #[command(alias = "lst")]
    List(ListCardsCommand),
    #[command(alias = "set", alias = "change")]
    Update(UpdateCardCommand),
    #[command(alias = "remove", alias = "rm")]
    Delete(DeleteCardCommand),
}

impl CardSubcommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute(printer, account),
            Self::Read(cmd) => cmd.execute(printer, account),
            Self::List(cmd) => cmd.execute(printer, account),
            Self::Update(cmd) => cmd.execute(printer, account),
            Self::Delete(cmd) => cmd.execute(printer, account),
        }
    }
}
