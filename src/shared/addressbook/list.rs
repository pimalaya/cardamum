//! # Addressbook list command
//!
//! Lists the addressbooks the account exposes, as a colored table or as
//! JSON.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, Row, Table},
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    addressbook::Addressbook, client::AddressbookClient, table::style_from_preset,
};

/// List every addressbook available to the active account.
///
/// JSON output: `{"addressbooks": [{"id", "name", "description",
/// "color"}]}`.
#[derive(Debug, Parser)]
pub struct AddressbookListCommand;

impl AddressbookListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbooks = client.list_addressbooks()?;

        let table = AddressbookListOutput {
            preset: client.account.table_preset().to_string(),
            id_color: client.account.addressbooks_list_table_id_color(),
            name_color: client.account.addressbooks_list_table_name_color(),
            description_color: client.account.addressbooks_list_table_description_color(),
            color_color: client.account.addressbooks_list_table_color_color(),
            rows: addressbooks.into_iter().map(From::from).collect(),
        };

        printer.out(table)
    }
}

/// Table of addressbooks, and the JSON shape the command prints.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddressbookListOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the ID column.
    #[serde(skip)]
    pub id_color: Color,
    /// Color of the NAME column.
    #[serde(skip)]
    pub name_color: Color,
    /// Color of the DESC column.
    #[serde(skip)]
    pub description_color: Color,
    /// Color of the COLOR column.
    #[serde(skip)]
    pub color_color: Color,
    /// The addressbooks the account exposes.
    #[serde(rename = "addressbooks")]
    pub rows: Vec<AddressbookRow>,
}

/// One addressbook, as rendered by [`AddressbookListOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddressbookRow {
    /// Backend-specific identifier of the collection.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Free-form description, when the backend exposes one.
    pub description: Option<String>,
    /// ASCII `#RRGGBB` color marker, when the backend exposes one.
    pub color: Option<String>,
}

impl From<Addressbook> for AddressbookRow {
    fn from(book: Addressbook) -> Self {
        Self {
            id: book.id,
            name: book.name,
            description: book.description,
            color: book.color,
        }
    }
}

impl fmt::Display for AddressbookListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("DESC"),
                Cell::new("COLOR"),
            ]))
            .add_rows(self.rows.iter().map(|book| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&book.id).fg(self.id_color))
                    .add_cell(Cell::new(&book.name).fg(self.name_color))
                    .add_cell(
                        Cell::new(book.description.as_deref().unwrap_or(""))
                            .fg(self.description_color),
                    )
                    .add_cell(Cell::new(book.color.as_deref().unwrap_or("")).fg(self.color_color));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}
