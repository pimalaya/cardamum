//! # DELETE command
//!
//! Runs a WebDAV `DELETE` against a card or a whole addressbook
//! collection.

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{carddav::client::CarddavClient, shared::receipt::IdReceipt};

/// DELETE a card, or a whole addressbook collection.
///
/// With a card id, deletes that card, gated on `--if-match`; without
/// one, deletes the addressbook collection and every card it contains.
///
/// JSON output: `{"id"}`.
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
        let receipt = match self.card_id {
            Some(card_id) => {
                client.delete_card(&self.addressbook_id, &card_id, self.if_match.as_deref())?;
                IdReceipt {
                    id: card_id,
                    done: "Card deleted",
                }
            }
            None => {
                if self.if_match.is_some() {
                    bail!("--if-match applies to a card delete, not to a collection");
                }
                client.delete_addressbook(&self.addressbook_id)?;
                IdReceipt {
                    id: self.addressbook_id,
                    done: "Addressbook deleted",
                }
            }
        };

        printer.out(receipt)
    }
}
