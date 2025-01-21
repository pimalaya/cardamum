// mod create;
// mod delete;
mod list;
// mod update;

use clap::Subcommand;
use color_eyre::Result;
use pimalaya_tui::terminal::cli::printer::Printer;

use crate::config::TomlConfig;

use self::{
    // create::CreateAddressbookCommand,
    // delete::DeleteAddressbookCommand,
    list::ListAddressbooksCommand,
    // update::UpdateAddressbookCommand,
};

#[derive(Debug, Subcommand)]
pub enum AddressbookSubcommand {
    // #[command(alias = "new", alias = "add")]
    // Create(CreateAddressbookCommand),
    #[command()]
    List(ListAddressbooksCommand),
    // #[command(alias = "set")]
    // Update(UpdateAddressbookCommand),
    // #[command(alias = "remove", alias = "rm")]
    // Delete(DeleteAddressbookCommand),
}

impl AddressbookSubcommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        match self {
            // Self::Create(cmd) => cmd.execute(printer, config),
            Self::List(cmd) => cmd.execute(printer, config),
            // Self::Update(cmd) => cmd.execute(printer, config),
            // Self::Delete(cmd) => cmd.execute(printer, config),
        }
    }
}
