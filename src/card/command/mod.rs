mod create;
mod delete;
mod list;
mod read;
mod update;

use clap::Subcommand;
use color_eyre::Result;
use pimalaya_tui::terminal::cli::printer::Printer;

use crate::config::TomlConfig;

use self::{
    create::CreateCardCommand, delete::DeleteCardCommand, list::ListCardsCommand,
    read::ReadCardCommand, update::UpdateCardCommand,
};

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
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute(printer, config),
            Self::Read(cmd) => cmd.execute(printer, config),
            Self::List(cmd) => cmd.execute(printer, config),
            Self::Update(cmd) => cmd.execute(printer, config),
            Self::Delete(cmd) => cmd.execute(printer, config),
        }
    }
}
