//! # PROPFIND command
//!
//! Enumerates the addressbook collections, or the card resources of
//! one, with a WebDAV `PROPFIND`.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_webdav::rfc6352::{addressbook::CarddavAddressbook, card::CarddavCardRef};
use pimalaya_cli::{
    columns::Columns,
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::carddav::client::CarddavClient;

/// PROPFIND the home-set or an addressbook.
///
/// Without an addressbook, lists the addressbook collections with their
/// DAV properties, including the CTag and sync token the unified
/// `addressbook list` hides. With one, enumerates its card resources with
/// ids and ETags but no bodies, through the `addressbook-query` REPORT
/// asking `getetag` alone, as io-webdav enumerates a collection.
///
/// JSON output: `{"addressbooks": [...]}` (no arg) or `{"cards": [{"id",
/// "etag"}], "truncated"}` (with arg).
#[derive(Debug, Parser)]
pub struct CarddavPropfindCommand {
    /// Addressbook to enumerate; omit to list the addressbook
    /// collections under the home-set.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: Option<String>,
}

impl CarddavPropfindCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        match self.addressbook_id {
            None => {
                let books = client.list_addressbooks()?;
                printer.out(CarddavPropfindOutput::Addressbooks(
                    CarddavPropfindAddressbooksOutput {
                        rows: books.into_iter().map(AddressbookRow::from).collect(),
                    },
                ))
            }
            Some(id) => {
                let enumerated = client.enum_cards(&id)?;
                printer.out(CarddavPropfindOutput::Cards(CarddavPropfindCardsOutput {
                    rows: enumerated.refs.into_iter().map(CardRefRow::from).collect(),
                    truncated: enumerated.truncated,
                }))
            }
        }
    }
}

/// What a PROPFIND prints, which is what it was pointed at.
///
/// Untagged, so each shape serializes exactly as it would on its own: the
/// argument already says which of the two came back.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum CarddavPropfindOutput {
    /// The addressbook collections under the home-set.
    Addressbooks(CarddavPropfindAddressbooksOutput),
    /// The card resources of one addressbook.
    Cards(CarddavPropfindCardsOutput),
}

impl fmt::Display for CarddavPropfindOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Addressbooks(out) => out.fmt(f),
            Self::Cards(out) => out.fmt(f),
        }
    }
}

/// Addressbook collections listed by a home-set PROPFIND.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavPropfindAddressbooksOutput {
    /// The addressbook collections under the home-set.
    #[serde(rename = "addressbooks")]
    pub rows: Vec<AddressbookRow>,
}

/// One addressbook collection with its DAV properties.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddressbookRow {
    /// Collection id, the last segment of its href, percent-decoded when the
    /// decoding encodes back to it.
    pub id: String,
    /// The `displayname` property.
    pub display_name: Option<String>,
    /// The CardDAV `addressbook-description` property.
    pub description: Option<String>,
    /// The Apple `calendar-color` property.
    pub color: Option<String>,
    /// The Apple `getctag` property, which changes on any member write.
    pub ctag: Option<String>,
    /// The RFC 6578 `sync-token`, to feed to `report sync`.
    pub sync_token: Option<String>,
}

impl From<CarddavAddressbook> for AddressbookRow {
    fn from(book: CarddavAddressbook) -> Self {
        Self {
            id: book.id,
            display_name: book.display_name,
            description: book.description,
            color: book.color,
            ctag: book.ctag,
            sync_token: book.sync_token,
        }
    }
}

impl fmt::Display for CarddavPropfindAddressbooksOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for book in &self.rows {
            // NOTE: a property the server did not send prints no line.
            let mut fields = Fields::default();
            let props = [
                ("Name", &book.display_name),
                ("Description", &book.description),
                ("Color", &book.color),
                ("CTag", &book.ctag),
                ("Sync token", &book.sync_token),
            ];

            for (label, value) in props {
                if let Some(value) = value {
                    fields.push(label, sanitize::text(value));
                }
            }

            records.push(Record {
                title: sanitize::text(&book.id).into_owned(),
                state: Vec::new(),
                fields,
                body: String::new(),
            });
        }

        write!(f, "{records}")
    }
}

