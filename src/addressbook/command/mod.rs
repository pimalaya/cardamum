// mod create;
// mod delete;
mod list;
// mod update;
// #[cfg(feature = "carddav")]
// mod discover;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::account::Account;

// #[cfg(feature = "carddav")]
// use self::discover::DiscoverAddressbooksCommand;
use self::{
    // create::CreateAddressbookCommand,
    // delete::DeleteAddressbookCommand,
    list::ListAddressbooksCommand,
    // update::UpdateAddressbookCommand,
};

#[derive(Debug, Subcommand)]
pub enum AddressbookSubcommand {
    // #[cfg(feature = "carddav")]
    // Discover(DiscoverAddressbooksCommand),
    // #[command(alias = "new", alias = "add")]
    // Create(CreateAddressbookCommand),
    List(ListAddressbooksCommand),
    // #[command(alias = "set")]
    // Update(UpdateAddressbookCommand),
    // #[command(alias = "remove", alias = "rm")]
    // Delete(DeleteAddressbookCommand),
}

impl AddressbookSubcommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            // Self::Discover(cmd) => cmd.execute(printer, account),
            // Self::Create(cmd) => cmd.execute(printer, config),
            Self::List(cmd) => cmd.execute(printer, account),
            // Self::Update(cmd) => cmd.execute(printer, config),
            // Self::Delete(cmd) => cmd.execute(printer, config),
        }
    }
}
