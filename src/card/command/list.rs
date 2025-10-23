use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::{account::Account, card::table::CardsTable, client::Client};

/// List all cards.
///
/// This command allows you to list vCards from a given addressbook.
#[derive(Debug, Parser)]
pub struct ListCardsCommand {
    /// The identifier of the CardDAV addressbook to list vCards from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,
}

impl ListCardsCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let cards = client.list_cards(self.addressbook_id)?;
        let table = CardsTable::from(cards);
        printer.out(table)
    }
}
