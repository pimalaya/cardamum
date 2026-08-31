//! # Card read command
//!
//! Fetches a single card and prints its raw vCard.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{arg::AddressbookIdArg, client::AddressbookClient};

/// Read the raw vCard bytes of the given card.
///
/// JSON output: `{"id", "etag", "contents"}`, with the raw vCard in
/// `contents`.
#[derive(Debug, Parser)]
pub struct CardReadCommand {
    /// Addressbook holding the card.
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// Card to read, as `card list` reports it.
    ///
    /// This is the backend's own id, not the vCard `UID`, which names no
    /// card on its own.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
}

impl CardReadCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let card = client.get_card(&addressbook_id, &self.card_id)?;
        let card = CardReadOutput {
            id: card.id,
            etag: card.etag,
            contents: String::from_utf8(card.contents)?,
        };

        printer.out(card)
    }
}

/// The card the command prints, with its vCard as text.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardReadOutput {
    /// Backend-specific identifier of the card.
    pub id: String,
    /// Entity tag, when the backend exposes one.
    pub etag: Option<String>,
    /// The raw vCard, as text.
    pub contents: String,
}

impl fmt::Display for CardReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents)
    }
}
