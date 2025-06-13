mod create;
mod delete;
mod list;
mod update;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::account::Account;

use self::{
    create::CreateAddressbookCommand, delete::DeleteAddressbookCommand,
    list::ListAddressbooksCommand, update::UpdateAddressbookCommand,
};

#[derive(Debug, Subcommand)]
pub enum AddressbookSubcommand {
    #[command(alias = "new", alias = "add")]
    Create(CreateAddressbookCommand),
    List(ListAddressbooksCommand),
    #[command(alias = "set")]
    Update(UpdateAddressbookCommand),
    #[command(alias = "remove", alias = "rm")]
    Delete(DeleteAddressbookCommand),
}

impl AddressbookSubcommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute(printer, account),
            Self::List(cmd) => cmd.execute(printer, account),
            Self::Update(cmd) => cmd.execute(printer, account),
            Self::Delete(cmd) => cmd.execute(printer, account),
        }
    }
}
