use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::AddressbookIdArg, client::AddressbookClient};

/// Permanently delete the given card.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CardDeleteCommand {
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// Card UID.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
}

impl CardDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        client.delete_card(&addressbook_id, &self.card_id)?;

        let msg = format!("Card `{}` successfully deleted", self.card_id);
        printer.out(Message::new(msg))
    }
}