/// Card references listed by an addressbook PROPFIND.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavPropfindCardsOutput {
    /// The card resources the addressbook holds.
    #[serde(rename = "cards")]
    pub rows: Vec<CardRefRow>,
    /// Whether the server cut the listing short with a 507 row.
    ///
    /// Such a listing (RFC 6578 §3.6) is a part of the addressbook
    /// rather than the whole of it.
    pub truncated: bool,
}

/// One card resource: its id and ETag, never its body.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardRefRow {
    /// Card resource id, the last segment of its href, percent-decoded when the
    /// decoding encodes back to it.
    pub id: String,
    /// The server ETag, when it sent one.
    pub etag: Option<String>,
}

impl From<CarddavCardRef> for CardRefRow {
    fn from(card: CarddavCardRef) -> Self {
        Self {
            id: card.id,
            etag: card.etag,
        }
    }
}

impl fmt::Display for CarddavPropfindCardsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.rows.is_empty() {
            let mut columns = Columns::new(["ID", "ETAG"]);

            for card in &self.rows {
                columns.push([sanitize::text(&card.id).into_owned(), text(&card.etag)]);
            }

            write!(f, "{columns}")?;
        }

        if self.truncated {
            if !self.rows.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "truncated: the server cut the listing short")?;
        }

        Ok(())
    }
}

/// Renders an optional server value, sanitized, or nothing.
fn text(value: &Option<String>) -> String {
    match value {
        Some(value) => sanitize::text(value).into_owned(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book(id: &str) -> AddressbookRow {
        AddressbookRow {
            id: id.to_string(),
            display_name: None,
            description: None,
            color: None,
            ctag: None,
            sync_token: None,
        }
    }

    /// A book prints the properties the server sent, its description
    /// included, and no empty line for one it did not send.
    #[test]
    fn a_book_prints_the_properties_it_has() {
        let output = CarddavPropfindAddressbooksOutput {
            rows: vec![
                AddressbookRow {
                    description: Some(String::from("Default addressbook")),
                    ctag: Some(String::from("1774329965-9277")),
                    ..book("Default")
                },
                AddressbookRow {
                    display_name: Some(String::from("Personal")),
                    color: Some(String::from("#0000FF")),
                    sync_token: Some(String::from("data:,1")),
                    ..book("card")
                },
            ],
        };

        let expected = "\
Default
Description:  Default addressbook
CTag:         1774329965-9277

card
Name:        Personal
Color:       #0000FF
Sync token:  data:,1
";
        assert_eq!(output.to_string(), expected);
    }

    /// The request `carddav propfind` sends, given `addressbook_id` or not.
    fn sent(addressbook_id: Option<&str>) -> String {
        use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
        use pimalaya_cli::printer::CapturePrinter;
        use url::Url;

        use crate::script::{self, Script};

        let empty = r#"<d:multistatus xmlns:d="DAV:"/>"#;
        let (script, requests) = Script::new([script::response(207, "application/xml", empty)]);
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, None, Some(home));

        let command = CarddavPropfindCommand {
            addressbook_id: addressbook_id.map(str::to_string),
        };
        let printer = &mut CapturePrinter::default();
        command.execute(printer, CarddavClient::new(inner)).unwrap();

        let requests = requests.lock().unwrap();
        String::from_utf8_lossy(&requests[0]).into_owned()
    }

    /// Without an address book the command sends a PROPFIND of the home set,
    /// with one the `addressbook-query` REPORT asking `getetag` alone, as its
    /// help says.
    #[test]
    fn an_addressbook_is_enumerated_through_a_report() {
        let home = sent(None);
        assert!(home.starts_with("PROPFIND /dav/ HTTP/1.1\r\n"), "{home}");

        let book = sent(Some("book"));
        assert!(book.starts_with("REPORT /dav/book/ HTTP/1.1\r\n"), "{book}");
        assert!(book.contains("addressbook-query"), "{book}");
        assert!(book.contains("getetag"), "{book}");
        assert!(!book.contains("address-data"), "{book}");
    }
}
