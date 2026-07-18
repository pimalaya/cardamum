use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, Row, Table};
use io_webdav::rfc6352::{addressbook::Addressbook, card::CardRef};
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::carddav::client::CarddavClient;

/// PROPFIND the home-set or an addressbook.
///
/// Without an addressbook, lists the addressbook collections with their
/// DAV properties, including the **CTag** and **sync-token** the shared
/// `addressbook list` hides. With one, enumerates its card resources
/// (id + ETag, no bodies) — the lightweight `getetag` PROPFIND.
///
/// JSON output: `{"addressbooks": [...]}` (no arg) or `{"cards":
/// [{"id", "etag"}]}` (with arg).
#[derive(Debug, Parser)]
pub struct CarddavPropfindCommand {
    /// Addressbook to enumerate; omit to list the addressbook
    /// collections under the home-set.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: Option<String>,
}

impl CarddavPropfindCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();

        match self.addressbook_id {
            None => {
                let id_color = client.account.addressbooks_list_table_id_color();
                let name_color = client.account.addressbooks_list_table_name_color();
                let books = client.list_addressbooks()?;
                printer.out(AddressbooksReport {
                    preset,
                    id_color,
                    name_color,
                    rows: books.into_iter().map(AddressbookRow::from).collect(),
                })
            }
            Some(id) => {
                let id_color = client.account.cards_list_table_id_color();
                let refs = client.enum_cards(&id)?;
                printer.out(CardRefsReport {
                    preset,
                    id_color,
                    rows: refs.into_iter().map(CardRefRow::from).collect(),
                })
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AddressbooksReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(skip)]
    pub name_color: Color,
    #[serde(rename = "addressbooks")]
    pub rows: Vec<AddressbookRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AddressbookRow {
    pub id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub ctag: Option<String>,
    pub sync_token: Option<String>,
}

impl From<Addressbook> for AddressbookRow {
    fn from(book: Addressbook) -> Self {
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

impl fmt::Display for AddressbooksReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("COLOR"),
                Cell::new("CTAG"),
                Cell::new("SYNC-TOKEN"),
            ]))
            .add_rows(self.rows.iter().map(|book| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&book.id).fg(self.id_color))
                    .add_cell(
                        Cell::new(book.display_name.as_deref().unwrap_or("")).fg(self.name_color),
                    )
                    .add_cell(Cell::new(book.color.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(book.ctag.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(book.sync_token.as_deref().unwrap_or("")));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CardRefsReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "cards")]
    pub rows: Vec<CardRefRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CardRefRow {
    pub id: String,
    pub etag: Option<String>,
}

impl From<CardRef> for CardRefRow {
    fn from(card: CardRef) -> Self {
        Self {
            id: card.id,
            etag: card.etag,
        }
    }
}

impl fmt::Display for CardRefsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_header(Row::from([Cell::new("ID"), Cell::new("ETAG")]))
            .add_rows(self.rows.iter().map(|card| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&card.id).fg(self.id_color))
                    .add_cell(Cell::new(card.etag.as_deref().unwrap_or("")));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}
