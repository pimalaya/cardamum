use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::client::AddressbookClient;

/// Permanently delete the given card.
#[derive(Debug, Parser)]
pub struct CardDeleteCommand {
    /// Identifier of the parent addressbook. Falls back to the
    /// `addressbook.default` config when omitted.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub addressbook_id: Option<String>,
    /// Card UID.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
}

impl CardDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook_id)?;
        client.delete_card(&addressbook_id, &self.card_id)?;

        let msg = format!("Card `{}` successfully deleted", self.card_id);
        printer.out(Message::new(msg))
    }
}
