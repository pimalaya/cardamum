//! # Card read command
//!
//! Fetches a single card and prints its raw vCard.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::Capability;
use crate::shared::{arg::AddressbookIdArg, card::project::CardFields, client::AddressbookClient};

/// Read the raw vCard bytes of the given card.
///
/// JSON output: `{"id", "etag", "contents", "uid", "fullName",
/// "givenName", "familyName", "emails", "phones", "organization",
/// "organizationUnits", "title", "note"}`, with the raw vCard in
/// `contents` and its common fields beside it.
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
    pub fn execute(self, printer: &mut impl Printer, client: &mut AddressbookClient) -> Result<()> {
        client.require(&[Capability::CardRead])?;
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let card = client.get_card(&addressbook_id, &self.card_id)?;
        let card = CardReadOutput {
            fields: CardFields::project(&card.contents),
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
    /// The common fields, read through the vCard's decoded model.
    #[serde(flatten)]
    pub fields: CardFields,
}

impl fmt::Display for CardReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: the card is the server's text: its layout is kept, CRLF
        // read as LF, and a character that could drive the terminal is
        // replaced. `--json` keeps the bytes.
        let contents = self.contents.replace("\r\n", "\n");
        let printable = contents.chars().map(|c| match c {
            '\t' | '\n' => c,
            c if c.is_control() => char::REPLACEMENT_CHARACTER,
            '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' => char::REPLACEMENT_CHARACTER,
            c => c,
        });

        for c in printable {
            write!(f, "{c}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A card is the server's text: an escape sequence or a bidi override
    /// in it never reaches the terminal, while the JSON keeps its bytes.
    #[test]
    fn the_text_view_defuses_what_could_drive_the_terminal() {
        let contents = "BEGIN:VCARD\r\nFN:Ada\x1b]52;c;AAAA\x07\u{202e}\r\nEND:VCARD\r\n";
        let output = CardReadOutput {
            id: String::from("ada"),
            etag: None,
            contents: contents.to_string(),
            fields: CardFields::default(),
        };

        let expected = "BEGIN:VCARD\nFN:Ada\u{fffd}]52;c;AAAA\u{fffd}\u{fffd}\nEND:VCARD\n";
        assert_eq!(output.to_string(), expected);

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["contents"], contents);
    }
}
