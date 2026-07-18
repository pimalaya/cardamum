use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::carddav::client::CarddavClient;

/// GET a card: its raw vCard bytes plus the server ETag.
///
/// JSON output: `{"id", "etag", "contents"}`.
#[derive(Debug, Parser)]
pub struct CarddavGetCommand {
    /// Identifier of the parent addressbook.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Card resource id (its href last path segment).
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
}

impl CarddavGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let body = client.read_card(&self.addressbook_id, &self.card_id)?;

        let card = Card {
            id: self.card_id,
            etag: body.etag,
            contents: String::from_utf8(body.data)?,
        };

        printer.out(card)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Card {
    pub id: String,
    pub etag: Option<String>,
    pub contents: String,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents)
    }
}
