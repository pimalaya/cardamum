//! # Card delete command
//!
//! Permanently deletes a card from an addressbook.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    arg::AddressbookIdArg,
    client::AddressbookClient,
    receipt::{Write, WriteReceipt},
};

/// Permanently delete the given card.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct CardDeleteCommand {
    /// Addressbook holding the card.
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// Card to delete, as `card list` reports it.
    ///
    /// This is the backend's own id, not the vCard `UID`, which names no
    /// card on its own.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,

    /// Gate the delete on this ETag, as `card list` or `card read`
    /// reports it.
    ///
    /// The delete only lands if the backend still holds that version
    /// (RFC 9110 `If-Match`), which is how a concurrent write is caught.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl CardDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        client.delete_card(&addressbook_id, &self.card_id, self.if_match.as_deref())?;

        printer.out(WriteReceipt {
            id: self.card_id,
            queued: client.queues_writes(),
            what: "Card",
            write: Write::Delete,
        })
    }
}
