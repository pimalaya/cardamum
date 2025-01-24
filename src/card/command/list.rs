use clap::Parser;
use color_eyre::Result;
use pimalaya_tui::terminal::{cli::printer::Printer, config::TomlConfig as _};

use crate::{
    account::arg::name::AccountNameFlag, card::table::CardsTable, config::TomlConfig, Client,
};

/// List all folders.
///
/// This command allows you to list all exsting folders.
#[derive(Debug, Parser)]
pub struct ListCardsCommand {
    #[command(flatten)]
    pub account: AccountNameFlag,

    /// The identifier of the CardDAV addressbook to list vCards from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,
}

impl ListCardsCommand {
    pub fn execute(self, printer: &mut impl Printer, config: TomlConfig) -> Result<()> {
        let (_, config) = config.to_toml_account_config(self.account.name.as_deref())?;
        let client = Client::new(config.backend)?;

        let cards = client.list_cards(self.addressbook_id)?;
        let table = CardsTable::from(cards);
        printer.out(table)
    }
}
