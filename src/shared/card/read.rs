//! # Card read command
//!
//! Fetches a single card and prints the row `card list` prints for it,
//! plus its raw vCard.

use core::fmt;

use anyhow::Result;
use base64_codec::codec::Base64Codec;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::Capability;
use crate::shared::{arg::AddressbookIdArg, card::list::CardRow, client::AddressbookClient};

/// Read a card: its row as `card list` prints it, and its raw vCard.
///
/// JSON output: the `card list` row (`{"id", "addressbookId", "etag",
/// "fnValue", "email", "tel", "uid", "fullName", "givenName",
/// "familyName", "emails", "phones", "organization", "organizationUnits",
/// "title", "note"}`, `etag` `null` where the backend versions no card)
/// plus `"contents"`, the raw vCard as text (`"encoding"` `utf-8`), or its
/// bytes in base64 when they are not UTF-8 (`"encoding"` `base64`).
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
        let contents = CardContents::new(card.contents.clone());

        printer.out(CardReadOutput {
            row: CardRow::from(card),
            contents,
        })
    }
}

/// The card the command prints: its listing row, plus its vCard.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardReadOutput {
    /// The row `card list` prints for the card.
    #[serde(flatten)]
    pub row: CardRow,
    /// The raw vCard.
    #[serde(flatten)]
    pub contents: CardContents,
}

impl fmt::Display for CardReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: the card is the server's text: its layout is kept, CRLF
        // read as LF, and a character that could drive the terminal is
        // replaced. `--json` keeps the bytes.
        let contents = String::from_utf8_lossy(&self.contents.bytes).replace("\r\n", "\n");
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

/// A vCard as JSON carries it: a string, the text when its bytes are
/// UTF-8 and their base64 otherwise, `encoding` telling which, so the text
/// is never decoded lossily nor mistaken for the base64. Calendula prints
/// an item's bytes the same way.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardContents {
    /// The raw vCard as text, or its bytes in base64 when `encoding` is
    /// `base64`.
    pub contents: String,
    /// How `contents` spells the bytes.
    pub encoding: CardEncoding,
    /// The bytes, which the text form prints.
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

impl CardContents {
    /// The vCard `bytes`, as text when they are UTF-8, else in base64.
    pub fn new(bytes: Vec<u8>) -> Self {
        let (contents, encoding) = match String::from_utf8(bytes.clone()) {
            Ok(text) => (text, CardEncoding::Utf8),
            Err(_) => {
                let mut encoded = Vec::new();
                Base64Codec::default().encode_vec(&bytes, &mut encoded);
                // NOTE: the base64 alphabet is ASCII, so this never falls
                // back.
                let encoded = String::from_utf8(encoded).unwrap_or_default();
                (encoded, CardEncoding::Base64)
            }
        };

        Self {
            contents,
            encoding,
            bytes,
        }
    }
}

/// How a card's `contents` spells its bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
pub enum CardEncoding {
    /// The text itself, the bytes being valid UTF-8.
    #[serde(rename = "utf-8")]
    Utf8,
    /// The standard base64 alphabet, padded (RFC 4648 §4), the bytes not
    /// being valid UTF-8.
    #[serde(rename = "base64")]
    Base64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::card::Card;

    fn output(contents: &[u8]) -> CardReadOutput {
        let card = Card {
            id: String::from("ada"),
            addressbook_id: String::from("personal"),
            etag: None,
            contents: contents.to_vec(),
        };

        CardReadOutput {
            contents: CardContents::new(card.contents.clone()),
            row: CardRow::from(card),
        }
    }

    /// A card is the server's text: an escape sequence or a bidi override
    /// in it never reaches the terminal, while the JSON keeps its bytes.
    #[test]
    fn the_text_view_defuses_what_could_drive_the_terminal() {
        let contents = "BEGIN:VCARD\r\nFN:Ada\x1b]52;c;AAAA\x07\u{202e}\r\nEND:VCARD\r\n";
        let output = output(contents.as_bytes());

        let expected = "BEGIN:VCARD\nFN:Ada\u{fffd}]52;c;AAAA\u{fffd}\u{fffd}\nEND:VCARD\n";
        assert_eq!(output.to_string(), expected);

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["contents"], contents);
        assert_eq!(json["encoding"], "utf-8", "{json}");
    }

    /// A read prints the row its listing prints, the same keys with the
    /// same values, then the vCard.
    #[test]
    fn a_read_prints_the_row_of_its_listing() {
        let contents = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:ada\r\nFN:Ada Lovelace\r\n\
                        EMAIL:ada@example.org\r\nTEL:+33 6\r\nEND:VCARD\r\n";
        let read = serde_json::to_value(output(contents.as_bytes())).unwrap();
        let row = serde_json::to_value(output(contents.as_bytes()).row).unwrap();

        for (key, value) in row.as_object().unwrap() {
            assert_eq!(&read[key], value, "{key}: {read}");
        }
        assert_eq!(read["addressbookId"], "personal", "{read}");
        assert_eq!(read["fnValue"], "Ada Lovelace", "{read}");
        assert_eq!(read["email"], "ada@example.org", "{read}");
        assert_eq!(read["tel"], "+33 6", "{read}");
        assert!(read["etag"].is_null(), "{read}");
        assert_eq!(read["contents"], contents, "{read}");
    }

    /// A card whose bytes are not UTF-8 reads as base64, `encoding` saying
    /// so, never decoded lossily.
    #[test]
    fn a_card_that_is_not_utf8_reads_as_base64() {
        let contents = b"BEGIN:VCARD\r\nFN:Ad\xe9\r\nEND:VCARD\r\n";
        let json = serde_json::to_value(output(contents)).unwrap();

        let encoded = "QkVHSU46VkNBUkQNCkZOOkFk6Q0KRU5EOlZDQVJEDQo=";
        assert_eq!(json["contents"], encoded, "{json}");
        assert_eq!(json["encoding"], "base64", "{json}");
        assert_eq!(json["id"], "ada", "{json}");
        assert!(json["fullName"].is_null(), "{json}");
    }
}
