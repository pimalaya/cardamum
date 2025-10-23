use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::{account::Account, client::Client};

/// Read the content of a card.
///
/// This command allows you to read the content of a vCard, from the
/// given addressbook.
#[derive(Debug, Parser)]
pub struct ReadCardCommand {
    /// The identifier of the addressbook where the vCard should be
    /// read from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,

    /// The identifier of the card that should be read.
    #[arg(name = "CARD-ID")]
    pub id: String,
}

impl ReadCardCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;
        let card = client.read_card(self.addressbook_id, self.id)?;
        printer.out(card.to_string().trim_end())
    }
}
