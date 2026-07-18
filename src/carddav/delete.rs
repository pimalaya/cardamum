use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::carddav::client::CarddavClient;

/// DELETE a card, or a whole addressbook collection.
///
/// With a card id, deletes that card (gate on `--if-match`); without
/// one, deletes the addressbook collection and every card it contains.
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CarddavDeleteCommand {
    /// Identifier of the addressbook (its last URL segment).
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Card resource id; omit to delete the whole collection.
    #[arg(value_name = "CARD-ID")]
    pub card_id: Option<String>,
    /// Gate a card delete on the resource being unchanged (If-Match).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl CarddavDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let message = match self.card_id {
            Some(card_id) => {
                client.delete_card(&self.addressbook_id, &card_id, self.if_match.as_deref())?;
                format!("Card `{card_id}` successfully deleted")
            }
            None => {
                if self.if_match.is_some() {
                    bail!("--if-match applies to a card delete, not to a collection");
                }
                client.delete_addressbook(&self.addressbook_id)?;
                format!("Addressbook `{}` successfully deleted", self.addressbook_id)
            }
        };

        printer.out(Message::new(message))
    }
}
