//! # DELETE command
//!
//! Runs a WebDAV `DELETE` against a card.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    carddav::client::{CarddavClient, card_refusal},
    shared::receipt::IdReceipt,
};

/// DELETE a card.
///
/// Deletes one card, gated on `--if-match`. An address book collection
/// is deleted by `carddav rmcol`.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct CarddavDeleteCommand {
    /// Identifier of the addressbook (its last URL segment).
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Card resource id (its href last path segment).
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
    /// Gate the delete on the resource being unchanged (If-Match).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl CarddavDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        client
            .delete_card(
                &self.addressbook_id,
                &self.card_id,
                self.if_match.as_deref(),
            )
            .map_err(card_refusal)?;

        printer.out(IdReceipt {
            id: self.card_id,
            done: "Card deleted",
        })
    }
}
