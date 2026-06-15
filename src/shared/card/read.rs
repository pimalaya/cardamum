use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::shared::{arg::AddressbookIdArg, client::AddressbookClient};

/// Read the raw vCard bytes of the given card.
///
/// JSON output: `{"id", "etag", "contents"}`, with the raw vCard in
/// `contents`.
#[derive(Debug, Parser)]
pub struct CardReadCommand {
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// Card UID.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
}

impl CardReadCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let card = client.get_card(&addressbook_id, &self.card_id)?;
        let card = Card {
            id: card.id,
            etag: card.etag,
            contents: String::from_utf8(card.contents)?,
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
